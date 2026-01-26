use clap::{Parser, Subcommand};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::io::{self, Write};
use std::process::Command;
use std::env;
use chrono;

mod agents;
mod builtin_commands;
mod config;
mod context;
mod error_handling;
mod execution;
mod history;
mod intent;
mod logging;
mod os_context;
mod performance;
mod providers;
mod quoting;
mod test_suite;
mod validation;

use agents::Orchestrator;
use config::{Config, SafetyLevel};
use error_handling::{enhance_error, display_success, display_warning, display_info, display_config_change, display_interface_reminder, display_tip};
use execution::{ExecutionMode, ExecutableCommand};
use history::History;
use logging::{init_logger, get_logger};
use validation::{ValidationResult, ValidationError, SecurityWarning};
use providers::{ProviderType, CircuitBreakerState};
use performance::{OperationType, PerformanceStats};
use test_suite::{TestSuite, TestCategory};

/// Copy-paste safe command output structure
#[derive(Debug, Clone)]
pub struct CommandOutput {
    pub command: Option<String>,
    pub explanation: String,
    pub warnings: Vec<String>,
}

impl CommandOutput {
    /// Create a new CommandOutput with a command
    pub fn with_command(command: String, explanation: String) -> Self {
        Self {
            command: Some(command),
            explanation,
            warnings: Vec::new(),
        }
    }
    
    /// Create a new CommandOutput without a command (explanation only)
    pub fn explanation_only(explanation: String) -> Self {
        Self {
            command: None,
            explanation,
            warnings: Vec::new(),
        }
    }
    
    /// Add a warning to the output
    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }
    
    /// Format the output for copy-paste safe display
    /// CRITICAL: Commands and explanations must NEVER be mixed
    pub fn format_for_display(&self) -> String {
        let mut output = String::new();
        
        // Rule 1: Command first, if present, with NO formatting
        if let Some(cmd) = &self.command {
            // Clean command of any markdown formatting
            let clean_cmd = self.clean_command_formatting(cmd);
            output.push_str(&clean_cmd);
            output.push('\n');
        }
        
        // Rule 2: Clear separation between command and explanation
        if !self.explanation.is_empty() {
            if self.command.is_some() {
                output.push('\n'); // Extra newline for separation
            }
            output.push_str(&self.explanation);
            output.push('\n');
        }
        
        // Rule 3: Warnings at the end, clearly marked
        if !self.warnings.is_empty() {
            output.push('\n');
            for warning in &self.warnings {
                output.push_str(&format!("⚠️  {}\n", warning));
            }
        }
        
        output
    }
    
    /// Remove all markdown formatting from command text
    fn clean_command_formatting(&self, command: &str) -> String {
        let mut result = command.trim().to_string();
        
        // Remove backticks
        result = result.trim_matches('`').to_string();
        
        // Remove markdown code blocks
        result = result.replace("```bash", "").replace("```sh", "").replace("```", "");
        
        // Remove bold/italic markdown (but be careful not to remove shell wildcards)
        // Only remove ** and __ when they appear to be markdown formatting
        result = result.replace("**", "");
        result = result.replace("__", "");
        
        // Remove strikethrough
        result = result.replace("~~", "");
        
        // Only remove single asterisks/underscores if they appear to be markdown formatting
        // (i.e., at word boundaries, not in the middle of shell patterns)
        // For now, be conservative and don't remove single * or _ to preserve shell patterns
        
        result.trim().to_string()
    }
}

#[derive(Parser)]
#[command(name = "cliai")]
#[command(author = "CLIAI Team")]
#[command(version = "0.1.0")]
#[command(about = "🤖 CLIAI: Your intelligent CLI assistant", long_about = "A CLI tool that uses AI to help you with terminal commands and questions.")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// The prompt to send to CLIAI
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    prompt: Vec<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Show current configuration
    Config,
    /// List available Ollama models
    ListModels,
    /// Select the default model to use
    Select {
        /// Name of the model
        name: String,
    },
    /// Set a custom prefix/alias for CLIAI (e.g. 'jarvis')
    SetPrefix {
        /// The new prefix to use
        name: String,
    },
    /// Clear chat history
    Clear,
    /// Enable or disable auto-execution of commands
    AutoExecute {
        /// Mode: on, off, enable, or disable
        mode: String,
    },
    /// Enable or disable dry-run mode
    DryRun {
        /// Mode: on, off, enable, or disable
        mode: String,
    },
    /// Set safety level (low, medium, high)
    SafetyLevel {
        /// Safety level: low, medium, or high
        level: String,
    },
    /// Set context gathering timeout in milliseconds
    ContextTimeout {
        /// Timeout in milliseconds (1-30000)
        timeout: u64,
    },
    /// Set AI provider timeout in milliseconds
    AiTimeout {
        /// Timeout in milliseconds (10000-600000)
        timeout: u64,
    },
    /// Check AI provider status and availability
    ProviderStatus,
    /// Run the comprehensive test suite
    Test {
        /// Run only specific categories (comma-separated)
        #[arg(long)]
        categories: Option<String>,
        /// Save results to file
        #[arg(long)]
        save: Option<String>,
        /// Run only a subset of tests for quick validation
        #[arg(long)]
        quick: bool,
    },
    /// Enable debug logging mode (requires explicit consent)
    DebugMode {
        /// Enable debug mode
        #[arg(long)]
        enable: bool,
        /// Disable debug mode
        #[arg(long)]
        disable: bool,
    },
    /// Show current log file location and status
    LogStatus,
    /// Clear the error log file
    ClearLogs,
    /// Show performance monitoring status and statistics
    PerformanceStatus,
    /// Login to CLIAI for professional features
    Login,
    /// Toggle cloud mode (use remote high-performance models)
    Cloud {
        /// Mode: on, off, enable, or disable
        mode: String,
    },
    /// Set the backend URL for cliai-web
    SetBackend {
        /// The backend URL (e.g. http://localhost:5000)
        url: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize the privacy-preserving logger
    if let Err(e) = init_logger() {
        eprintln!("Warning: Failed to initialize logger: {}", e);
    }
    
    let app_config = Config::load();
    
    // Log application startup
    if let Ok(logger) = get_logger() {
        if let Ok(logger_guard) = logger.lock() {
            let os_info = format!("{} {}", std::env::consts::OS, std::env::consts::ARCH);
            let _ = logger_guard.log_startup("0.1.0", &os_info);
        }
    }
    
    // Check if we were called via a custom prefix
    let exe_path = env::current_exe().unwrap_or_default();
    let exe_name = exe_path.file_name().and_then(|n| n.to_str()).unwrap_or("cliai");
    
    let is_custom_prefix = app_config.prefix.as_deref() == Some(exe_name);

    if is_custom_prefix {
        // If called via custom prefix, skip subcommand parsing and treat everything as prompt
        let args: Vec<String> = env::args().skip(1).collect();
        if args.is_empty() {
            println!("{}", "🤖 CLIAI".bold().cyan());
            println!("Speak up! I'm listening. Try: {} {}", exe_name.green(), "what's the weather like?".dimmed());
            return Ok(());
        }
        return run_ai_prompt(args.join(" "), app_config).await;
    }

    // Normal CLIAI behavior
    let cli = Cli::parse();

    if let Some(command) = cli.command {
        match command {
            Commands::Config => {
                app_config.display();
                return Ok(());
            }
            Commands::ListModels => {
                let history = History::load();
                let orchestrator = Orchestrator::new(app_config.clone(), history);
                
                // Check local provider availability to provide better feedback
                let local_available = orchestrator.is_local_provider_available().await;

                match orchestrator.list_models().await {
                    Ok(models) => {
                        println!("{}", "Available Models:".bold().cyan());
                        
                        if !local_available {
                             println!("  {}", "⚠️  Local models hidden: Ollama not running or unreachable".yellow());
                             println!("  {}", format!("   (Expected at: {})", app_config.ollama_url).dimmed());
                        } else {
                            // If local is available, make sure we can actually list models
                            match orchestrator.list_local_models().await {
                                Ok(local_list) => {
                                    if local_list.is_empty() {
                                        println!("  {}", "⚠️  Ollama connected but no models found".yellow());
                                        println!("  {}", "   Try running: ollama pull mistral".dimmed());
                                    }
                                },
                                Err(e) => {
                                    println!("  {}", format!("⚠️  Failed to list local models: {}", e).yellow());
                                }
                            }
                        }

                        if models.is_empty() {
                             // Global fallback if everything is empty
                            println!("  {}", "No models found. Please check if Ollama is running and has models installed.".yellow());
                            println!("  {}", "Try: ollama pull mistral".dimmed());
                        } else {
                            for model in models {
                                if model == app_config.model {
                                    println!("* {} {}", model.green().bold(), "(active)".dimmed());
                                } else {
                                    println!("  {}", model);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        let enhanced_error = enhance_error(&e);
                        enhanced_error.display();
                        
                        // If error occurred but we suspect it's just local being down (and no cloud configured), give hint
                        if !local_available && app_config.api_token.is_none() {
                             println!("\n{}", "Tip: Ensure Ollama is running with 'ollama serve'".dimmed());
                        }
                    }
                }
                return Ok(());
            }
            Commands::Select { name } => {
                let mut new_config = app_config.clone();
                new_config.model = name.clone();
                match new_config.save() {
                    Ok(_) => display_success(&format!("Switched to model: {}", name.bold().yellow())),
                    Err(e) => {
                        let enhanced_error = enhance_error(&e);
                        enhanced_error.display();
                    }
                }
                return Ok(());
            }
            Commands::SetPrefix { name } => {
                let old_prefix = app_config.prefix.clone();
                let mut new_config = app_config.clone();
                new_config.prefix = Some(name.clone());
                
                match new_config.save() {
                    Ok(_) => {
                        println!("{} Prefix set to: {}", "✅".green(), name.bold().yellow());
                        
                        // Try to create a symlink in ~/.local/bin
                        if let Some(home) = dirs::home_dir() {
                            let local_bin = home.join(".local").join("bin");
                            let new_link = local_bin.join(&name);
                            
                            // Remove old link if it exists
                            if let Some(old) = old_prefix {
                                let old_link = local_bin.join(old);
                                if old_link.exists() {
                                    let _ = std::fs::remove_file(old_link);
                                }
                            }

                            if let Ok(current_exe) = env::current_exe() {
                                #[cfg(unix)]
                                {
                                    use std::os::unix::fs::symlink;
                                    let _ = std::fs::create_dir_all(&local_bin);
                                    if new_link.exists() {
                                        let _ = std::fs::remove_file(&new_link);
                                    }
                                    match symlink(current_exe, &new_link) {
                                        Ok(_) => println!("{} Shortcut created! You can now use: {}", "✨".cyan(), name.bold().green()),
                                        Err(e) => eprintln!("{} Could not create symlink: {}", "⚠️".yellow(), e),
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        let enhanced_error = enhance_error(&e);
                        enhanced_error.display();
                    }
                }
                return Ok(());
            }
            Commands::Clear => {
                let mut history = History::load();
                history.clear();
                println!("{} Chat history cleared.", "🧹".cyan());
                return Ok(());
            }
            Commands::AutoExecute { mode } => {
                let mut config = app_config.clone();
                let enabled = match mode.to_lowercase().as_str() {
                    "on" | "enable" | "true" | "yes" => true,
                    "off" | "disable" | "false" | "no" => false,
                    _ => {
                        eprintln!("{} Invalid mode. Use: on, off, enable, or disable", "❌".red());
                        return Ok(());
                    }
                };
                
                let old_value = config.auto_execute.to_string();
                match config.set_auto_execute(enabled) {
                    Ok(_) => {
                        display_config_change("auto_execute", &old_value, &enabled.to_string());
                        if enabled {
                            display_warning("Commands will now execute automatically for safe operations");
                            display_info("Sensitive commands will still require confirmation");
                        } else {
                            display_info("Commands will be displayed for manual execution");
                            display_tip("Use 'cliai auto-execute on' to re-enable automatic execution");
                        }
                    },
                    Err(e) => {
                        let enhanced_error = enhance_error(&e);
                        enhanced_error.display();
                    }
                }
                return Ok(());
            }
            Commands::DryRun { mode } => {
                let mut config = app_config.clone();
                let enabled = match mode.to_lowercase().as_str() {
                    "on" | "enable" | "true" | "yes" => true,
                    "off" | "disable" | "false" | "no" => false,
                    _ => {
                        eprintln!("{} Invalid mode. Use: on, off, enable, or disable", "❌".red());
                        return Ok(());
                    }
                };
                
                let old_value = config.dry_run.to_string();
                match config.set_dry_run(enabled) {
                    Ok(_) => {
                        display_config_change("dry_run", &old_value, &enabled.to_string());
                        if enabled {
                            display_info("Commands will be shown with 'DRY RUN:' prefix but never executed");
                            display_tip("Perfect for testing and validation without side effects");
                        } else {
                            display_info("Dry-run mode disabled - normal execution behavior restored");
                        }
                    },
                    Err(e) => {
                        let enhanced_error = enhance_error(&e);
                        enhanced_error.display();
                    }
                }
                return Ok(());
            }
            Commands::SafetyLevel { level } => {
                let mut config = app_config.clone();
                let old_level = format!("{:?}", config.safety_level);
                let safety_level = match level.to_lowercase().as_str() {
                    "low" => SafetyLevel::Low,
                    "medium" => SafetyLevel::Medium,
                    "high" => SafetyLevel::High,
                    _ => {
                        eprintln!("{} Invalid safety level. Use: low, medium, or high", "❌".red());
                        display_info("Current safety levels:");
                        display_info("  • low: Minimal safety checks, allows most commands");
                        display_info("  • medium: Balanced safety with confirmation for risky commands");
                        display_info("  • high: Maximum safety, blocks dangerous operations");
                        return Ok(());
                    }
                };
                
                match config.set_safety_level(safety_level.clone()) {
                    Ok(_) => {
                        display_config_change("safety_level", &old_level, &format!("{:?}", safety_level));
                        match safety_level {
                            SafetyLevel::Low => {
                                display_warning("Low safety mode: Minimal command validation");
                                display_info("Most commands will execute with basic checks only");
                            }
                            SafetyLevel::Medium => {
                                display_info("Medium safety mode: Balanced protection");
                                display_info("Risky commands will require confirmation");
                            }
                            SafetyLevel::High => {
                                display_success("High safety mode: Maximum protection");
                                display_info("Dangerous commands will be blocked entirely");
                            }
                        }
                    },
                    Err(e) => {
                        let enhanced_error = enhance_error(&e);
                        enhanced_error.display();
                    }
                }
                return Ok(());
            }
            Commands::ContextTimeout { timeout } => {
                let mut config = app_config.clone();
                let old_timeout = config.context_timeout.to_string();
                
                if timeout < 1000 || timeout > 60000 {
                    eprintln!("{} Timeout must be between 1000ms (1s) and 60000ms (60s)", "❌".red());
                    display_info(&format!("Current timeout: {}ms", config.context_timeout));
                    display_tip("Recommended values: 2000ms for fast, 5000ms for thorough, 30000ms for complex requests");
                    return Ok(());
                }
                
                match config.set_context_timeout(timeout) {
                    Ok(_) => {
                        display_config_change("context_timeout", &format!("{}ms", old_timeout), &format!("{}ms", timeout));
                        if timeout < 2000 {
                            display_warning("Short timeout may cause context gathering to fail");
                        } else if timeout > 10000 {
                            display_info("Long timeout provides thorough context but may slow responses");
                        } else {
                            display_success("Timeout set to optimal range for reliable context gathering");
                        }
                    },
                    Err(e) => {
                        let enhanced_error = enhance_error(&e);
                        enhanced_error.display();
                    }
                }
                return Ok(());
            }
            Commands::AiTimeout { timeout } => {
                let mut config = app_config.clone();
                let old_timeout = config.ai_timeout.to_string();
                
                if timeout < 10000 || timeout > 600000 {
                    eprintln!("{} AI timeout must be between 10000ms (10s) and 600000ms (10min)", "❌".red());
                    display_info(&format!("Current AI timeout: {}ms", config.ai_timeout));
                    display_tip("Recommended values: 30000ms for fast, 120000ms for normal, 300000ms for complex requests");
                    return Ok(());
                }
                
                config.ai_timeout = timeout;
                match config.save() {
                    Ok(_) => {
                        display_config_change("ai_timeout", &format!("{}ms", old_timeout), &format!("{}ms", timeout));
                        if timeout < 30000 {
                            display_warning("Short AI timeout may cause requests to fail for slower models");
                        } else if timeout >= 30000 && timeout <= 120000 {
                            display_success(&format!("AI timeout set to {}ms ({}s)", timeout, timeout / 1000));
                            display_info("The assistant will now be patient and wait for slow responses");
                        } else {
                            display_info("Long AI timeout provides maximum patience for complex requests");
                        }
                    },
                    Err(e) => {
                        let enhanced_error = enhance_error(&e);
                        enhanced_error.display();
                    }
                }
                return Ok(());
            }
            Commands::ProviderStatus => {
                let history = History::load();
                let orchestrator = Orchestrator::new(app_config.clone(), history);
                
                println!("{}", "🤖 AI Provider Status:".bold().cyan());
                
                // Check provider availability
                let local_available = orchestrator.is_local_provider_available().await;
                let cloud_available = orchestrator.is_cloud_provider_available().await;
                let any_available = orchestrator.is_any_provider_available().await;
                
                println!("Local Provider (Ollama): {}", 
                    if local_available { "✅ Available".green() } else { "❌ Unavailable".red() });
                println!("Overall Status: {}", 
                    if local_available { "✅ Ready".green() } else { "❌ Local provider unavailable".red() });
                
                // Show detailed provider status
                let status = orchestrator.get_provider_status();
                if !status.is_empty() {
                    println!("\n{}", "Provider Details:".bold());
                    for (name, provider_type, circuit_state) in status {
                        let type_str = match provider_type {
                            ProviderType::Local => "Local",
                            ProviderType::Cloud => "Cloud",
                        };
                        let state_str = match circuit_state {
                            CircuitBreakerState::Closed => "✅ Normal".green(),
                            CircuitBreakerState::Open => "❌ Failed".red(),
                            CircuitBreakerState::HalfOpen => "⚠️  Testing".yellow(),
                        };
                        println!("  {} ({}): {}", name, type_str, state_str);
                    }
                }
                
                // Offline functionality check
                println!("\n{}", "Offline Functionality:".bold());
                if local_available {
                    println!("✅ Core command generation works without internet");
                    println!("✅ Local mode never requires internet connectivity");
                } else {
                    println!("❌ Local provider unavailable - offline functionality limited");
                    println!("   Please ensure Ollama is running at {}", app_config.ollama_url);
                }
                
                return Ok(());
            }
            Commands::Test { categories, save, quick } => {
                let test_suite = TestSuite::new();
                
                println!("{}", "🧪 CLIAI Test Suite".bold().cyan());
                
                if quick {
                    println!("Running quick validation tests...\n");
                    // Run a subset of tests for quick validation
                    let quick_categories = vec![
                        TestCategory::FileManagement,
                        TestCategory::SystemInfo,
                    ];
                    let results = test_suite.run_category_tests(app_config.clone(), quick_categories).await?;
                    
                    if let Some(filename) = save {
                        test_suite.save_test_results(&results, &filename)?;
                    }
                } else if let Some(cat_str) = categories {
                    // Parse categories
                    let category_names: Vec<&str> = cat_str.split(',').map(|s| s.trim()).collect();
                    let mut parsed_categories = Vec::new();
                    
                    for cat_name in category_names {
                        match cat_name.to_lowercase().as_str() {
                            "file-management" => parsed_categories.push(TestCategory::FileManagement),
                            "system-info" => parsed_categories.push(TestCategory::SystemInfo),
                            "git-operations" => parsed_categories.push(TestCategory::GitOperations),
                            "network" => parsed_categories.push(TestCategory::Network),
                            "programming" => parsed_categories.push(TestCategory::Programming),
                            "process-management" => parsed_categories.push(TestCategory::ProcessManagement),
                            "general" => parsed_categories.push(TestCategory::General),
                            _ => {
                                eprintln!("{} Unknown category: {}", "❌".red(), cat_name);
                                println!("Available categories: file-management, system-info, git-operations, network, programming, process-management, general");
                                return Ok(());
                            }
                        }
                    }
                    
                    let results = test_suite.run_category_tests(app_config.clone(), parsed_categories).await?;
                    
                    if let Some(filename) = save {
                        test_suite.save_test_results(&results, &filename)?;
                    }
                } else {
                    println!("Running complete test suite (50 questions)...\n");
                    display_warning("This will take several minutes and make many AI requests");
                    
                    let results = test_suite.run_complete_test_suite(app_config.clone()).await?;
                    
                    if let Some(filename) = save {
                        test_suite.save_test_results(&results, &filename)?;
                    } else {
                        // Save to default location
                        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
                        let default_filename = format!("cliai_test_results_{}.md", timestamp);
                        test_suite.save_test_results(&results, &default_filename)?;
                    }
                }
                
                return Ok(());
            }
            Commands::DebugMode { enable, disable } => {
                if enable && disable {
                    eprintln!("{} Cannot enable and disable debug mode at the same time", "❌".red());
                    return Ok(());
                }
                if !enable && !disable {
                    eprintln!("{} Must specify either --enable or --disable", "❌".red());
                    return Ok(());
                }
                
                if let Ok(logger) = get_logger() {
                    if let Ok(mut logger_guard) = logger.lock() {
                        if enable {
                            println!("{} {}", "⚠️".yellow(), "Debug mode will log detailed information that may include sensitive data.".yellow());
                            println!("{} {}", "🔒".cyan(), "This information is stored locally in ~/.config/cliai/error.log".dimmed());
                            print!("{} ", "Do you consent to debug logging? (y/n):".bold());
                            io::stdout().flush()?;

                            let mut input = String::new();
                            io::stdin().read_line(&mut input)?;

                            let consent = input.trim().to_lowercase() == "y";
                            
                            match logger_guard.enable_debug_mode(consent) {
                                Ok(_) => {
                                    if consent {
                                        display_success("Debug mode enabled with user consent");
                                        display_warning("Detailed logging is now active - may include sensitive information");
                                        display_info("Debug logs are clearly marked with 🐛 DEBUG prefix");
                                        display_tip("Use 'cliai debug-mode --disable' to turn off debug logging");
                                    } else {
                                        display_info("Debug mode activation cancelled - user consent not given");
                                    }
                                },
                                Err(e) => {
                                    let enhanced_error = enhance_error(&e);
                                    enhanced_error.display();
                                }
                            }
                        } else {
                            match logger_guard.disable_debug_mode() {
                                Ok(_) => {
                                    display_success("Debug mode disabled");
                                    display_info("Detailed logging is now turned off");
                                },
                                Err(e) => {
                                    let enhanced_error = enhance_error(&e);
                                    enhanced_error.display();
                                }
                            }
                        }
                    } else {
                        eprintln!("{} Failed to access logger", "❌".red());
                    }
                } else {
                    eprintln!("{} Logger not initialized", "❌".red());
                }
                return Ok(());
            }
            Commands::LogStatus => {
                if let Ok(logger) = get_logger() {
                    if let Ok(logger_guard) = logger.lock() {
                        println!("{}", "📋 CLIAI Logging Status:".bold().cyan());
                        println!("Log file: {}", logger_guard.get_current_log_path().display().to_string().green());
                        println!("Debug mode: {}", 
                            if logger_guard.is_debug_mode() { 
                                "enabled 🐛".yellow() 
                            } else { 
                                "disabled 🛡️".green() 
                            }
                        );
                        
                        // Check if log file exists and show size
                        let log_path = logger_guard.get_current_log_path();
                        if log_path.exists() {
                            if let Ok(metadata) = std::fs::metadata(log_path) {
                                let size_kb = metadata.len() / 1024;
                                println!("Log file size: {} KB", size_kb);
                            }
                        } else {
                            println!("Log file: not created yet");
                        }
                        
                        println!();
                        display_info("Privacy protection: Commands and prompts are never logged in production mode");
                        display_info("Only system errors, performance metrics, and configuration changes are recorded");
                        if logger_guard.is_debug_mode() {
                            display_warning("Debug mode is active - detailed information may be logged");
                        } else {
                            display_tip("Use 'cliai debug-mode --enable' for detailed troubleshooting logs");
                        }
                    }
                } else {
                    eprintln!("{} Logger not initialized", "❌".red());
                }
                return Ok(());
            }
            Commands::ClearLogs => {
                if let Ok(logger) = get_logger() {
                    if let Ok(logger_guard) = logger.lock() {
                        print!("{} ", "Are you sure you want to clear all logs? (y/n):".bold());
                        io::stdout().flush()?;

                        let mut input = String::new();
                        io::stdin().read_line(&mut input)?;

                        if input.trim().to_lowercase() == "y" {
                            match logger_guard.clear_logs() {
                                Ok(_) => {
                                    display_success("Log file cleared successfully");
                                    display_info("A new log entry has been created to record this action");
                                },
                                Err(e) => {
                                    let enhanced_error = enhance_error(&e);
                                    enhanced_error.display();
                                }
                            }
                        } else {
                            display_info("Log clearing cancelled");
                        }
                    }
                } else {
                    eprintln!("{} Logger not initialized", "❌".red());
                }
                return Ok(());
            }
            Commands::PerformanceStatus => {
                let history = History::load();
                let orchestrator = Orchestrator::new(app_config.clone(), history);
                
                println!("{}", "📊 Performance Status:".bold().cyan());
                
                let summary = orchestrator.get_performance_summary();
                let is_healthy = orchestrator.is_system_healthy();
                
                println!("System Health: {}", 
                    if is_healthy { "✅ Healthy".green() } else { "⚠️  Degraded".yellow() });
                println!("Total Operations: {}", summary.total_operations);
                println!("Success Rate: {:.1}%", summary.overall_success_rate * 100.0);
                
                if summary.total_operations > 0 {
                    println!("\n{}", "Operation Performance:".bold());
                    
                    let operation_types = [
                        (OperationType::BuiltinCommand, "Built-in Commands"),
                        (OperationType::LocalOllama, "Local Ollama"),
                        (OperationType::TotalSystem, "Total System"),
                        (OperationType::ContextGathering, "Context Gathering"),
                        (OperationType::CommandValidation, "Command Validation"),
                        (OperationType::IntentClassification, "Intent Classification"),
                    ];
                    
                    for (op_type, display_name) in &operation_types {
                        if let Some(stats) = summary.stats.get(op_type) {
                            if stats.total_operations > 0 {
                                let compliance_color = if stats.target_compliance_rate >= 0.9 {
                                    "green"
                                } else if stats.target_compliance_rate >= 0.7 {
                                    "yellow"
                                } else {
                                    "red"
                                };
                                
                                println!("  {}: {} ops, avg {}, compliance {:.1}%", 
                                    display_name,
                                    stats.total_operations,
                                    PerformanceStats::format_duration(stats.avg_duration),
                                    match compliance_color {
                                        "green" => format!("{:.1}%", stats.target_compliance_rate * 100.0).green(),
                                        "yellow" => format!("{:.1}%", stats.target_compliance_rate * 100.0).yellow(),
                                        "red" => format!("{:.1}%", stats.target_compliance_rate * 100.0).red(),
                                        _ => format!("{:.1}%", stats.target_compliance_rate * 100.0).normal(),
                                    }
                                );
                            }
                        }
                    }
                    
                    println!("\n{}", "Performance Targets:".bold());
                    let targets = orchestrator.get_performance_monitor().get_targets();
                    println!("  Built-in Commands: <{}ms", targets.builtin_command.as_millis());
                    println!("  Local Ollama: <{}s", targets.local_ollama.as_secs());
                    println!("  Total System: <{}s", targets.total_system.as_secs());
                } else {
                    println!("\n{}", "No performance data available yet.".dimmed());
                    println!("{}", "Run some commands to see performance statistics.".dimmed());
                }
                
                return Ok(());
            }
            Commands::Login => {
                println!("{}", "🔐 CLIAI Device Login".bold().cyan());
                
                let client = reqwest::Client::new();
                let backend_url = app_config.backend_url.clone();
                let start_url = format!("{}/v1/auth/device/start", backend_url);

                match client.post(&start_url).send().await {
                    Ok(resp) => {
                        if let Ok(data) = resp.json::<serde_json::Value>().await {
                            if let Some(error_msg) = data["error"].as_str() {
                                eprintln!("{} Device login failed: {}", "❌".red(), error_msg.red());
                                if error_msg.contains("Failed to generate") {
                                    eprintln!("{} Hint: The backend database might be missing required tables.", "💡".yellow());
                                }
                                return Ok(());
                            }

                            let device_code = data["device_code"].as_str().unwrap_or_default();
                            
                            if device_code.is_empty() {
                                eprintln!("{} Received empty device code from server.", "❌".red());
                                return Ok(());
                            }
                            
                            let verification_url = data["verification_url"].as_str().unwrap_or("http://localhost:3000/activate");

                            let full_verification_url = format!("{}?code={}", verification_url, device_code);

                            println!("\nAttempting to open browser...");
                            if webbrowser::open(&full_verification_url).is_err() {
                                println!("1. Visit: {}", full_verification_url.bold().underline().blue());
                            } else {
                                println!("✅ Browser opened. Please complete login in the opened window.");
                            }
                            
                            println!("Security code: {} {}", device_code.bold().yellow(), "(Verified automatically)".dimmed());
                            println!("\nWaiting for activation...");

                            // Polling for token
                            let poll_url = format!("{}/v1/auth/device/token", backend_url);
                            let pb = ProgressBar::new_spinner();
                            pb.set_style(ProgressStyle::default_spinner()
                                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
                                .template("{spinner:.green} {msg}")?);
                            pb.set_message("Waiting for verification...");
                            pb.enable_steady_tick(std::time::Duration::from_millis(100));

                            let mut token = None;
                            for _ in 0..60 { // Poll for up to 5 minutes (every 5 seconds)
                                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                                let poll_resp = client.post(&poll_url)
                                    .json(&serde_json::json!({ "device_code": device_code }))
                                    .send().await;

                                if let Ok(r) = poll_resp {
                                    if r.status().is_success() {
                                        if let Ok(token_data) = r.json::<serde_json::Value>().await {
                                            token = token_data["access_token"].as_str().map(|s| s.to_string());
                                            break;
                                        }
                                    }
                                }
                            }

                            pb.finish_and_clear();

                            if let Some(t) = token {
                                let mut new_config = app_config.clone();
                                new_config.api_token = Some(t);
                                new_config.use_cloud = true;
                                new_config.save()?;
                                println!("{} Login successful! Cloud mode enabled.", "✅".green());
                                println!("Try asking: {}", "cliai explain this directory".dimmed());
                            } else {
                                eprintln!("{} Login timed out or failed.", "❌".red());
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("{} Failed to connect to backend: {}", "❌".red(), e);
                    }
                }
                
                return Ok(());
            }
            Commands::Cloud { mode } => {
                let mut config = app_config.clone();
                let enabled = match mode.to_lowercase().as_str() {
                    "on" | "enable" | "true" | "yes" => true,
                    "off" | "disable" | "false" | "no" => false,
                    _ => {
                        eprintln!("{} Invalid mode. Use: on, off, enable, or disable", "❌".red());
                        return Ok(());
                    }
                };

                if enabled && config.api_token.is_none() {
                    println!("{} You must login first to use cloud mode.", "⚠️".yellow());
                    println!("Try: {}", "cliai login".bold());
                    return Ok(());
                }

                config.use_cloud = enabled;
                config.save()?;
                display_success(&format!("Cloud mode {}", if enabled { "enabled ☁️" } else { "disabled 🏠" }));
                return Ok(());
            }
            Commands::SetBackend { url } => {
                let mut config = app_config.clone();
                config.backend_url = url.clone();
                config.save()?;
                display_success(&format!("Backend URL set to: {}", url.bold().yellow()));
                return Ok(());
            }
        }
    }

    let prompt = cli.prompt.join(" ");

    if prompt.is_empty() {
        println!("{}", "🤖 CLIAI".bold().cyan());
        println!("Ask me anything: {} {}", "cliai".green(), "how do I list files?".dimmed());
        println!("Or run {} for model options", "cliai --help".yellow());
        println!();
        display_interface_reminder();
        return Ok(());
    }

    run_ai_prompt(prompt, app_config).await
}

async fn execute_command_with_confirmation(cmd: &str, execution_mode: &ExecutionMode) -> anyhow::Result<()> {
    match execution_mode {
        ExecutionMode::Safe => {
            println!("\n{} {}", "🚀 Executing:".bold().green(), cmd.green());
            execute_shell_command(cmd).await
        }
        ExecutionMode::RequiresConfirmation(reasons) => {
            println!("\n{} {}", "⚠️  Sensitive command:".bold().yellow(), cmd.red());
            
            // Show confirmation reasons
            for reason in reasons {
                println!("   • {}", reason.yellow());
            }
            
            print!("{} ", "Run this command? (y/n):".bold());
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            if input.trim().to_lowercase() != "y" {
                println!("{}", "Aborted.".dimmed());
                return Ok(());
            }
            
            println!("\n{} {}", "🚀 Executing:".bold().green(), cmd.green());
            execute_shell_command(cmd).await
        }
        ExecutionMode::SuggestOnly => {
            println!("\n{} To execute this command, copy and paste it into your terminal:", "💡".cyan());
            println!("{}", cmd.green());
            Ok(())
        }
        ExecutionMode::DryRunOnly => {
            println!("\n{} {}", "🔍 DRY RUN:".bold().blue(), cmd.blue());
            println!("{}", "Command shown for preview only (dry-run mode enabled)".dimmed());
            Ok(())
        }
        ExecutionMode::Blocked(reason) => {
            println!("\n{} {}", "🚫 Command blocked:".bold().red(), reason.red());
            println!("{} {}", "Original command:".dimmed(), cmd.dimmed());
            Ok(())
        }
    }
}

async fn execute_shell_command(cmd: &str) -> anyhow::Result<()> {
    let status = if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/C", cmd]).status()?
    } else {
        Command::new("sh").args(["-c", cmd]).status()?
    };

    if !status.success() {
        eprintln!("\n{}", "Command failed.".red());
    }
    
    Ok(())
}

async fn run_ai_prompt(prompt: String, app_config: config::Config) -> anyhow::Result<()> {
    let mut history = History::load();
    
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈")
            .template("{spinner:.cyan} {msg}")?,
    );
    pb.set_message("Thinking...");
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    let mut orchestrator = Orchestrator::new(app_config.clone(), history.clone());

    match orchestrator.process(&prompt).await {
        Ok(response) => {
            pb.finish_and_clear();
            
            // Parse the response into a CommandOutput for copy-paste safe formatting
            let command_output = parse_response_to_command_output(&response);
            
            // Display the formatted output
            display_command_output(&command_output);
            
            history.add_turn("user", &prompt);
            history.add_turn("assistant", &response);
            let _ = history.save();

            // Handle command execution if a command is present
            if let Some(cmd) = &command_output.command {
                // Validate the command using the integrated validator
                let validation_result = orchestrator.validate_command(cmd);
                
                // Determine execution mode based on config and validation
                let execution_mode = ExecutionMode::determine(&app_config, &validation_result);
                
                // Create executable command with mode information
                let mut executable_cmd = ExecutableCommand::new(
                    cmd.clone(),
                    command_output.explanation.clone(),
                    execution_mode.clone()
                );
                
                // Add any warnings from the command output
                for warning in &command_output.warnings {
                    executable_cmd.add_warning(warning.clone());
                }
                
                // Handle different validation results with integrated safety checking
                match validation_result {
                    ValidationResult::Valid(validated_cmd) => {
                        executable_cmd.command = validated_cmd.clone();
                        if execution_mode.can_execute() {
                            execute_command_with_confirmation(&validated_cmd, &execution_mode).await?;
                        } else {
                            // Show execution instructions for non-executable modes
                            if let Some(instructions) = executable_cmd.get_execution_instructions() {
                                println!("\n{} {}", "💡".cyan(), instructions.dimmed());
                            }
                        }
                    }
                    ValidationResult::Rewritten(rewritten_cmd, fixes) => {
                        println!("\n{} Command was automatically fixed:", "🔧".yellow());
                        for fix in &fixes {
                            println!("  • {}", fix.dimmed());
                        }
                        executable_cmd.command = rewritten_cmd.clone();
                        if execution_mode.can_execute() {
                            execute_command_with_confirmation(&rewritten_cmd, &execution_mode).await?;
                        } else {
                            if let Some(instructions) = executable_cmd.get_execution_instructions() {
                                println!("\n{} {}", "💡".cyan(), instructions.dimmed());
                            }
                        }
                    }
                    ValidationResult::Invalid(invalid_cmd, errors) => {
                        println!("\n{} Command validation failed:", "❌".red());
                        for error in &errors {
                            match error {
                                ValidationError::HallucinatedFlag(flag) => {
                                    println!("  • Unknown flag: {}", flag.red());
                                }
                                ValidationError::PlaceholderDetected(placeholder) => {
                                    println!("  • Placeholder detected: {}", placeholder.red());
                                    println!("    Please provide specific values instead of placeholders.");
                                }
                                ValidationError::SyntaxError(msg) => {
                                    println!("  • Syntax error: {}", msg.red());
                                }
                                ValidationError::QuotingIssue(msg) => {
                                    println!("  • Quoting issue: {}", msg.red());
                                }
                            }
                        }
                        println!("\n{} {}", "Original command:".dimmed(), invalid_cmd.dimmed());
                        if let Some(reason) = execution_mode.get_block_reason() {
                            println!("{} {}", "🚫".red(), reason.red());
                        }
                    }
                    ValidationResult::Sensitive(sensitive_cmd, warnings) => {
                        println!("\n{} Sensitive command detected:", "⚠️".yellow());
                        for warning in &warnings {
                            match warning {
                                SecurityWarning::DataLoss(msg) => {
                                    println!("  • {}: {}", "Data Loss Risk".red(), msg);
                                }
                                SecurityWarning::SystemModification(msg) => {
                                    println!("  • {}: {}", "System Modification".yellow(), msg);
                                }
                                SecurityWarning::DangerousPattern(msg) => {
                                    println!("  • {}: {}", "Dangerous Pattern".red(), msg);
                                }
                            }
                        }
                        executable_cmd.command = sensitive_cmd.clone();
                        if execution_mode.can_execute() {
                            execute_command_with_confirmation(&sensitive_cmd, &execution_mode).await?;
                        } else {
                            if let Some(reason) = execution_mode.get_block_reason() {
                                println!("\n{} {}", "🚫".red(), reason.red());
                            }
                        }
                    }
                }
            }
        }
        Err(e) => {
            pb.finish_and_clear();
            let enhanced_error = enhance_error(&e);
            enhanced_error.display();
        }
    }
    Ok(())
}

/// Parse AI response into a CommandOutput struct for copy-paste safe formatting
fn parse_response_to_command_output(response: &str) -> CommandOutput {
    // Extract command using existing logic
    let command = agents::extract_command(response);
    
    // Extract explanation by removing the command part
    let explanation = if let Some(cmd_start) = response.find("Command: ") {
        let after_command = &response[cmd_start..];
        if let Some(newline_pos) = after_command.find('\n') {
            // Everything after the first newline following "Command: " is explanation
            after_command[newline_pos + 1..].trim().to_string()
        } else {
            // No explanation if no newline after command
            String::new()
        }
    } else {
        // If no "Command: " prefix, treat entire response as explanation
        response.trim().to_string()
    };
    
    if let Some(cmd) = command {
        CommandOutput::with_command(cmd, explanation)
    } else {
        CommandOutput::explanation_only(explanation)
    }
}

/// Display command output in copy-paste safe format
fn display_command_output(output: &CommandOutput) {
    if let Some(cmd) = &output.command {
        // Display command in copy-paste safe format (no decorations)
        println!("{}", cmd);
        
        // Add explanation if present, clearly separated
        if !output.explanation.is_empty() {
            println!();
            println!("{} {}", "💡".cyan(), output.explanation.dimmed());
        }
    } else {
        // No command, just explanation
        println!("{} {}", "🤖 AI:".bold().cyan(), output.explanation);
    }
    
    // Display warnings if any
    for warning in &output.warnings {
        println!("{} {}", "⚠️".yellow(), warning.yellow());
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_command_output_with_command() {
        let output = CommandOutput::with_command(
            "ls -la".to_string(),
            "Lists all files including hidden ones".to_string()
        );
        
        assert_eq!(output.command, Some("ls -la".to_string()));
        assert_eq!(output.explanation, "Lists all files including hidden ones");
        assert!(output.warnings.is_empty());
    }
    
    #[test]
    fn test_command_output_explanation_only() {
        let output = CommandOutput::explanation_only(
            "This is just an explanation".to_string()
        );
        
        assert_eq!(output.command, None);
        assert_eq!(output.explanation, "This is just an explanation");
        assert!(output.warnings.is_empty());
    }
    
    #[test]
    fn test_command_output_format_with_command() {
        let output = CommandOutput::with_command(
            "ls -la".to_string(),
            "Lists all files including hidden ones".to_string()
        );
        
        let formatted = output.format_for_display();
        
        // Command should be first, clean, no formatting
        assert!(formatted.starts_with("ls -la\n"));
        // Should have clear separation
        assert!(formatted.contains("\n\nLists all files including hidden ones"));
    }
    
    #[test]
    fn test_command_output_format_explanation_only() {
        let output = CommandOutput::explanation_only(
            "This is just an explanation".to_string()
        );
        
        let formatted = output.format_for_display();
        
        // Should only contain explanation
        assert_eq!(formatted.trim(), "This is just an explanation");
    }
    
    #[test]
    fn test_clean_command_formatting() {
        let output = CommandOutput::with_command(
            "`ls -la`".to_string(),
            "".to_string()
        );
        
        let cleaned = output.clean_command_formatting("`ls -la`");
        assert_eq!(cleaned, "ls -la");
        
        let cleaned = output.clean_command_formatting("**ls -la**");
        assert_eq!(cleaned, "ls -la");
        
        let cleaned = output.clean_command_formatting("```ls -la```");
        assert_eq!(cleaned, "ls -la");
        
        let cleaned = output.clean_command_formatting("__pwd__");
        assert_eq!(cleaned, "pwd");
        
        // Shell patterns should be preserved
        let cleaned = output.clean_command_formatting("find . -name '*.rs'");
        assert_eq!(cleaned, "find . -name '*.rs'");
    }
    
    #[test]
    fn test_command_output_with_warnings() {
        let mut output = CommandOutput::with_command(
            "rm -rf /tmp/test".to_string(),
            "Removes the test directory".to_string()
        );
        
        output.add_warning("This command will permanently delete files".to_string());
        
        let formatted = output.format_for_display();
        
        // Should contain the warning
        assert!(formatted.contains("⚠️  This command will permanently delete files"));
    }
    
    #[test]
    fn test_parse_response_with_command() {
        let response = "Command: ls -la\nThis lists all files including hidden ones.";
        let output = parse_response_to_command_output(response);
        
        assert_eq!(output.command, Some("ls -la".to_string()));
        assert_eq!(output.explanation, "This lists all files including hidden ones.");
    }
    
    #[test]
    fn test_parse_response_with_none_command() {
        let response = "Command: (none)\nThis is just an explanation without a command.";
        let output = parse_response_to_command_output(response);
        
        assert_eq!(output.command, None);
        assert_eq!(output.explanation, "This is just an explanation without a command.");
    }
    
    #[test]
    fn test_parse_response_no_command_prefix() {
        let response = "This is just a regular explanation without any command.";
        let output = parse_response_to_command_output(response);
        
        assert_eq!(output.command, None);
        assert_eq!(output.explanation, "This is just a regular explanation without any command.");
    }
    
    #[test]
    fn test_parse_response_command_only() {
        let response = "Command: ls -la";
        let output = parse_response_to_command_output(response);
        
        assert_eq!(output.command, Some("ls -la".to_string()));
        assert_eq!(output.explanation, "");
    }
    
    #[test]
    fn test_copy_paste_safety() {
        // Test that commands are completely clean of formatting
        let test_cases = vec![
            ("`ls -la`", "ls -la"),
            ("**find . -name '*.rs'**", "find . -name '*.rs'"),
            ("```bash\ngrep pattern file\n```", "bash\ngrep pattern file"),
            ("__pwd__", "pwd"),
            ("~~rm file~~", "rm file"),
            // Single asterisks should be preserved for shell patterns
            ("find . -name '*.rs'", "find . -name '*.rs'"),
            ("echo *", "echo *"),
        ];
        
        for (input, expected) in test_cases {
            let output = CommandOutput::with_command(input.to_string(), "".to_string());
            let cleaned = output.clean_command_formatting(input);
            assert_eq!(cleaned, expected, "Failed to clean: {}", input);
        }
    }
    
    #[test]
    fn test_command_explanation_separation() {
        let output = CommandOutput::with_command(
            "ls -la".to_string(),
            "This command lists files".to_string()
        );
        
        let formatted = output.format_for_display();
        let lines: Vec<&str> = formatted.lines().collect();
        
        // First line should be the command only
        assert_eq!(lines[0], "ls -la");
        // Second line should be empty (separation)
        assert_eq!(lines[1], "");
        // Third line should be the explanation
        assert_eq!(lines[2], "This command lists files");
    }
}
