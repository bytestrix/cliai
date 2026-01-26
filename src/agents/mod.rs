use crate::config::Config;
use crate::history::{History, ContextWindow, ContextPriority};
use crate::context::ContextGatherer;
use crate::validation::{CommandValidator, DefaultCommandValidator, ValidationResult};
use crate::builtin_commands::BuiltinCommands;
use crate::os_context::OSContext;
use crate::intent::{IntentClassifier, UserIntent, IntentAnalysis};
use crate::providers::{ProviderManager, OllamaProvider, CloudProvider, ProviderType, CircuitBreakerState};
use crate::performance::{OperationType, PerformanceMonitor, SystemPerformanceSummary};
use anyhow::{Result, anyhow};
use serde_json::json;
use crate::agents::profiles::*;
use std::fs::OpenOptions;
use std::io::Write;
use std::time::Instant;

pub mod profiles;

pub struct Orchestrator {
    config: Config,
    history: History,
    validator: DefaultCommandValidator,
    builtin_commands: BuiltinCommands,
    os_context: OSContext,
    context_gatherer: ContextGatherer,
    intent_classifier: IntentClassifier,
    provider_manager: ProviderManager,
}

impl Orchestrator {
    pub fn new(config: Config, history: History) -> Self {
        let os_context = OSContext::detect();
        let context_gatherer = ContextGatherer::new(&config);
        
        // Initialize provider manager with local-first architecture
        let mut provider_manager = ProviderManager::new();
        
        // Use the configured timeout for providers, with a reasonable minimum of 10s
        let timeout = std::time::Duration::from_millis(std::cmp::max(config.ai_timeout, 10000)); // Min 10 seconds
        
        // Always add local Ollama provider (offline functionality)
        let ollama_provider = OllamaProvider::with_timeout(
            config.ollama_url.clone(),
            config.model.clone(),
            timeout
        );
        provider_manager.add_provider(Box::new(ollama_provider));
        
        // Add cloud provider if configured
        if let Some(token) = &config.api_token {
            let cloud_provider = CloudProvider::new(config.backend_url.clone(), token.clone());
            provider_manager.add_provider(Box::new(cloud_provider));
            
            // If cloud is enabled, prioritize it
            if config.use_cloud {
                provider_manager.set_fallback_chain(vec![ProviderType::Cloud, ProviderType::Local]);
            } else {
                provider_manager.set_fallback_chain(vec![ProviderType::Local, ProviderType::Cloud]);
            }
        } else {
            provider_manager.set_fallback_chain(vec![ProviderType::Local]);
        }
        
        Self {
            config,
            history,
            validator: DefaultCommandValidator::new(),
            builtin_commands: BuiltinCommands::new(),
            os_context,
            context_gatherer,
            intent_classifier: IntentClassifier::new(),
            provider_manager,
        }
    }

    fn log_activity(&self, activity: &str) {
        if let Some(mut path) = dirs::config_dir() {
            path.push("cliai");
            path.push("activity.log");
            if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
                let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
                let _ = writeln!(file, "[{}] {}", timestamp, activity);
            }
        }
    }

    pub async fn process(&mut self, prompt: &str) -> Result<String> {
        self.log_activity(&format!("User Prompt: {}", prompt));
        
        // 1. Classify user intent FIRST (before any processing)
        let intent_analysis = self.intent_classifier.classify_intent(prompt);
        self.log_activity(&format!("Intent Analysis: {:?} (confidence: {:.2})", 
                                  intent_analysis.intent, intent_analysis.confidence));
        
        // 2. Handle ambiguous intent with clarification
        if intent_analysis.intent == UserIntent::Ambiguous {
            if let Some(clarification) = &intent_analysis.clarification_needed {
                self.log_activity("Intent ambiguous, requesting clarification");
                return Ok(format!("Command: (none)\n\n{}\n\n{}", clarification, intent_analysis.reasoning));
            }
        }
        
        // 3. Check for destructive actions and require explicit confirmation
        if self.intent_classifier.is_destructive_action(prompt) {
            self.log_activity("Destructive action detected");
            if intent_analysis.intent == UserIntent::Explanatory {
                return Ok(format!("Command: (none)\n\nI can explain how destructive operations work, but I won't suggest actual destructive commands for explanatory requests. Please clarify if you want to learn about these operations or actually perform them."));
            } else if intent_analysis.intent == UserIntent::Actionable {
                // For actionable destructive requests, we'll still generate the command but flag it as sensitive
                self.log_activity("Proceeding with destructive actionable request (will be flagged as sensitive)");
            }
        }
        
        // 4. Check for built-in commands (instant response without AI processing)
        if let Some(builtin_cmd) = self.builtin_commands.match_command(prompt) {
            // Start performance monitoring for built-in command
            let builtin_start = Instant::now();
            
            self.log_activity(&format!("Built-in Command Matched: {}", builtin_cmd.description));
            
            // Validate built-in command against intent
            let command = self.builtin_commands.generate_command(builtin_cmd, prompt);
            if let Err(validation_error) = self.intent_classifier.validate_command_for_intent(&command, &intent_analysis.intent) {
                self.log_activity(&format!("Built-in command validation failed: {}", validation_error));
                return Ok(format!("Command: (none)\n\n{}", validation_error));
            }
            
            // Log built-in command usage
            let command_id = self.find_builtin_command_id(builtin_cmd);
            self.builtin_commands.log_usage(&command_id, prompt);
            
            // Return in the standard "Command: " format
            let response = format!("Command: {}\n\n{}", command, builtin_cmd.description);
            
            // Record built-in command performance
            let builtin_duration = builtin_start.elapsed();
            self.provider_manager.get_performance_monitor_mut().record_measurement(
                OperationType::BuiltinCommand,
                builtin_duration,
                true
            );
            
            // Log performance if target was exceeded
            let target = OperationType::BuiltinCommand.get_target_duration(
                self.provider_manager.get_performance_monitor().get_targets()
            );
            if builtin_duration > target {
                // eprintln!("⚠️  Built-in command exceeded target: took {}ms (target: {}ms)", 
                //     builtin_duration.as_millis(), 
                //     target.as_millis());
            }
            
            self.log_activity(&format!("Built-in Response: {}", response.replace('\n', " ").chars().take(100).collect::<String>()));
            return Ok(response);
        }
        
        // 5. If no built-in match, proceed with AI processing
        self.log_activity("No built-in command match, proceeding with AI processing");
        
        // 6. Analyze Request (Intent + Context commands in one call)
        let analysis = self.analyze_request(prompt).await?;
        let category = analysis["category"].as_str().unwrap_or("GENERAL").to_uppercase();
        let commands = analysis["commands"].as_array();
        
        self.log_activity(&format!("Detected Category: {}", category));
        
        // 7. Gather Context using the new safe context gathering system
        let mut context_commands = Vec::new();
        if let Some(cmds) = commands {
            for cmd_val in cmds {
                if let Some(cmd) = cmd_val.as_str() {
                    // Map command suggestions to whitelisted context commands
                    let context_cmd = match cmd {
                        cmd if cmd.starts_with("ls") => Some("ls-current".to_string()),
                        cmd if cmd.starts_with("pwd") => Some("pwd".to_string()),
                        cmd if cmd.starts_with("whoami") => Some("whoami".to_string()),
                        cmd if cmd.starts_with("hostname") => Some("hostname".to_string()),
                        cmd if cmd.starts_with("uname") => Some("uname".to_string()),
                        cmd if cmd.starts_with("cat /etc/os-release") => Some("os-release".to_string()),
                        cmd if cmd.starts_with("git status") => Some("git-status".to_string()),
                        cmd if cmd.starts_with("git branch") => Some("git-branch".to_string()),
                        _ => None,
                    };
                    
                    if let Some(ctx_cmd) = context_cmd {
                        context_commands.push(ctx_cmd);
                    }
                }
            }
        }
        
        // Gather system context safely with performance monitoring
        let context_start = Instant::now();
        let system_context = self.context_gatherer.gather_context(&context_commands).await;
        let context_duration = context_start.elapsed();
        
        // Record context gathering performance
        self.provider_manager.get_performance_monitor_mut().record_measurement(
            OperationType::ContextGathering,
            context_duration,
            true
        );
        
        let context_str = self.context_gatherer.format_context_for_prompt(&system_context);
        
        if !context_str.is_empty() {
            self.log_activity(&format!("Gathered system context in {}ms", system_context.total_duration_ms));
            
            // Log performance if target was exceeded
            let target = OperationType::ContextGathering.get_target_duration(
                self.provider_manager.get_performance_monitor().get_targets()
            );
            if context_duration > target {
                // eprintln!("⚠️  Context gathering exceeded target: took {}ms (target: {}ms)", 
                //     context_duration.as_millis(), 
                //     target.as_millis());
            }
        }

        // 8. Select appropriate agent and context window
        let (agent, context_window) = match category.as_str() {
            "SHELL" => (&SHELL_EXPERT, ContextWindow::shell_expert()),
            "CODE" => (&CODE_EXPERT, ContextWindow::specialized_agent()),
            "LOG" => (&LOG_EXPERT, ContextWindow::specialized_agent()),
            _ => (&GENERAL_CLIAI, ContextWindow::general_agent()),
        };

        // 9. Build final prompt with appropriate context and intent information
        let final_prompt = self.build_agent_prompt_with_intent(agent, prompt, &context_window, &context_str, &intent_analysis);

        // 10. Generate Response with format validation for ShellExpert
        let response = if category == "SHELL" {
            let initial_response = self.call_ollama_with_prompt(&final_prompt).await?;
            let validated_response = self.validate_and_retry_shell_response(prompt, &initial_response, 2).await?;
            
            // Additional validation: check if generated command matches intent
            if let Some(command) = extract_command(&validated_response) {
                if let Err(validation_error) = self.intent_classifier.validate_command_for_intent(&command, &intent_analysis.intent) {
                    self.log_activity(&format!("Generated command validation failed: {}", validation_error));
                    return Ok(format!("Command: (none)\n\n{}\n\nOriginal request: {}", validation_error, prompt));
                }
            }
            
            validated_response
        } else {
            self.call_ollama_with_prompt(&final_prompt).await?
        };
        
        self.log_activity(&format!("AI Response: {}", response.replace('\n', " ").chars().take(100).collect::<String>()));
        self.log_activity(&format!("AI Response Length: {} chars", response.len()));
        Ok(response)
    }
    
    /// Build agent prompt with appropriate context window and system context
    fn build_agent_prompt(&self, agent: &AgentProfile, user_prompt: &str, context_window: &ContextWindow, system_context: &str) -> String {
        let mut full_prompt = agent.system_prompt.to_string();
        
        // Add OS context information to all agents
        full_prompt.push_str(&format!("\n\nSYSTEM CONTEXT:\n"));
        full_prompt.push_str(&format!("Operating System: {} ({})\n", self.os_context.version_info, self.os_context.architecture));
        full_prompt.push_str(&format!("Package Manager: {:?}\n", self.os_context.package_manager));
        full_prompt.push_str(&format!("Shell: {:?}\n", self.os_context.shell));
        full_prompt.push_str(&format!("Working Directory: {}\n", std::env::current_dir().unwrap_or_default().display()));
        
        // Add gathered system context if available
        if !system_context.is_empty() {
            full_prompt.push_str(&format!("\nCURRENT SYSTEM STATE:\n{}\n", system_context));
        }
        
        // Add conversation history based on context window
        if !self.history.is_empty() {
            let history_context = self.history.format_for_prompt(context_window);
            if !history_context.is_empty() {
                full_prompt.push_str(&format!("\n{}", history_context));
            }
        }
        
        full_prompt.push_str(&format!("\n\nUser: {}", user_prompt));
        full_prompt
    }
    
    /// Build agent prompt with intent analysis information
    fn build_agent_prompt_with_intent(&self, agent: &AgentProfile, user_prompt: &str, context_window: &ContextWindow, system_context: &str, intent_analysis: &IntentAnalysis) -> String {
        let mut full_prompt = agent.system_prompt.to_string();
        
        // Add intent analysis information for ShellExpert
        if agent.name == "ShellExpert" {
            full_prompt.push_str(&format!("\n\nINTENT ANALYSIS:\n"));
            full_prompt.push_str(&format!("User Intent: {:?}\n", intent_analysis.intent));
            full_prompt.push_str(&format!("Confidence: {:.2}\n", intent_analysis.confidence));
            full_prompt.push_str(&format!("Reasoning: {}\n", intent_analysis.reasoning));
            
            match intent_analysis.intent {
                UserIntent::Explanatory => {
                    full_prompt.push_str("\nIMPORTANT: This is an EXPLANATORY request. The user wants to learn, not execute commands.\n");
                    full_prompt.push_str("- Use 'Command: (none)' and provide educational explanation\n");
                    full_prompt.push_str("- If you must show a command example, make it clearly educational\n");
                    full_prompt.push_str("- Do NOT suggest destructive or system-modifying commands\n");
                    full_prompt.push_str("- Focus on teaching and explaining concepts\n");
                }
                UserIntent::Actionable => {
                    full_prompt.push_str("\nIMPORTANT: This is an ACTIONABLE request. The user wants to perform an action.\n");
                    full_prompt.push_str("- Provide the actual command to accomplish the task\n");
                    full_prompt.push_str("- Ensure the command is safe and appropriate\n");
                    full_prompt.push_str("- If the action is destructive, the system will flag it for confirmation\n");
                }
                UserIntent::Ambiguous => {
                    full_prompt.push_str("\nIMPORTANT: This request is AMBIGUOUS. Intent is unclear.\n");
                    full_prompt.push_str("- Prefer safe, informational commands\n");
                    full_prompt.push_str("- Avoid destructive operations\n");
                    full_prompt.push_str("- Consider asking for clarification if needed\n");
                }
            }
        }
        
        // Add OS context information to all agents
        full_prompt.push_str(&format!("\n\nSYSTEM CONTEXT:\n"));
        full_prompt.push_str(&format!("Operating System: {} ({})\n", self.os_context.version_info, self.os_context.architecture));
        full_prompt.push_str(&format!("Package Manager: {:?}\n", self.os_context.package_manager));
        full_prompt.push_str(&format!("Shell: {:?}\n", self.os_context.shell));
        full_prompt.push_str(&format!("Working Directory: {}\n", std::env::current_dir().unwrap_or_default().display()));
        
        // Add gathered system context if available
        if !system_context.is_empty() {
            full_prompt.push_str(&format!("\nCURRENT SYSTEM STATE:\n{}\n", system_context));
        }
        
        // Add conversation history based on context window
        if !self.history.is_empty() {
            let history_context = self.history.format_for_prompt(context_window);
            if !history_context.is_empty() {
                full_prompt.push_str(&format!("\n{}", history_context));
            }
        }
        
        full_prompt.push_str(&format!("\n\nUser: {}", user_prompt));
        full_prompt
    }
    
    /// Execute Ollama call with pre-built prompt
    async fn call_ollama_with_prompt(&mut self, full_prompt: &str) -> Result<String> {
        self.execute_ollama_call(full_prompt).await
    }
    
    /// Validate ShellExpert response format and retry if needed
    async fn validate_and_retry_shell_response(&mut self, original_prompt: &str, response: &str, max_retries: u32) -> Result<String> {
        // Check if response follows the required format
        if self.is_valid_shell_response(response) {
            return Ok(response.to_string());
        }
        
        // If format is invalid and we have retries left, try again
        if max_retries > 0 {
            self.log_activity(&format!("ShellExpert response format invalid, retrying. Attempts left: {}", max_retries));
            
            let retry_prompt = format!(
                "IMPORTANT: Your previous response did not follow the required format. 
                
Previous response: {}

REQUIRED FORMAT:
- Start with \"Command: \" followed by the command on the same line
- For multiple operations, use shell operators (&&, ||, |) in ONE command line
- For non-executable requests, use \"Command: (none)\" followed by explanation
- NEVER provide multiple separate commands

Original request: {}",
                response.trim(),
                original_prompt
            );
            
            // Build full prompt with context for retry
            let context_window = ContextWindow::shell_expert();
            let full_retry_prompt = self.build_agent_prompt(&SHELL_EXPERT, &retry_prompt, &context_window, "");
            
            let retry_response = self.call_ollama_with_prompt(&full_retry_prompt).await?;
            
            // Use Box::pin to handle async recursion
            return Box::pin(self.validate_and_retry_shell_response(original_prompt, &retry_response, max_retries - 1)).await;
        }
        
        // If we've exhausted retries, return the last response with a warning
        self.log_activity("ShellExpert format validation failed after all retries");
        Ok(response.to_string())
    }
    
    /// Check if a ShellExpert response follows the required format
    pub fn is_valid_shell_response(&self, response: &str) -> bool {
        let trimmed = response.trim();
        
        // Must start with "Command: "
        if !trimmed.starts_with("Command: ") {
            return false;
        }
        
        // Extract the command part
        let command_part = &trimmed[9..]; // Skip "Command: "
        let command_line = command_part.lines().next().unwrap_or("").trim();
        
        // Command line must not be empty
        if command_line.is_empty() {
            return false;
        }
        
        // Special case: "(none)" is always valid
        if command_line == "(none)" {
            return true;
        }
        
        // Check for multiple "Command: " lines (invalid)
        if trimmed.matches("Command: ").count() > 1 {
            return false;
        }
        
        // Validate that it's a single command line (may contain operators)
        self.is_valid_command_line(command_line)
    }
    
    /// Check if a command line is valid (single line with possible operators)
    pub fn is_valid_command_line(&self, command_line: &str) -> bool {
        // Should not contain newlines (single line requirement)
        if command_line.contains('\n') {
            return false;
        }
        
        // Should not be just operators
        let operators = ["&&", "||", "|", ";", ">", "<", ">>"];
        let trimmed = command_line.trim();
        if operators.iter().any(|op| trimmed == *op) {
            return false;
        }
        
        // Should contain at least one command word
        let words: Vec<&str> = command_line.split_whitespace().collect();
        if words.is_empty() {
            return false;
        }
        
        // First word should look like a command (not an operator)
        let first_word = words[0];
        if operators.iter().any(|op| first_word.contains(op)) {
            return false;
        }
        
        true
    }
    
    /// Check if any AI provider is available (for offline functionality verification)
    pub async fn is_any_provider_available(&self) -> bool {
        self.provider_manager.is_any_provider_available().await
    }
    
    /// Check if local provider (Ollama) is available
    pub async fn is_local_provider_available(&self) -> bool {
        if let Some(provider) = self.provider_manager.get_provider_by_type(&ProviderType::Local) {
            provider.is_available().await
        } else {
            false
        }
    }
    
    /// Try to list ONLY local models to debug availability issues
    pub async fn list_local_models(&self) -> Result<Vec<String>> {
        if let Some(provider) = self.provider_manager.get_provider_by_type(&ProviderType::Local) {
            provider.list_models().await
        } else {
            Err(anyhow!("Local provider not initialized"))
        }
    }

    /// Check if cloud provider is available
    pub async fn is_cloud_provider_available(&self) -> bool {
        if let Some(provider) = self.provider_manager.get_provider_by_type(&ProviderType::Cloud) {
            provider.is_available().await
        } else {
            false
        }
    }
    
    /// Get provider status for debugging
    pub fn get_provider_status(&self) -> Vec<(String, ProviderType, CircuitBreakerState)> {
        self.provider_manager.get_provider_status()
    }
    
    /// Validate a command using the command validator with performance monitoring
    pub fn validate_command(&mut self, command: &str) -> ValidationResult {
        let validation_start = Instant::now();
        let result = self.validator.validate(command);
        let validation_duration = validation_start.elapsed();
        
        // Record validation performance
        let success = matches!(result, ValidationResult::Valid(_) | ValidationResult::Rewritten(_, _));
        self.provider_manager.get_performance_monitor_mut().record_measurement(
            OperationType::CommandValidation,
            validation_duration,
            success
        );
        
        // Log performance if target was exceeded
        let target = OperationType::CommandValidation.get_target_duration(
            self.provider_manager.get_performance_monitor().get_targets()
        );
        /*
        if validation_duration > target {
            eprintln!("⚠️  Command validation exceeded target: took {}ms (target: {}ms)", 
                validation_duration.as_millis(), 
                target.as_millis());
        }
        */
        
        result
    }
    
    /// Find the ID of a built-in command for logging purposes
    fn find_builtin_command_id(&self, target_cmd: &crate::builtin_commands::BuiltinCommand) -> String {
        // This is a bit inefficient but works for the small number of built-in commands
        for (id, cmd) in &self.builtin_commands.commands {
            if std::ptr::eq(cmd, target_cmd) {
                return id.clone();
            }
        }
        // Fallback: generate ID from description
        target_cmd.description.to_lowercase().replace(' ', "_")
    }

    /// Get performance monitor for external access
    pub fn get_performance_monitor(&self) -> &PerformanceMonitor {
        self.provider_manager.get_performance_monitor()
    }

    /// Get performance summary
    pub fn get_performance_summary(&self) -> SystemPerformanceSummary {
        self.provider_manager.get_performance_summary()
    }

    /// Check if system is performing within acceptable limits
    pub fn is_system_healthy(&self) -> bool {
        self.provider_manager.is_system_healthy()
    }

    pub async fn list_models(&self) -> Result<Vec<String>> {
        // Use provider manager to list models from available providers
        self.provider_manager.list_models().await
    }

    async fn analyze_request(&mut self, prompt: &str) -> Result<serde_json::Value> {
        let response = self.call_ollama_no_history(&PLANNER_AGENT, prompt).await?;
        
        if let Ok(json) = serde_json::from_str(&response) {
            return Ok(json);
        }
        
        // Fallback for messy JSON
        if let (Some(start), Some(end)) = (response.find('{'), response.rfind('}')) {
            if let Ok(json) = serde_json::from_str(&response[start..=end]) {
                return Ok(json);
            }
        }
        
        Ok(json!({"category": "GENERAL", "commands": []}))
    }

    async fn call_ollama_no_history(&mut self, agent: &AgentProfile, prompt: &str) -> Result<String> {
        let context_window = ContextWindow { 
            max_turns: 0, 
            include_system_context: true, 
            prioritize_recent: false,
            include_working_directory: false,
            context_priority: ContextPriority::Balanced,
        };
        let full_prompt = self.build_agent_prompt(agent, prompt, &context_window, "");
        self.execute_ollama_call(&full_prompt).await
    }

    async fn execute_ollama_call(&mut self, full_prompt: &str) -> Result<String> {
        // Convert configured timeout (ms) to Duration
        let timeout = std::time::Duration::from_millis(self.config.ai_timeout);
        
        // Use provider manager for offline-first functionality
        match self.provider_manager.get_response(full_prompt, &GENERAL_CLIAI, timeout).await {
            Ok(response) => Ok(response),
            Err(_e) => {
                // Log the error for debugging
                self.log_activity(&format!("Provider error: {}", _e));
                
                // Check if any provider is available
                    return Err(anyhow!(
                        "Local AI provider unavailable. Please ensure:\n\
                        1. Ollama is running at {} (for local mode)\n\
                        2. The configured model is pulled",
                        self.config.ollama_url
                    ));
                
                Err(_e)
            }
        }
    }
}

pub fn extract_command(response: &str) -> Option<String> {
    // Method 1: Look for "Command: " prefix (most reliable and required format)
    if let Some(start) = response.find("Command: ") {
        let after_prefix = &response[start + 9..]; // Skip "Command: " (9 characters)
        let cmd_line = after_prefix.lines().next().unwrap_or("").trim();
        
        // Handle "(none)" case - return None to indicate no command to execute
        if cmd_line == "(none)" {
            return None;
        }
        
        // Clean up formatting but preserve the command structure
        let cleaned = cmd_line.trim();
        
        if !cleaned.is_empty() {
            return Some(cleaned.to_string());
        }
    }
    
    // Method 2: Look for "🚀 Executing:" pattern (from main.rs output) - legacy support
    if let Some(start) = response.rfind("🚀 Executing:") {
        let after_prefix = &response[start..];
        // Find the colon and skip past it and any whitespace
        if let Some(colon_pos) = after_prefix.find(':') {
            let after_colon = &after_prefix[colon_pos + 1..];
            let cmd_line = after_colon.lines().next().unwrap_or("").trim();
            let cleaned = cmd_line.trim_matches(|c| c == '`' || c == '*' || c == '"' || c == '\'').to_string();
            if !cleaned.is_empty() && cleaned != "(none)" {
                return Some(cleaned);
            }
        }
    }
    
    // Method 3: Look for single-line backtick commands - legacy support
    for line in response.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('`') && trimmed.ends_with('`') && trimmed.len() > 2 {
            let cmd = &trimmed[1..trimmed.len()-1];
            // Only return if it looks like a shell command and is not "(none)"
            if is_shell_command(cmd) && cmd != "(none)" {
                return Some(cmd.to_string());
            }
        }
    }
    
    // Method 4: Look for exactly one markdown code block - legacy support
    let parts: Vec<&str> = response.split("```").collect();
    if parts.len() == 3 {
        let block = parts[1];
        let actual_code = if let Some(newline_pos) = block.find('\n') {
            &block[newline_pos + 1..]
        } else {
            block
        }.trim();
        
        if (!actual_code.contains('\n') || actual_code.lines().count() < 3) && actual_code != "(none)" {
            return Some(actual_code.to_string());
        }
    }
    
    None
}

fn is_shell_command(cmd: &str) -> bool {
    let common_commands = [
        "ls", "find", "mkdir", "cat", "grep", "cp", "mv", "rm", "chmod", 
        "ps", "kill", "git", "ping", "curl", "wget", "df", "du", "free",
        "top", "htop", "whoami", "hostname", "uptime", "uname", "which",
        "echo", "touch", "head", "tail", "wc", "sort", "uniq", "awk", "sed"
    ];
    
    let first_word = cmd.split_whitespace().next().unwrap_or("");
    common_commands.contains(&first_word)
}
