use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Privacy-preserving logging system for CLIAI
///
/// This logging system ensures that user commands and prompts are never logged
/// in production mode, only system errors and performance metrics are recorded.
/// Debug mode requires explicit user consent and clearly marks debug logs.
pub struct PrivacyLogger {
    log_file_path: PathBuf,
    debug_mode: bool,
    debug_consent_given: bool,
    writer: Arc<Mutex<Option<std::fs::File>>>,
}

/// Log entry structure for structured logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub category: LogCategory,
    pub message: String,
    pub context: Option<LogContext>,
    pub is_debug: bool,
}

/// Log levels for different types of events
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LogLevel {
    Error,
    Warning,
    Info,
    Debug,
}

/// Categories for different types of log events
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LogCategory {
    System,
    Configuration,
    Provider,
    Performance,
    Validation,
    Safety,
    Authentication,
    Network,
    Debug,
}

/// Context information for log entries (privacy-safe)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogContext {
    pub component: Option<String>,
    pub operation: Option<String>,
    pub duration_ms: Option<u64>,
    pub error_code: Option<String>,
    pub provider_type: Option<String>,
    pub os_type: Option<String>,
    pub operation_type: Option<String>,
    pub target_ms: Option<u64>,
    pub success: Option<bool>,
}

#[allow(dead_code)]
impl LogContext {
    /// Create a new empty log context
    pub fn new() -> Self {
        Self {
            component: None,
            operation: None,
            duration_ms: None,
            error_code: None,
            provider_type: None,
            os_type: None,
            operation_type: None,
            target_ms: None,
            success: None,
        }
    }

    /// Add component information
    pub fn with_component(mut self, component: String) -> Self {
        self.component = Some(component);
        self
    }

    /// Add operation information
    pub fn with_operation(mut self, operation: String) -> Self {
        self.operation = Some(operation);
        self
    }

    /// Add duration information
    pub fn with_duration_ms(mut self, duration_ms: u64) -> Self {
        self.duration_ms = Some(duration_ms);
        self
    }

    /// Add error code information
    pub fn with_error_code(mut self, error_code: String) -> Self {
        self.error_code = Some(error_code);
        self
    }

    /// Add provider type information
    pub fn with_provider_type(mut self, provider_type: String) -> Self {
        self.provider_type = Some(provider_type);
        self
    }

    /// Add OS type information
    pub fn with_os_type(mut self, os_type: String) -> Self {
        self.os_type = Some(os_type);
        self
    }

    /// Add operation type information (for performance monitoring)
    pub fn with_operation_type(mut self, operation_type: String) -> Self {
        self.operation_type = Some(operation_type);
        self
    }

    /// Add target duration information (for performance monitoring)
    pub fn with_target_ms(mut self, target_ms: u64) -> Self {
        self.target_ms = Some(target_ms);
        self
    }

    /// Add success information
    pub fn with_success(mut self, success: bool) -> Self {
        self.success = Some(success);
        self
    }
}

#[allow(dead_code)]
impl PrivacyLogger {
    /// Create a new privacy logger instance
    pub fn new() -> Result<Self> {
        let log_file_path = Self::get_log_file_path()?;

        // Ensure log directory exists
        if let Some(parent) = log_file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        Ok(Self {
            log_file_path,
            debug_mode: false,
            debug_consent_given: false,
            writer: Arc::new(Mutex::new(None)),
        })
    }

    /// Enable debug mode with explicit user consent
    /// This method requires explicit confirmation that debug logging is acceptable
    pub fn enable_debug_mode(&mut self, consent_given: bool) -> Result<()> {
        if !consent_given {
            return Err(anyhow!("Debug mode requires explicit user consent"));
        }

        self.debug_mode = true;
        self.debug_consent_given = true;

        // Log the debug mode activation
        self.log_info(
            LogCategory::System,
            "Debug mode enabled with user consent".to_string(),
            None,
        )?;

        // Add clear warning about debug mode
        self.log_warning(
            LogCategory::Debug,
            "DEBUG MODE ACTIVE: Detailed logging enabled - may include sensitive information"
                .to_string(),
            None,
        )?;

        Ok(())
    }

    /// Disable debug mode
    pub fn disable_debug_mode(&mut self) -> Result<()> {
        if self.debug_mode {
            self.log_info(LogCategory::System, "Debug mode disabled".to_string(), None)?;
        }

        self.debug_mode = false;
        self.debug_consent_given = false;

        Ok(())
    }

    /// Log an error event (always logged, privacy-safe)
    pub fn log_error(
        &self,
        category: LogCategory,
        message: String,
        context: Option<LogContext>,
    ) -> Result<()> {
        let entry = LogEntry {
            timestamp: Utc::now(),
            level: LogLevel::Error,
            category,
            message: self.redact_sensitive_info(&message),
            context,
            is_debug: false,
        };

        self.write_log_entry(&entry)
    }

    /// Log a warning event (always logged, privacy-safe)
    pub fn log_warning(
        &self,
        category: LogCategory,
        message: String,
        context: Option<LogContext>,
    ) -> Result<()> {
        let entry = LogEntry {
            timestamp: Utc::now(),
            level: LogLevel::Warning,
            category,
            message: self.redact_sensitive_info(&message),
            context,
            is_debug: false,
        };

        self.write_log_entry(&entry)
    }

    /// Log an info event (always logged, privacy-safe)
    pub fn log_info(
        &self,
        category: LogCategory,
        message: String,
        context: Option<LogContext>,
    ) -> Result<()> {
        let entry = LogEntry {
            timestamp: Utc::now(),
            level: LogLevel::Info,
            category,
            message: self.redact_sensitive_info(&message),
            context,
            is_debug: false,
        };

        self.write_log_entry(&entry)
    }

    /// Log a debug event (only logged if debug mode is enabled with consent)
    pub fn log_debug(
        &self,
        category: LogCategory,
        message: String,
        context: Option<LogContext>,
    ) -> Result<()> {
        if !self.debug_mode || !self.debug_consent_given {
            return Ok(()); // Silently ignore debug logs when not in debug mode
        }

        let entry = LogEntry {
            timestamp: Utc::now(),
            level: LogLevel::Debug,
            category,
            message: format!("[DEBUG] {}", message), // Clear debug marking
            context,
            is_debug: true,
        };

        self.write_log_entry(&entry)
    }

    /// Log with custom context (for performance monitoring)
    pub fn log_with_context(
        &self,
        category: LogCategory,
        message: &str,
        context: &LogContext,
    ) -> Result<()> {
        let entry = LogEntry {
            timestamp: Utc::now(),
            level: LogLevel::Info,
            category,
            message: self.redact_sensitive_info(message),
            context: Some(context.clone()),
            is_debug: false,
        };

        self.write_log_entry(&entry)
    }

    /// Log system startup information
    pub fn log_startup(&self, version: &str, os_info: &str) -> Result<()> {
        let context = LogContext {
            component: Some("system".to_string()),
            operation: Some("startup".to_string()),
            duration_ms: None,
            error_code: None,
            provider_type: None,
            os_type: Some(os_info.to_string()),
            operation_type: None,
            target_ms: None,
            success: None,
        };

        self.log_info(
            LogCategory::System,
            format!("CLIAI {} started", version),
            Some(context),
        )
    }

    /// Log configuration changes (privacy-safe)
    pub fn log_config_change(&self, setting: &str, old_value: &str, new_value: &str) -> Result<()> {
        let context = LogContext {
            component: Some("configuration".to_string()),
            operation: Some("update".to_string()),
            duration_ms: None,
            error_code: None,
            provider_type: None,
            os_type: None,
            operation_type: None,
            target_ms: None,
            success: None,
        };

        // Redact potentially sensitive values
        let safe_old = self.redact_config_value(setting, old_value);
        let safe_new = self.redact_config_value(setting, new_value);

        self.log_info(
            LogCategory::Configuration,
            format!(
                "Configuration updated: {} changed from {} to {}",
                setting, safe_old, safe_new
            ),
            Some(context),
        )
    }

    /// Log provider operations (privacy-safe)
    pub fn log_provider_operation(
        &self,
        provider_type: &str,
        operation: &str,
        duration_ms: u64,
        success: bool,
    ) -> Result<()> {
        let context = LogContext {
            component: Some("provider".to_string()),
            operation: Some(operation.to_string()),
            duration_ms: Some(duration_ms),
            error_code: if success {
                None
            } else {
                Some("operation_failed".to_string())
            },
            provider_type: Some(provider_type.to_string()),
            os_type: None,
            operation_type: None,
            target_ms: None,
            success: Some(success),
        };

        let level = if success {
            LogLevel::Info
        } else {
            LogLevel::Warning
        };
        let message = format!(
            "Provider {} {}: {} ({}ms)",
            provider_type,
            operation,
            if success { "success" } else { "failed" },
            duration_ms
        );

        let entry = LogEntry {
            timestamp: Utc::now(),
            level,
            category: LogCategory::Provider,
            message,
            context: Some(context),
            is_debug: false,
        };

        self.write_log_entry(&entry)
    }

    /// Log performance metrics (privacy-safe)
    pub fn log_performance(
        &self,
        operation: &str,
        duration_ms: u64,
        details: Option<&str>,
    ) -> Result<()> {
        let context = LogContext {
            component: Some("performance".to_string()),
            operation: Some(operation.to_string()),
            duration_ms: Some(duration_ms),
            error_code: None,
            provider_type: None,
            os_type: None,
            operation_type: None,
            target_ms: None,
            success: None,
        };

        let message = if let Some(details) = details {
            format!(
                "Performance: {} completed in {}ms ({})",
                operation, duration_ms, details
            )
        } else {
            format!("Performance: {} completed in {}ms", operation, duration_ms)
        };

        self.log_info(LogCategory::Performance, message, Some(context))
    }

    /// Log validation events (privacy-safe)
    pub fn log_validation(
        &self,
        validation_type: &str,
        success: bool,
        details: Option<&str>,
    ) -> Result<()> {
        let context = LogContext {
            component: Some("validation".to_string()),
            operation: Some(validation_type.to_string()),
            duration_ms: None,
            error_code: if success {
                None
            } else {
                Some("validation_failed".to_string())
            },
            provider_type: None,
            os_type: None,
            operation_type: None,
            target_ms: None,
            success: Some(success),
        };

        let level = if success {
            LogLevel::Info
        } else {
            LogLevel::Warning
        };
        let message = if let Some(details) = details {
            format!(
                "Validation {}: {} ({})",
                validation_type,
                if success { "passed" } else { "failed" },
                details
            )
        } else {
            format!(
                "Validation {}: {}",
                validation_type,
                if success { "passed" } else { "failed" }
            )
        };

        let entry = LogEntry {
            timestamp: Utc::now(),
            level,
            category: LogCategory::Validation,
            message,
            context: Some(context),
            is_debug: false,
        };

        self.write_log_entry(&entry)
    }

    /// Log safety events (privacy-safe)
    pub fn log_safety_event(
        &self,
        event_type: &str,
        severity: &str,
        details: Option<&str>,
    ) -> Result<()> {
        let context = LogContext {
            component: Some("safety".to_string()),
            operation: Some(event_type.to_string()),
            duration_ms: None,
            error_code: None,
            provider_type: None,
            os_type: None,
            operation_type: None,
            target_ms: None,
            success: None,
        };

        let level = match severity.to_lowercase().as_str() {
            "high" | "critical" => LogLevel::Error,
            "medium" | "warning" => LogLevel::Warning,
            _ => LogLevel::Info,
        };

        let message = if let Some(details) = details {
            format!(
                "Safety event {}: {} severity ({})",
                event_type, severity, details
            )
        } else {
            format!("Safety event {}: {} severity", event_type, severity)
        };

        let entry = LogEntry {
            timestamp: Utc::now(),
            level,
            category: LogCategory::Safety,
            message,
            context: Some(context),
            is_debug: false,
        };

        self.write_log_entry(&entry)
    }

    /// Get the log file path
    fn get_log_file_path() -> Result<PathBuf> {
        let config_dir =
            dirs::config_dir().ok_or_else(|| anyhow!("Could not find config directory"))?;

        let mut log_path = config_dir;
        log_path.push("cliai");
        log_path.push("error.log");

        Ok(log_path)
    }

    /// Write a log entry to the file
    fn write_log_entry(&self, entry: &LogEntry) -> Result<()> {
        let mut writer_guard = self
            .writer
            .lock()
            .map_err(|_| anyhow!("Failed to acquire log writer lock"))?;

        // Open file if not already open
        if writer_guard.is_none() {
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&self.log_file_path)?;
            *writer_guard = Some(file);
        }

        if let Some(ref mut file) = *writer_guard {
            let log_line = self.format_log_entry(entry);
            writeln!(file, "{}", log_line)?;
            file.flush()?;
        }

        Ok(())
    }

    /// Format a log entry for writing
    fn format_log_entry(&self, entry: &LogEntry) -> String {
        let timestamp = entry.timestamp.format("%Y-%m-%d %H:%M:%S UTC");
        let level = format!("{:?}", entry.level).to_uppercase();
        let category = format!("{:?}", entry.category).to_uppercase();

        let mut formatted = format!("[{}] {} {} {}", timestamp, level, category, entry.message);

        // Add context information if present
        if let Some(ref context) = entry.context {
            let mut context_parts = Vec::new();

            if let Some(ref component) = context.component {
                context_parts.push(format!("component={}", component));
            }
            if let Some(ref operation) = context.operation {
                context_parts.push(format!("operation={}", operation));
            }
            if let Some(duration) = context.duration_ms {
                context_parts.push(format!("duration={}ms", duration));
            }
            if let Some(ref error_code) = context.error_code {
                context_parts.push(format!("error={}", error_code));
            }
            if let Some(ref provider_type) = context.provider_type {
                context_parts.push(format!("provider={}", provider_type));
            }
            if let Some(ref os_type) = context.os_type {
                context_parts.push(format!("os={}", os_type));
            }
            if let Some(ref operation_type) = context.operation_type {
                context_parts.push(format!("op_type={}", operation_type));
            }
            if let Some(target) = context.target_ms {
                context_parts.push(format!("target={}ms", target));
            }
            if let Some(success) = context.success {
                context_parts.push(format!("success={}", success));
            }

            if !context_parts.is_empty() {
                formatted.push_str(&format!(" [{}]", context_parts.join(", ")));
            }
        }

        // Mark debug entries clearly
        if entry.is_debug {
            formatted = format!("🐛 DEBUG: {}", formatted);
        }

        formatted
    }

    /// Redact sensitive information from log messages
    fn redact_sensitive_info(&self, message: &str) -> String {
        let mut redacted = message.to_string();

        // In production mode, be extra cautious about potential sensitive data
        if !self.debug_mode {
            // Redact potential file paths that might contain usernames
            redacted = regex::Regex::new(r"/home/[^/\s]+")
                .unwrap()
                .replace_all(&redacted, "/home/[USER]")
                .to_string();

            redacted = regex::Regex::new(r"/Users/[^/\s]+")
                .unwrap()
                .replace_all(&redacted, "/Users/[USER]")
                .to_string();

            // Redact potential API keys or tokens - simplified pattern
            redacted = regex::Regex::new(r"api_key=\S+")
                .unwrap()
                .replace_all(&redacted, "api_key=[REDACTED]")
                .to_string();

            redacted = regex::Regex::new(r"token=\S+")
                .unwrap()
                .replace_all(&redacted, "token=[REDACTED]")
                .to_string();

            // Redact potential passwords
            redacted = regex::Regex::new(r"password=\S+")
                .unwrap()
                .replace_all(&redacted, "password=[REDACTED]")
                .to_string();
        }

        redacted
    }

    /// Redact potentially sensitive configuration values
    fn redact_config_value(&self, setting: &str, value: &str) -> String {
        match setting.to_lowercase().as_str() {
            "api_key" | "token" | "password" | "secret" => "[REDACTED]".to_string(),
            "ollama_url" if value.contains("@") => {
                // Redact credentials from URLs
                regex::Regex::new(r"://[^@]+@")
                    .unwrap()
                    .replace_all(value, "://[REDACTED]@")
                    .to_string()
            }
            _ => value.to_string(),
        }
    }

    /// Check if debug mode is enabled
    pub fn is_debug_mode(&self) -> bool {
        self.debug_mode && self.debug_consent_given
    }

    /// Get the current log file path
    pub fn get_current_log_path(&self) -> &PathBuf {
        &self.log_file_path
    }

    /// Clear the log file (useful for testing or maintenance)
    pub fn clear_logs(&self) -> Result<()> {
        // Close the current writer
        {
            let mut writer_guard = self
                .writer
                .lock()
                .map_err(|_| anyhow!("Failed to acquire log writer lock"))?;
            *writer_guard = None;
        }

        // Remove the file if it exists
        if self.log_file_path.exists() {
            fs::remove_file(&self.log_file_path)?;
        }

        // Log the clear action (this will recreate the file)
        self.log_info(LogCategory::System, "Log file cleared".to_string(), None)?;

        Ok(())
    }
}

/// Global logger instance
static GLOBAL_LOGGER: std::sync::OnceLock<Arc<Mutex<PrivacyLogger>>> = std::sync::OnceLock::new();

/// Initialize the global logger
pub fn init_logger() -> Result<()> {
    let logger = PrivacyLogger::new()?;
    let _ = GLOBAL_LOGGER.set(Arc::new(Mutex::new(logger)));
    Ok(())
}

/// Get the global logger instance
pub fn get_logger() -> Result<Arc<Mutex<PrivacyLogger>>> {
    GLOBAL_LOGGER
        .get()
        .cloned()
        .ok_or_else(|| anyhow!("Logger not initialized. Call init_logger() first."))
}

/// Convenience macros for logging
#[macro_export]
macro_rules! log_error {
    ($category:expr, $message:expr) => {
        if let Ok(logger) = $crate::logging::get_logger() {
            if let Ok(logger_guard) = logger.lock() {
                let _ = logger_guard.log_error($category, $message.to_string(), None);
            }
        }
    };
    ($category:expr, $message:expr, $context:expr) => {
        if let Ok(logger) = $crate::logging::get_logger() {
            if let Ok(logger_guard) = logger.lock() {
                let _ = logger_guard.log_error($category, $message.to_string(), Some($context));
            }
        }
    };
}

#[macro_export]
macro_rules! log_warning {
    ($category:expr, $message:expr) => {
        if let Ok(logger) = $crate::logging::get_logger() {
            if let Ok(logger_guard) = logger.lock() {
                let _ = logger_guard.log_warning($category, $message.to_string(), None);
            }
        }
    };
    ($category:expr, $message:expr, $context:expr) => {
        if let Ok(logger) = $crate::logging::get_logger() {
            if let Ok(logger_guard) = logger.lock() {
                let _ = logger_guard.log_warning($category, $message.to_string(), Some($context));
            }
        }
    };
}

#[macro_export]
macro_rules! log_info {
    ($category:expr, $message:expr) => {
        if let Ok(logger) = $crate::logging::get_logger() {
            if let Ok(logger_guard) = logger.lock() {
                let _ = logger_guard.log_info($category, $message.to_string(), None);
            }
        }
    };
    ($category:expr, $message:expr, $context:expr) => {
        if let Ok(logger) = $crate::logging::get_logger() {
            if let Ok(logger_guard) = logger.lock() {
                let _ = logger_guard.log_info($category, $message.to_string(), Some($context));
            }
        }
    };
}

#[macro_export]
macro_rules! log_debug {
    ($category:expr, $message:expr) => {
        if let Ok(logger) = $crate::logging::get_logger() {
            if let Ok(logger_guard) = logger.lock() {
                let _ = logger_guard.log_debug($category, $message.to_string(), None);
            }
        }
    };
    ($category:expr, $message:expr, $context:expr) => {
        if let Ok(logger) = $crate::logging::get_logger() {
            if let Ok(logger_guard) = logger.lock() {
                let _ = logger_guard.log_debug($category, $message.to_string(), Some($context));
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_logger() -> (PrivacyLogger, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("test.log");

        let logger = PrivacyLogger {
            log_file_path: log_path,
            debug_mode: false,
            debug_consent_given: false,
            writer: Arc::new(Mutex::new(None)),
        };

        (logger, temp_dir)
    }

    #[test]
    fn test_privacy_logger_creation() {
        let (logger, _temp_dir) = create_test_logger();
        assert!(!logger.debug_mode);
        assert!(!logger.debug_consent_given);
    }

    #[test]
    fn test_enable_debug_mode_requires_consent() {
        let (mut logger, _temp_dir) = create_test_logger();

        // Should fail without consent
        let result = logger.enable_debug_mode(false);
        assert!(result.is_err());
        assert!(!logger.debug_mode);

        // Should succeed with consent
        let result = logger.enable_debug_mode(true);
        assert!(result.is_ok());
        assert!(logger.debug_mode);
        assert!(logger.debug_consent_given);
    }

    #[test]
    fn test_disable_debug_mode() {
        let (mut logger, _temp_dir) = create_test_logger();

        // Enable first
        logger.enable_debug_mode(true).unwrap();
        assert!(logger.debug_mode);

        // Then disable
        logger.disable_debug_mode().unwrap();
        assert!(!logger.debug_mode);
        assert!(!logger.debug_consent_given);
    }

    #[test]
    fn test_log_error() {
        let (logger, _temp_dir) = create_test_logger();

        let result = logger.log_error(LogCategory::System, "Test error message".to_string(), None);

        assert!(result.is_ok());

        // Check that log file was created and contains the message
        let log_content = fs::read_to_string(&logger.log_file_path).unwrap();
        assert!(log_content.contains("ERROR"));
        assert!(log_content.contains("SYSTEM"));
        assert!(log_content.contains("Test error message"));
    }

    #[test]
    fn test_log_with_context() {
        let (logger, _temp_dir) = create_test_logger();

        let context = LogContext {
            component: Some("test_component".to_string()),
            operation: Some("test_operation".to_string()),
            duration_ms: Some(100),
            error_code: None,
            provider_type: Some("local".to_string()),
            os_type: None,
            operation_type: None,
            success: Some(true),
            target_ms: Some(50),
        };

        let result = logger.log_info(
            LogCategory::Performance,
            "Test with context".to_string(),
            Some(context),
        );

        assert!(result.is_ok());

        let log_content = fs::read_to_string(&logger.log_file_path).unwrap();
        assert!(log_content.contains("component=test_component"));
        assert!(log_content.contains("operation=test_operation"));
        assert!(log_content.contains("duration=100ms"));
        assert!(log_content.contains("provider=local"));
    }

    #[test]
    fn test_debug_logs_only_in_debug_mode() {
        let (mut logger, _temp_dir) = create_test_logger();

        // Debug log without debug mode should be ignored
        let result = logger.log_debug(LogCategory::Debug, "Debug message".to_string(), None);
        assert!(result.is_ok());

        // Log file should not contain debug message
        if logger.log_file_path.exists() {
            let log_content = fs::read_to_string(&logger.log_file_path).unwrap();
            assert!(!log_content.contains("Debug message"));
        }

        // Enable debug mode and try again
        logger.enable_debug_mode(true).unwrap();
        let result = logger.log_debug(
            LogCategory::Debug,
            "Debug message with consent".to_string(),
            None,
        );
        assert!(result.is_ok());

        let log_content = fs::read_to_string(&logger.log_file_path).unwrap();
        assert!(log_content.contains("🐛 DEBUG"));
        assert!(log_content.contains("Debug message with consent"));
    }

    #[test]
    fn test_redact_sensitive_info() {
        let (logger, _temp_dir) = create_test_logger();

        let sensitive_message = "User path: /home/testuser/secret and api_key=sk-1234567890abcdef1234567890abcdef and token=abc123def456ghi789jkl012mno345pqr678";
        let redacted = logger.redact_sensitive_info(sensitive_message);

        assert!(redacted.contains("/home/[USER]"));
        assert!(redacted.contains("api_key=[REDACTED]"));
        assert!(redacted.contains("token=[REDACTED]"));
        assert!(!redacted.contains("testuser"));
        assert!(!redacted.contains("sk-1234567890abcdef1234567890abcdef"));
        assert!(!redacted.contains("abc123def456ghi789jkl012mno345pqr678"));
    }

    #[test]
    fn test_redact_config_value() {
        let (logger, _temp_dir) = create_test_logger();

        // Sensitive settings should be redacted
        assert_eq!(
            logger.redact_config_value("api_key", "secret123"),
            "[REDACTED]"
        );
        assert_eq!(
            logger.redact_config_value("password", "mypassword"),
            "[REDACTED]"
        );

        // Non-sensitive settings should not be redacted
        assert_eq!(logger.redact_config_value("model", "mistral"), "mistral");
        assert_eq!(logger.redact_config_value("timeout", "5000"), "5000");

        // URLs with credentials should be partially redacted
        let url_with_creds = "http://user:pass@localhost:11434";
        let redacted_url = logger.redact_config_value("ollama_url", url_with_creds);
        assert!(redacted_url.contains("[REDACTED]"));
        assert!(!redacted_url.contains("user:pass"));
    }

    #[test]
    fn test_log_provider_operation() {
        let (logger, _temp_dir) = create_test_logger();

        let result = logger.log_provider_operation("ollama", "generate", 1500, true);
        assert!(result.is_ok());

        let log_content = fs::read_to_string(&logger.log_file_path).unwrap();
        assert!(log_content.contains("Provider ollama generate: success"));
        assert!(log_content.contains("1500ms"));
        assert!(log_content.contains("provider=ollama"));
    }

    #[test]
    fn test_log_performance() {
        let (logger, _temp_dir) = create_test_logger();

        let result = logger.log_performance("command_validation", 50, Some("all checks passed"));
        assert!(result.is_ok());

        let log_content = fs::read_to_string(&logger.log_file_path).unwrap();
        assert!(log_content.contains("Performance: command_validation completed in 50ms"));
        assert!(log_content.contains("all checks passed"));
    }

    #[test]
    fn test_log_safety_event() {
        let (logger, _temp_dir) = create_test_logger();

        let result = logger.log_safety_event("dangerous_command", "high", Some("rm -rf detected"));
        assert!(result.is_ok());

        let log_content = fs::read_to_string(&logger.log_file_path).unwrap();
        assert!(log_content.contains("ERROR")); // High severity should be ERROR level
        assert!(log_content.contains("Safety event dangerous_command: high severity"));
        assert!(log_content.contains("rm -rf detected"));
    }

    #[test]
    fn test_log_config_change() {
        let (logger, _temp_dir) = create_test_logger();

        let result = logger.log_config_change("auto_execute", "false", "true");
        assert!(result.is_ok());

        let log_content = fs::read_to_string(&logger.log_file_path).unwrap();
        assert!(
            log_content.contains("Configuration updated: auto_execute changed from false to true")
        );
        assert!(log_content.contains("component=configuration"));
    }

    #[test]
    fn test_clear_logs() {
        let (logger, _temp_dir) = create_test_logger();

        // Create a log entry first
        logger
            .log_info(LogCategory::System, "Test message".to_string(), None)
            .unwrap();
        assert!(logger.log_file_path.exists());

        // Clear logs
        let result = logger.clear_logs();
        assert!(result.is_ok());

        // After clearing, the file should exist again because clear_logs() writes a log entry
        assert!(logger.log_file_path.exists());
        let log_content = fs::read_to_string(&logger.log_file_path).unwrap();
        assert!(log_content.contains("Log file cleared"));
        // The original test message should not be there anymore
        assert!(!log_content.contains("Test message"));
    }

    #[test]
    fn test_is_debug_mode() {
        let (mut logger, _temp_dir) = create_test_logger();

        assert!(!logger.is_debug_mode());

        logger.enable_debug_mode(true).unwrap();
        assert!(logger.is_debug_mode());

        logger.disable_debug_mode().unwrap();
        assert!(!logger.is_debug_mode());
    }

    #[test]
    fn test_log_entry_formatting() {
        let (logger, _temp_dir) = create_test_logger();

        let context = LogContext {
            component: Some("test".to_string()),
            operation: Some("format_test".to_string()),
            duration_ms: Some(42),
            error_code: Some("TEST_ERROR".to_string()),
            provider_type: Some("test_provider".to_string()),
            os_type: Some("linux".to_string()),
            operation_type: None,
            success: Some(false),
            target_ms: Some(30),
        };

        let entry = LogEntry {
            timestamp: Utc::now(),
            level: LogLevel::Warning,
            category: LogCategory::System,
            message: "Test formatting".to_string(),
            context: Some(context),
            is_debug: false,
        };

        let formatted = logger.format_log_entry(&entry);

        assert!(formatted.contains("WARNING"));
        assert!(formatted.contains("SYSTEM"));
        assert!(formatted.contains("Test formatting"));
        assert!(formatted.contains("component=test"));
        assert!(formatted.contains("operation=format_test"));
        assert!(formatted.contains("duration=42ms"));
        assert!(formatted.contains("error=TEST_ERROR"));
        assert!(formatted.contains("provider=test_provider"));
        assert!(formatted.contains("os=linux"));
    }

    #[test]
    fn test_debug_entry_marking() {
        let (logger, _temp_dir) = create_test_logger();

        let entry = LogEntry {
            timestamp: Utc::now(),
            level: LogLevel::Debug,
            category: LogCategory::Debug,
            message: "[DEBUG] Test debug message".to_string(),
            context: None,
            is_debug: true,
        };

        let formatted = logger.format_log_entry(&entry);
        assert!(formatted.contains("🐛 DEBUG"));
    }
}
