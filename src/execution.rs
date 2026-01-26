use serde::{Deserialize, Serialize};
use crate::config::{Config, SafetyLevel};
use crate::validation::{ValidationResult, SecurityWarning};

/// Execution mode determines how commands should be handled
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExecutionMode {
    /// Show command only, never execute (default safe behavior)
    SuggestOnly,
    /// Can execute without confirmation (for safe commands)
    Safe,
    /// Requires user confirmation before execution (for sensitive commands)
    RequiresConfirmation(Vec<String>), // Reasons for requiring confirmation
    /// Show command with "DRY RUN:" prefix, never execute
    DryRunOnly,
    /// Cannot execute, show reason (for blocked commands)
    Blocked(String), // Reason for blocking
}

impl ExecutionMode {
    /// Determine execution mode based on configuration and validation result
    pub fn determine(config: &Config, validation_result: &ValidationResult) -> Self {
        // If dry-run mode is enabled, always use DryRunOnly
        if config.dry_run {
            return ExecutionMode::DryRunOnly;
        }

        match validation_result {
            ValidationResult::Valid(_) | ValidationResult::Rewritten(_, _) => {
                if config.auto_execute {
                    ExecutionMode::Safe
                } else {
                    ExecutionMode::RequiresConfirmation(vec!["User confirmation required (auto-execute: off)".to_string()])
                }
            }
            ValidationResult::Invalid(_, errors) => {
                let error_msg = format!("Command validation failed: {}", 
                    errors.iter()
                        .map(|e| format!("{:?}", e))
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                ExecutionMode::Blocked(error_msg)
            }
            ValidationResult::Sensitive(_, warnings) => {
                let reasons: Vec<String> = warnings.iter()
                    .map(|w| match w {
                        SecurityWarning::DataLoss(msg) => format!("Data Loss Risk: {}", msg),
                        SecurityWarning::SystemModification(msg) => format!("System Modification: {}", msg),
                        SecurityWarning::DangerousPattern(msg) => format!("Dangerous Pattern: {}", msg),
                    })
                    .collect();

                // Sensitive commands always require confirmation, even with auto_execute enabled
                match config.safety_level {
                    SafetyLevel::High => {
                        // High safety: block some dangerous commands entirely
                        if warnings.iter().any(|w| matches!(w, SecurityWarning::DangerousPattern(_))) {
                            ExecutionMode::Blocked("Command blocked due to high safety level".to_string())
                        } else {
                            ExecutionMode::RequiresConfirmation(reasons)
                        }
                    }
                    SafetyLevel::Medium | SafetyLevel::Low => {
                        ExecutionMode::RequiresConfirmation(reasons)
                    }
                }
            }
        }
    }

    /// Check if the command can be executed
    pub fn can_execute(&self) -> bool {
        matches!(self, ExecutionMode::Safe | ExecutionMode::RequiresConfirmation(_))
    }

    /// Check if confirmation is required
    pub fn requires_confirmation(&self) -> bool {
        matches!(self, ExecutionMode::RequiresConfirmation(_))
    }

    /// Check if this is dry-run mode
    pub fn is_dry_run(&self) -> bool {
        matches!(self, ExecutionMode::DryRunOnly)
    }

    /// Check if the command is blocked
    pub fn is_blocked(&self) -> bool {
        matches!(self, ExecutionMode::Blocked(_))
    }

    /// Get the display prefix for the command
    pub fn get_display_prefix(&self) -> Option<String> {
        match self {
            ExecutionMode::DryRunOnly => Some("DRY RUN: ".to_string()),
            ExecutionMode::Blocked(reason) => Some(format!("BLOCKED ({}): ", reason)),
            _ => None,
        }
    }

    /// Get confirmation reasons if any
    pub fn get_confirmation_reasons(&self) -> Vec<String> {
        match self {
            ExecutionMode::RequiresConfirmation(reasons) => reasons.clone(),
            _ => Vec::new(),
        }
    }

    /// Get block reason if blocked
    pub fn get_block_reason(&self) -> Option<String> {
        match self {
            ExecutionMode::Blocked(reason) => Some(reason.clone()),
            _ => None,
        }
    }
}

/// Enhanced command output with execution mode information
#[derive(Debug, Clone)]
pub struct ExecutableCommand {
    pub command: String,
    pub explanation: String,
    pub execution_mode: ExecutionMode,
    pub warnings: Vec<String>,
}

impl ExecutableCommand {
    pub fn new(command: String, explanation: String, execution_mode: ExecutionMode) -> Self {
        Self {
            command,
            explanation,
            execution_mode,
            warnings: Vec::new(),
        }
    }

    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    /// Format the command for display with appropriate prefixes
    pub fn format_for_display(&self) -> String {
        let mut output = String::new();

        // Add execution mode prefix if needed
        if let Some(prefix) = self.execution_mode.get_display_prefix() {
            output.push_str(&prefix);
        }

        // Add the command
        output.push_str(&self.command);
        output.push('\n');

        // Add explanation if present
        if !self.explanation.is_empty() {
            output.push('\n');
            output.push_str(&self.explanation);
            output.push('\n');
        }

        // Add warnings
        for warning in &self.warnings {
            output.push_str(&format!("⚠️  {}\n", warning));
        }

        // Add confirmation reasons if needed
        let reasons = self.execution_mode.get_confirmation_reasons();
        if !reasons.is_empty() {
            output.push('\n');
            output.push_str("⚠️  This command requires confirmation:\n");
            for reason in reasons {
                output.push_str(&format!("   • {}\n", reason));
            }
        }

        // Add block reason if blocked
        if let Some(reason) = self.execution_mode.get_block_reason() {
            output.push('\n');
            output.push_str(&format!("🚫 Command blocked: {}\n", reason));
        }

        output
    }

    /// Get execution instructions for the user
    pub fn get_execution_instructions(&self) -> Option<String> {
        match &self.execution_mode {
            ExecutionMode::SuggestOnly => {
                Some("To execute this command, copy and paste it into your terminal.".to_string())
            }
            ExecutionMode::RequiresConfirmation(_) => {
                Some("This command requires confirmation due to potential risks.".to_string())
            }
            ExecutionMode::DryRunOnly => {
                Some("Dry-run mode: Command shown for preview only.".to_string())
            }
            ExecutionMode::Blocked(reason) => {
                Some(format!("Command cannot be executed: {}", reason))
            }
            ExecutionMode::Safe => None, // No special instructions needed
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation::{ValidationError};

    fn create_test_config(auto_execute: bool, dry_run: bool, safety_level: SafetyLevel) -> Config {
        Config {
            model: "test".to_string(),
            ollama_url: "http://localhost:11434".to_string(),
            prefix: None,
            auto_execute,
            dry_run,
            safety_level,
            context_timeout: 2000,
        }
    }

    #[test]
    fn test_execution_mode_dry_run_override() {
        let config = create_test_config(true, true, SafetyLevel::Low);
        let validation = ValidationResult::Valid("ls -la".to_string());
        
        let mode = ExecutionMode::determine(&config, &validation);
        assert_eq!(mode, ExecutionMode::DryRunOnly);
    }

    #[test]
    fn test_execution_mode_auto_execute_enabled() {
        let config = create_test_config(true, false, SafetyLevel::Medium);
        let validation = ValidationResult::Valid("ls -la".to_string());
        
        let mode = ExecutionMode::determine(&config, &validation);
        assert_eq!(mode, ExecutionMode::Safe);
    }

    #[test]
    fn test_execution_mode_auto_execute_disabled() {
        let config = create_test_config(false, false, SafetyLevel::Medium);
        let validation = ValidationResult::Valid("ls -la".to_string());
        
        let mode = ExecutionMode::determine(&config, &validation);
        assert_eq!(mode, ExecutionMode::SuggestOnly);
    }

    #[test]
    fn test_execution_mode_invalid_command() {
        let config = create_test_config(true, false, SafetyLevel::Medium);
        let validation = ValidationResult::Invalid(
            "bad command".to_string(),
            vec![ValidationError::SyntaxError("Invalid syntax".to_string())]
        );
        
        let mode = ExecutionMode::determine(&config, &validation);
        assert!(matches!(mode, ExecutionMode::Blocked(_)));
    }

    #[test]
    fn test_execution_mode_sensitive_command() {
        let config = create_test_config(true, false, SafetyLevel::Medium);
        let validation = ValidationResult::Sensitive(
            "rm -rf /tmp/test".to_string(),
            vec![SecurityWarning::DataLoss("Will delete files".to_string())]
        );
        
        let mode = ExecutionMode::determine(&config, &validation);
        assert!(matches!(mode, ExecutionMode::RequiresConfirmation(_)));
    }

    #[test]
    fn test_execution_mode_high_safety_blocks_dangerous() {
        let config = create_test_config(true, false, SafetyLevel::High);
        let validation = ValidationResult::Sensitive(
            ":(){ :|:& };:".to_string(),
            vec![SecurityWarning::DangerousPattern("Fork bomb detected".to_string())]
        );
        
        let mode = ExecutionMode::determine(&config, &validation);
        assert!(matches!(mode, ExecutionMode::Blocked(_)));
    }

    #[test]
    fn test_executable_command_display_dry_run() {
        let mode = ExecutionMode::DryRunOnly;
        let cmd = ExecutableCommand::new(
            "ls -la".to_string(),
            "List files".to_string(),
            mode
        );
        
        let display = cmd.format_for_display();
        assert!(display.starts_with("DRY RUN: ls -la"));
    }

    #[test]
    fn test_executable_command_display_blocked() {
        let mode = ExecutionMode::Blocked("Invalid command".to_string());
        let cmd = ExecutableCommand::new(
            "bad command".to_string(),
            "".to_string(),
            mode
        );
        
        let display = cmd.format_for_display();
        assert!(display.contains("🚫 Command blocked: Invalid command"));
    }

    #[test]
    fn test_executable_command_display_requires_confirmation() {
        let mode = ExecutionMode::RequiresConfirmation(vec![
            "Data Loss Risk: Will delete files".to_string()
        ]);
        let cmd = ExecutableCommand::new(
            "rm -rf /tmp/test".to_string(),
            "Remove test directory".to_string(),
            mode
        );
        
        let display = cmd.format_for_display();
        assert!(display.contains("This command requires confirmation"));
        assert!(display.contains("Data Loss Risk: Will delete files"));
    }

    #[test]
    fn test_execution_mode_can_execute() {
        assert!(ExecutionMode::Safe.can_execute());
        assert!(ExecutionMode::RequiresConfirmation(vec![]).can_execute());
        assert!(!ExecutionMode::SuggestOnly.can_execute());
        assert!(!ExecutionMode::DryRunOnly.can_execute());
        assert!(!ExecutionMode::Blocked("reason".to_string()).can_execute());
    }

    #[test]
    fn test_execution_mode_requires_confirmation() {
        assert!(!ExecutionMode::Safe.requires_confirmation());
        assert!(ExecutionMode::RequiresConfirmation(vec![]).requires_confirmation());
        assert!(!ExecutionMode::SuggestOnly.requires_confirmation());
        assert!(!ExecutionMode::DryRunOnly.requires_confirmation());
        assert!(!ExecutionMode::Blocked("reason".to_string()).requires_confirmation());
    }

    #[test]
    fn test_execution_mode_is_dry_run() {
        assert!(!ExecutionMode::Safe.is_dry_run());
        assert!(!ExecutionMode::RequiresConfirmation(vec![]).is_dry_run());
        assert!(!ExecutionMode::SuggestOnly.is_dry_run());
        assert!(ExecutionMode::DryRunOnly.is_dry_run());
        assert!(!ExecutionMode::Blocked("reason".to_string()).is_dry_run());
    }

    #[test]
    fn test_execution_mode_is_blocked() {
        assert!(!ExecutionMode::Safe.is_blocked());
        assert!(!ExecutionMode::RequiresConfirmation(vec![]).is_blocked());
        assert!(!ExecutionMode::SuggestOnly.is_blocked());
        assert!(!ExecutionMode::DryRunOnly.is_blocked());
        assert!(ExecutionMode::Blocked("reason".to_string()).is_blocked());
    }

    #[test]
    fn test_execution_instructions() {
        let safe_cmd = ExecutableCommand::new(
            "ls".to_string(),
            "".to_string(),
            ExecutionMode::Safe
        );
        assert!(safe_cmd.get_execution_instructions().is_none());

        let suggest_cmd = ExecutableCommand::new(
            "ls".to_string(),
            "".to_string(),
            ExecutionMode::SuggestOnly
        );
        assert!(suggest_cmd.get_execution_instructions().unwrap().contains("copy and paste"));

        let dry_run_cmd = ExecutableCommand::new(
            "ls".to_string(),
            "".to_string(),
            ExecutionMode::DryRunOnly
        );
        assert!(dry_run_cmd.get_execution_instructions().unwrap().contains("Dry-run mode"));
    }
}