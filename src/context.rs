use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use crate::config::Config;

/// Safe context gathering system with whitelisted commands
#[derive(Debug, Clone)]
pub struct ContextGatherer {
    /// Whitelisted commands that are safe to execute for context
    whitelisted_commands: HashMap<String, ContextCommand>,
    /// Timeout for context gathering operations
    timeout: Duration,
}

/// A whitelisted context command with metadata
#[derive(Debug, Clone)]
pub struct ContextCommand {
    pub command: String,
    pub description: String,
    pub category: ContextCategory,
    pub timeout_override: Option<Duration>,
}

/// Categories of context information
#[derive(Debug, Clone, PartialEq)]
pub enum ContextCategory {
    SystemInfo,
    Environment,
    FileSystem,
    Git,
}

/// Result of context gathering operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextResult {
    pub command: String,
    pub output: String,
    pub success: bool,
    pub duration_ms: u64,
    pub category: String,
}

/// Complete context information gathered from the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemContext {
    pub working_directory: String,
    pub results: Vec<ContextResult>,
    pub gathered_at: String,
    pub total_duration_ms: u64,
}

impl ContextGatherer {
    /// Create a new context gatherer with default whitelisted commands
    pub fn new(config: &Config) -> Self {
        let timeout = Duration::from_millis(config.context_timeout);
        let mut whitelisted_commands = HashMap::new();
        
        // System information commands
        whitelisted_commands.insert("uname".to_string(), ContextCommand {
            command: "uname -a".to_string(),
            description: "System information".to_string(),
            category: ContextCategory::SystemInfo,
            timeout_override: None,
        });
        
        whitelisted_commands.insert("os-release".to_string(), ContextCommand {
            command: "cat /etc/os-release".to_string(),
            description: "OS release information".to_string(),
            category: ContextCategory::SystemInfo,
            timeout_override: None,
        });
        
        // Environment commands
        whitelisted_commands.insert("pwd".to_string(), ContextCommand {
            command: "pwd".to_string(),
            description: "Current working directory".to_string(),
            category: ContextCategory::Environment,
            timeout_override: Some(Duration::from_millis(500)), // Very fast command
        });
        
        whitelisted_commands.insert("whoami".to_string(), ContextCommand {
            command: "whoami".to_string(),
            description: "Current user".to_string(),
            category: ContextCategory::Environment,
            timeout_override: Some(Duration::from_millis(500)),
        });
        
        whitelisted_commands.insert("hostname".to_string(), ContextCommand {
            command: "hostname".to_string(),
            description: "System hostname".to_string(),
            category: ContextCategory::Environment,
            timeout_override: Some(Duration::from_millis(500)),
        });
        
        // File system commands (read-only)
        whitelisted_commands.insert("ls-current".to_string(), ContextCommand {
            command: "ls -la".to_string(),
            description: "Current directory contents".to_string(),
            category: ContextCategory::FileSystem,
            timeout_override: Some(Duration::from_millis(1000)),
        });
        
        // Git commands (read-only)
        whitelisted_commands.insert("git-status".to_string(), ContextCommand {
            command: "git status --porcelain".to_string(),
            description: "Git repository status".to_string(),
            category: ContextCategory::Git,
            timeout_override: Some(Duration::from_millis(1500)),
        });
        
        whitelisted_commands.insert("git-branch".to_string(), ContextCommand {
            command: "git branch --show-current".to_string(),
            description: "Current git branch".to_string(),
            category: ContextCategory::Git,
            timeout_override: Some(Duration::from_millis(1000)),
        });
        
        Self {
            whitelisted_commands,
            timeout,
        }
    }
    
    /// Gather context information safely using whitelisted commands
    pub async fn gather_context(&self, requested_commands: &[String]) -> SystemContext {
        let start_time = Instant::now();
        let mut results = Vec::new();
        
        // Always include working directory
        let working_directory = std::env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| "unknown".to_string());
        
        // Execute requested commands if they are whitelisted
        for cmd_id in requested_commands {
            if let Some(context_cmd) = self.whitelisted_commands.get(cmd_id) {
                let result = self.execute_safe_command(context_cmd).await;
                results.push(result);
            } else {
                // Log attempt to use non-whitelisted command
                results.push(ContextResult {
                    command: cmd_id.clone(),
                    output: "Command not whitelisted for context gathering".to_string(),
                    success: false,
                    duration_ms: 0,
                    category: "blocked".to_string(),
                });
            }
        }
        
        // If no specific commands requested, gather basic context
        if requested_commands.is_empty() {
            let basic_commands = ["pwd", "whoami", "uname"];
            for cmd_id in &basic_commands {
                if let Some(context_cmd) = self.whitelisted_commands.get(*cmd_id) {
                    let result = self.execute_safe_command(context_cmd).await;
                    results.push(result);
                }
            }
        }
        
        let total_duration = start_time.elapsed();
        
        SystemContext {
            working_directory,
            results,
            gathered_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            total_duration_ms: total_duration.as_millis() as u64,
        }
    }
    
    /// Execute a whitelisted command safely with timeout
    async fn execute_safe_command(&self, context_cmd: &ContextCommand) -> ContextResult {
        let start_time = Instant::now();
        let timeout = context_cmd.timeout_override.unwrap_or(self.timeout);
        
        // Use tokio::time::timeout for async timeout handling
        let result = tokio::time::timeout(timeout, async {
            self.execute_command_sync(&context_cmd.command).await
        }).await;
        
        let duration = start_time.elapsed();
        
        match result {
            Ok(Ok(output)) => ContextResult {
                command: context_cmd.command.clone(),
                output,
                success: true,
                duration_ms: duration.as_millis() as u64,
                category: format!("{:?}", context_cmd.category),
            },
            Ok(Err(e)) => ContextResult {
                command: context_cmd.command.clone(),
                output: format!("Error: {}", e),
                success: false,
                duration_ms: duration.as_millis() as u64,
                category: format!("{:?}", context_cmd.category),
            },
            Err(_) => ContextResult {
                command: context_cmd.command.clone(),
                output: format!("Timeout after {}ms", timeout.as_millis()),
                success: false,
                duration_ms: timeout.as_millis() as u64,
                category: format!("{:?}", context_cmd.category),
            },
        }
    }
    
    /// Execute a command synchronously (wrapped in async for timeout handling)
    async fn execute_command_sync(&self, command: &str) -> Result<String> {
        let output = tokio::task::spawn_blocking({
            let command = command.to_string();
            move || {
                Command::new("sh")
                    .arg("-c")
                    .arg(&command)
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .output()
            }
        }).await??;
        
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow!("Command failed: {}", stderr))
        }
    }
    
    /// Get list of available whitelisted commands
    pub fn get_available_commands(&self) -> Vec<String> {
        self.whitelisted_commands.keys().cloned().collect()
    }
    
    /// Get command details by ID
    pub fn get_command_details(&self, command_id: &str) -> Option<&ContextCommand> {
        self.whitelisted_commands.get(command_id)
    }
    
    /// Format context for inclusion in AI prompts
    pub fn format_context_for_prompt(&self, context: &SystemContext) -> String {
        let mut formatted = String::new();
        
        formatted.push_str(&format!("Working Directory: {}\n", context.working_directory));
        
        for result in &context.results {
            if result.success && !result.output.is_empty() {
                formatted.push_str(&format!("$ {}\n{}\n\n", result.command, result.output));
            }
        }
        
        formatted.trim().to_string()
    }
    
    /// Update timeout configuration
    pub fn update_timeout(&mut self, new_timeout: Duration) {
        self.timeout = new_timeout;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, SafetyLevel};
    
    fn create_test_config() -> Config {
        Config {
            model: "test".to_string(),
            ollama_url: "http://localhost:11434".to_string(),
            prefix: None,
            auto_execute: false,
            dry_run: false,
            safety_level: SafetyLevel::Medium,
            context_timeout: 2000,
            ai_timeout: 30000,
            api_token: None,
            use_cloud: false,
            backend_url: "https://api.cliai.com".to_string(),
        }
    }
    
    #[test]
    fn test_context_gatherer_creation() {
        let config = create_test_config();
        let gatherer = ContextGatherer::new(&config);
        
        assert_eq!(gatherer.timeout, Duration::from_millis(2000));
        assert!(!gatherer.whitelisted_commands.is_empty());
    }
    
    #[test]
    fn test_whitelisted_commands_present() {
        let config = create_test_config();
        let gatherer = ContextGatherer::new(&config);
        
        // Check that essential commands are whitelisted
        assert!(gatherer.whitelisted_commands.contains_key("pwd"));
        assert!(gatherer.whitelisted_commands.contains_key("whoami"));
        assert!(gatherer.whitelisted_commands.contains_key("uname"));
        assert!(gatherer.whitelisted_commands.contains_key("os-release"));
    }
    
    #[test]
    fn test_get_available_commands() {
        let config = create_test_config();
        let gatherer = ContextGatherer::new(&config);
        
        let commands = gatherer.get_available_commands();
        assert!(!commands.is_empty());
        assert!(commands.contains(&"pwd".to_string()));
        assert!(commands.contains(&"whoami".to_string()));
    }
    
    #[test]
    fn test_get_command_details() {
        let config = create_test_config();
        let gatherer = ContextGatherer::new(&config);
        
        let pwd_details = gatherer.get_command_details("pwd");
        assert!(pwd_details.is_some());
        assert_eq!(pwd_details.unwrap().command, "pwd");
        assert_eq!(pwd_details.unwrap().category, ContextCategory::Environment);
        
        let invalid_details = gatherer.get_command_details("invalid-command");
        assert!(invalid_details.is_none());
    }
    
    #[test]
    fn test_context_command_categories() {
        let config = create_test_config();
        let gatherer = ContextGatherer::new(&config);
        
        let pwd_cmd = gatherer.get_command_details("pwd").unwrap();
        assert_eq!(pwd_cmd.category, ContextCategory::Environment);
        
        let uname_cmd = gatherer.get_command_details("uname").unwrap();
        assert_eq!(uname_cmd.category, ContextCategory::SystemInfo);
        
        let git_status_cmd = gatherer.get_command_details("git-status").unwrap();
        assert_eq!(git_status_cmd.category, ContextCategory::Git);
    }
    
    #[test]
    fn test_timeout_overrides() {
        let config = create_test_config();
        let gatherer = ContextGatherer::new(&config);
        
        let pwd_cmd = gatherer.get_command_details("pwd").unwrap();
        assert_eq!(pwd_cmd.timeout_override, Some(Duration::from_millis(500)));
        
        let uname_cmd = gatherer.get_command_details("uname").unwrap();
        assert_eq!(uname_cmd.timeout_override, None); // Uses default timeout
    }
    
    #[test]
    fn test_format_context_for_prompt() {
        let config = create_test_config();
        let gatherer = ContextGatherer::new(&config);
        
        let context = SystemContext {
            working_directory: "/home/user/project".to_string(),
            results: vec![
                ContextResult {
                    command: "pwd".to_string(),
                    output: "/home/user/project".to_string(),
                    success: true,
                    duration_ms: 10,
                    category: "Environment".to_string(),
                },
                ContextResult {
                    command: "whoami".to_string(),
                    output: "user".to_string(),
                    success: true,
                    duration_ms: 15,
                    category: "Environment".to_string(),
                },
            ],
            gathered_at: "2024-01-01 12:00:00".to_string(),
            total_duration_ms: 25,
        };
        
        let formatted = gatherer.format_context_for_prompt(&context);
        
        assert!(formatted.contains("Working Directory: /home/user/project"));
        assert!(formatted.contains("$ pwd"));
        assert!(formatted.contains("/home/user/project"));
        assert!(formatted.contains("$ whoami"));
        assert!(formatted.contains("user"));
    }
    
    #[test]
    fn test_update_timeout() {
        let config = create_test_config();
        let mut gatherer = ContextGatherer::new(&config);
        
        assert_eq!(gatherer.timeout, Duration::from_millis(2000));
        
        gatherer.update_timeout(Duration::from_millis(5000));
        assert_eq!(gatherer.timeout, Duration::from_millis(5000));
    }
    
    #[tokio::test]
    async fn test_gather_context_with_empty_requests() {
        let config = create_test_config();
        let gatherer = ContextGatherer::new(&config);
        
        let context = gatherer.gather_context(&[]).await;
        
        // Should gather basic context when no specific commands requested
        assert!(!context.working_directory.is_empty());
        assert!(!context.results.is_empty());
        assert!(context.total_duration_ms > 0);
    }
    
    #[tokio::test]
    async fn test_gather_context_with_non_whitelisted_command() {
        let config = create_test_config();
        let gatherer = ContextGatherer::new(&config);
        
        let context = gatherer.gather_context(&["rm".to_string()]).await;
        
        // Should have one result showing the command was blocked
        assert_eq!(context.results.len(), 1);
        assert!(!context.results[0].success);
        assert!(context.results[0].output.contains("not whitelisted"));
    }
}