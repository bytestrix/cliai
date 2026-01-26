use colored::*;
use std::fmt;
use crate::logging::{get_logger, LogCategory, LogContext};

/// Enhanced error handling with actionable suggestions
#[derive(Debug, Clone)]
pub struct UserFriendlyError {
    pub error_type: ErrorType,
    pub message: String,
    pub suggestions: Vec<String>,
    pub technical_details: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorType {
    Connection,
    Configuration,
    Validation,
    Provider,
    System,
    Authentication,
    Permission,
    NotFound,
    Timeout,
    General,
}

impl UserFriendlyError {
    pub fn new(error_type: ErrorType, message: String) -> Self {
        Self {
            error_type,
            message,
            suggestions: Vec::new(),
            technical_details: None,
        }
    }

    pub fn with_suggestion(mut self, suggestion: String) -> Self {
        self.suggestions.push(suggestion);
        self
    }

    pub fn with_suggestions(mut self, suggestions: Vec<String>) -> Self {
        self.suggestions.extend(suggestions);
        self
    }

    pub fn with_technical_details(mut self, details: String) -> Self {
        self.technical_details = Some(details);
        self
    }

    /// Display the error in a user-friendly format with actionable suggestions
    pub fn display(&self) {
        // Log the error (privacy-safe)
        if let Ok(logger) = get_logger() {
            if let Ok(logger_guard) = logger.lock() {
                let context = LogContext {
                    component: Some("error_handling".to_string()),
                    operation: Some("display_error".to_string()),
                    duration_ms: None,
                    error_code: Some(format!("{:?}", self.error_type)),
                    provider_type: None,
                    os_type: None,
                    operation_type: None,
                    target_ms: None,
                    success: Some(false),
                };
                
                let _ = logger_guard.log_error(
                    LogCategory::System,
                    format!("{}: {}", format!("{:?}", self.error_type), self.message),
                    Some(context),
                );
            }
        }
        
        let icon = match self.error_type {
            ErrorType::Connection => "🔌",
            ErrorType::Configuration => "⚙️",
            ErrorType::Validation => "❌",
            ErrorType::Provider => "🤖",
            ErrorType::System => "💻",
            ErrorType::Authentication => "🔐",
            ErrorType::Permission => "🚫",
            ErrorType::NotFound => "🔍",
            ErrorType::Timeout => "⏱️",
            ErrorType::General => "❌",
        };

        let error_title = match self.error_type {
            ErrorType::Connection => "Connection Error",
            ErrorType::Configuration => "Configuration Error",
            ErrorType::Validation => "Validation Error",
            ErrorType::Provider => "AI Provider Error",
            ErrorType::System => "System Error",
            ErrorType::Authentication => "Authentication Error",
            ErrorType::Permission => "Permission Error",
            ErrorType::NotFound => "Not Found",
            ErrorType::Timeout => "Timeout Error",
            ErrorType::General => "Error",
        };

        eprintln!("{} {}: {}", 
            icon, 
            error_title.bold().red(), 
            self.message
        );

        if !self.suggestions.is_empty() {
            eprintln!();
            eprintln!("{} {}", "💡".cyan(), "Suggested solutions:".bold().yellow());
            for (i, suggestion) in self.suggestions.iter().enumerate() {
                eprintln!("  {}. {}", (i + 1).to_string().green(), suggestion);
            }
        }

        if let Some(details) = &self.technical_details {
            eprintln!();
            eprintln!("{} {}", "🔧".dimmed(), "Technical details:".dimmed());
            eprintln!("   {}", details.dimmed());
        }
    }
}

impl fmt::Display for UserFriendlyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for UserFriendlyError {}

/// Convert common errors into user-friendly errors with actionable suggestions
pub fn enhance_error(error: &anyhow::Error) -> UserFriendlyError {
    let error_msg = error.to_string().to_lowercase();

    // Connection errors
    if error_msg.contains("connection refused") || error_msg.contains("failed to connect") {
        return UserFriendlyError::new(
            ErrorType::Connection,
            "Unable to connect to AI provider".to_string(),
        )
        .with_suggestions(vec![
            "Ensure Ollama is running: ollama serve".to_string(),
            "Check if the service is accessible at the configured URL".to_string(),
            "Verify your internet connection for cloud providers".to_string(),
            "Run 'cliai provider-status' to check provider availability".to_string(),
            "Try switching to a different provider if available".to_string(),
        ])
        .with_technical_details(error.to_string());
    }

    // Model not found errors
    if (error_msg.contains("model") && error_msg.contains("not found")) || 
       error_msg.contains("no such model") || 
       error_msg.contains("not found") && error_msg.contains("error") {
        return UserFriendlyError::new(
            ErrorType::Provider,
            "The requested AI model is not available".to_string(),
        )
        .with_suggestions(vec![
            "List available models: cliai list-models".to_string(),
            "Install the default model: ollama pull mistral".to_string(),
            "Select a different model: cliai select <model-name>".to_string(),
            "Check if Ollama is running: ollama serve".to_string(),
        ])
        .with_technical_details(error.to_string());
    }

    // Rate limiting errors
    if error_msg.contains("rate limit") || error_msg.contains("too many requests") {
        return UserFriendlyError::new(
            ErrorType::Provider,
            "Rate limit exceeded for AI provider".to_string(),
        )
        .with_suggestions(vec![
            "Wait a moment and try again".to_string(),
            "Switch to local provider to avoid rate limits".to_string(),
            "Check your API usage limits".to_string(),
            "Consider upgrading your subscription if using cloud providers".to_string(),
        ])
        .with_technical_details(error.to_string());
    }

    // Network connectivity errors
    if error_msg.contains("network") || error_msg.contains("dns") || error_msg.contains("resolve") {
        return UserFriendlyError::new(
            ErrorType::Connection,
            "Network connectivity issue detected".to_string(),
        )
        .with_suggestions(vec![
            "Check your internet connection".to_string(),
            "Try using a local provider: ollama serve".to_string(),
            "Verify DNS settings if using custom endpoints".to_string(),
            "Check firewall settings that might block connections".to_string(),
        ])
        .with_technical_details(error.to_string());
    }

    // Timeout errors
    if error_msg.contains("timeout") || error_msg.contains("timed out") {
        return UserFriendlyError::new(
            ErrorType::Timeout,
            "Request timed out while waiting for AI response".to_string(),
        )
        .with_suggestions(vec![
            "Try again - the AI provider might be temporarily busy".to_string(),
            "Check your internet connection".to_string(),
            "Consider using a local provider for faster responses".to_string(),
            "Increase context timeout: cliai context-timeout 5000".to_string(),
            "Try a simpler request to test connectivity".to_string(),
        ])
        .with_technical_details(error.to_string());
    }

    // Configuration errors
    if error_msg.contains("config") || error_msg.contains("configuration") {
        return UserFriendlyError::new(
            ErrorType::Configuration,
            "Configuration issue detected".to_string(),
        )
        .with_suggestions(vec![
            "Check your configuration: cliai config".to_string(),
            "Reset to defaults by deleting ~/.config/cliai/config.json".to_string(),
            "Verify all settings are valid".to_string(),
            "Check file permissions on config directory".to_string(),
            "Ensure config directory exists: mkdir -p ~/.config/cliai".to_string(),
        ])
        .with_technical_details(error.to_string());
    }

    // Provider errors
    if error_msg.contains("no ai providers") || error_msg.contains("provider") {
        return UserFriendlyError::new(
            ErrorType::Provider,
            "No AI providers are available".to_string(),
        )
        .with_suggestions(vec![
            "Start Ollama for offline functionality: ollama serve".to_string(),
            "Install a model: ollama pull mistral".to_string(),
            "Check provider status: cliai provider-status".to_string(),
            "Verify API keys are configured for cloud providers".to_string(),
            "Ensure at least one provider is properly configured".to_string(),
        ])
        .with_technical_details(error.to_string());
    }

    // Command validation errors
    if error_msg.contains("validation") || error_msg.contains("invalid command") {
        return UserFriendlyError::new(
            ErrorType::Validation,
            "Command validation failed".to_string(),
        )
        .with_suggestions(vec![
            "Try rephrasing your request more specifically".to_string(),
            "Avoid using placeholder text like '/path/to/file'".to_string(),
            "Check if the command syntax is correct".to_string(),
            "Use standard command flags instead of custom ones".to_string(),
        ])
        .with_technical_details(error.to_string());
    }

    // Permission errors
    if error_msg.contains("permission denied") || error_msg.contains("access denied") {
        return UserFriendlyError::new(
            ErrorType::Permission,
            "Permission denied".to_string(),
        )
        .with_suggestions(vec![
            "Check file/directory permissions".to_string(),
            "Try running with appropriate permissions".to_string(),
            "Ensure you have write access to the target location".to_string(),
            "Check if the file is being used by another process".to_string(),
            "Verify you own the file or have necessary permissions".to_string(),
        ])
        .with_technical_details(error.to_string());
    }

    // File not found errors
    if error_msg.contains("not found") || error_msg.contains("no such file") {
        return UserFriendlyError::new(
            ErrorType::NotFound,
            "File or resource not found".to_string(),
        )
        .with_suggestions(vec![
            "Check if the file path is correct".to_string(),
            "Verify the file exists in the current directory".to_string(),
            "Use absolute paths if relative paths aren't working".to_string(),
            "Check spelling of file names and paths".to_string(),
            "Ensure the file hasn't been moved or deleted".to_string(),
        ])
        .with_technical_details(error.to_string());
    }

    // Authentication errors
    if error_msg.contains("unauthorized") || error_msg.contains("authentication") {
        return UserFriendlyError::new(
            ErrorType::Authentication,
            "Authentication failed".to_string(),
        )
        .with_suggestions(vec![
            "Check your API key configuration".to_string(),
            "Try logging in again: cliai login".to_string(),
            "Verify your subscription status".to_string(),
            "Ensure API key has not expired".to_string(),
            "Check if you have the necessary permissions".to_string(),
        ])
        .with_technical_details(error.to_string());
    }

    // System errors
    if error_msg.contains("system") || error_msg.contains("os error") {
        return UserFriendlyError::new(
            ErrorType::System,
            "System error occurred".to_string(),
        )
        .with_suggestions(vec![
            "Check system resources (disk space, memory)".to_string(),
            "Verify system permissions".to_string(),
            "Try restarting the application".to_string(),
            "Check system logs for more details".to_string(),
            "Ensure system dependencies are installed".to_string(),
        ])
        .with_technical_details(error.to_string());
    }

    // Disk space errors
    if error_msg.contains("no space") || error_msg.contains("disk full") {
        return UserFriendlyError::new(
            ErrorType::System,
            "Insufficient disk space".to_string(),
        )
        .with_suggestions(vec![
            "Free up disk space by removing unnecessary files".to_string(),
            "Check disk usage: df -h".to_string(),
            "Clear temporary files and caches".to_string(),
            "Move files to external storage if needed".to_string(),
        ])
        .with_technical_details(error.to_string());
    }

    // Generic error fallback
    UserFriendlyError::new(
        ErrorType::General,
        "An unexpected error occurred".to_string(),
    )
    .with_suggestions(vec![
        "Try the command again".to_string(),
        "Check 'cliai provider-status' for system status".to_string(),
        "Report this issue if it persists".to_string(),
    ])
    .with_technical_details(error.to_string())
}

/// Display success messages with consistent formatting
pub fn display_success(message: &str) {
    println!("{} {}", "✅".green(), message);
}

/// Display warning messages with consistent formatting
pub fn display_warning(message: &str) {
    println!("{} {}", "⚠️".yellow(), message.yellow());
}

/// Display info messages with consistent formatting
pub fn display_info(message: &str) {
    println!("{} {}", "💡".cyan(), message.dimmed());
}

/// Display configuration change confirmations
pub fn display_config_change(setting: &str, old_value: &str, new_value: &str) {
    println!("{} Configuration updated:", "✅".green());
    println!("  {}: {} → {}", 
        setting.bold(), 
        old_value.dimmed(), 
        new_value.green().bold()
    );
    println!("{} Changes take effect immediately", "💡".cyan());
}

/// Display progress information during operations
pub fn display_progress(message: &str) {
    println!("{} {}", "⏳".cyan(), message.dimmed());
}

/// Display completion confirmation
pub fn display_completion(message: &str) {
    println!("{} {}", "✨".green(), message.green());
}

/// Display helpful tips
pub fn display_tip(message: &str) {
    println!("{} {}: {}", "💡".cyan(), "Tip".bold().cyan(), message.dimmed());
}

/// Display system status information
pub fn display_status(component: &str, status: &str, is_healthy: bool) {
    let icon = if is_healthy { "✅" } else { "❌" };
    let status_color = if is_healthy { status.green() } else { status.red() };
    println!("{} {}: {}", icon, component.bold(), status_color);
}

/// Display command suggestions
pub fn display_command_suggestion(description: &str, command: &str) {
    println!("{} {}: {}", "💡".cyan(), description, command.green());
}

/// Display interface consistency message
pub fn display_interface_reminder() {
    println!("{} Remember: Use {} for all requests", 
        "💡".cyan(), 
        "cliai <your request>".green().bold()
    );
    println!("   No quotes needed, just speak naturally!");
}

/// Display terminal environment compatibility message
pub fn display_terminal_compatibility() {
    println!("{} CLIAI works consistently across all terminal environments", "✅".green());
    println!("   Tested with: bash, zsh, fish, PowerShell, and more");
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::anyhow;

    #[test]
    fn test_user_friendly_error_creation() {
        let error = UserFriendlyError::new(
            ErrorType::Connection,
            "Test connection error".to_string(),
        );

        assert_eq!(error.error_type, ErrorType::Connection);
        assert_eq!(error.message, "Test connection error");
        assert!(error.suggestions.is_empty());
        assert!(error.technical_details.is_none());
    }

    #[test]
    fn test_user_friendly_error_with_suggestions() {
        let error = UserFriendlyError::new(
            ErrorType::Configuration,
            "Config error".to_string(),
        )
        .with_suggestion("Try this".to_string())
        .with_suggestion("Or this".to_string());

        assert_eq!(error.suggestions.len(), 2);
        assert_eq!(error.suggestions[0], "Try this");
        assert_eq!(error.suggestions[1], "Or this");
    }

    #[test]
    fn test_user_friendly_error_with_technical_details() {
        let error = UserFriendlyError::new(
            ErrorType::System,
            "System error".to_string(),
        )
        .with_technical_details("Stack trace here".to_string());

        assert_eq!(error.technical_details, Some("Stack trace here".to_string()));
    }

    #[test]
    fn test_enhance_connection_error() {
        let error = anyhow!("Connection refused");
        let enhanced = enhance_error(&error);

        assert_eq!(enhanced.error_type, ErrorType::Connection);
        assert!(enhanced.message.contains("Unable to connect"));
        assert!(!enhanced.suggestions.is_empty());
        assert!(enhanced.suggestions.iter().any(|s| s.contains("ollama serve")));
    }

    #[test]
    fn test_enhance_timeout_error() {
        let error = anyhow!("Request timed out");
        let enhanced = enhance_error(&error);

        assert_eq!(enhanced.error_type, ErrorType::Timeout);
        assert!(enhanced.message.contains("timed out"));
        assert!(!enhanced.suggestions.is_empty());
    }

    #[test]
    fn test_enhance_provider_error() {
        let error = anyhow!("No AI providers are available");
        let enhanced = enhance_error(&error);

        assert_eq!(enhanced.error_type, ErrorType::Provider);
        assert!(enhanced.message.contains("No AI providers"));
        assert!(enhanced.suggestions.iter().any(|s| s.contains("ollama serve")));
    }

    #[test]
    fn test_enhance_permission_error() {
        let error = anyhow!("Permission denied");
        let enhanced = enhance_error(&error);

        assert_eq!(enhanced.error_type, ErrorType::Permission);
        assert!(enhanced.message.contains("Permission denied"));
        assert!(!enhanced.suggestions.is_empty());
    }

    #[test]
    fn test_enhance_not_found_error() {
        let error = anyhow!("File not found");
        let enhanced = enhance_error(&error);

        assert_eq!(enhanced.error_type, ErrorType::NotFound);
        assert!(enhanced.message.contains("not found"));
        assert!(!enhanced.suggestions.is_empty());
    }

    #[test]
    fn test_enhance_generic_error() {
        let error = anyhow!("Some random error");
        let enhanced = enhance_error(&error);

        assert_eq!(enhanced.error_type, ErrorType::General);
        assert!(enhanced.message.contains("unexpected error"));
        assert!(!enhanced.suggestions.is_empty());
    }

    #[test]
    fn test_error_display_trait() {
        let error = UserFriendlyError::new(
            ErrorType::Validation,
            "Test error".to_string(),
        );

        assert_eq!(format!("{}", error), "Test error");
    }

    #[test]
    fn test_error_types() {
        // Test all error types exist and are different
        let types = vec![
            ErrorType::Connection,
            ErrorType::Configuration,
            ErrorType::Validation,
            ErrorType::Provider,
            ErrorType::System,
            ErrorType::Authentication,
            ErrorType::Permission,
            ErrorType::NotFound,
            ErrorType::Timeout,
            ErrorType::General,
        ];

        // Ensure all types are unique
        for (i, type1) in types.iter().enumerate() {
            for (j, type2) in types.iter().enumerate() {
                if i != j {
                    assert_ne!(type1, type2);
                }
            }
        }
    }
}