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
    /// Multi-step execution with individual step validation
    MultiStep(Vec<ExecutableStep>),
}

/// Represents a single step in a multi-step command execution
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutableStep {
    pub command: String,
    pub description: String,
    pub step_number: usize,
    pub depends_on_previous: bool,
    pub validation_result: Option<ValidationResult>,
    pub execution_mode: Option<Box<ExecutionMode>>,
}

/// Multi-step command parser and executor
#[derive(Debug, Clone)]
pub struct MultiStepHandler {
    pub steps: Vec<ExecutableStep>,
    pub total_steps: usize,
    pub current_step: usize,
}

impl MultiStepHandler {
    /// Parse a multi-line command into individual steps
    pub fn parse_multi_step_command(command_text: &str) -> Option<Self> {
        let lines: Vec<&str> = command_text.lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .collect();
        
        if lines.len() <= 1 {
            return None; // Not a multi-step command
        }
        
        let mut steps = Vec::new();
        for (index, line) in lines.iter().enumerate() {
            // Check if this step depends on the previous one
            let depends_on_previous = line.contains("&&") || 
                                    (index > 0 && !line.contains("||") && !line.contains(";"));
            
            steps.push(ExecutableStep {
                command: line.to_string(),
                description: format!("Step {}: {}", index + 1, 
                    if line.len() > 50 { 
                        format!("{}...", &line[..47]) 
                    } else { 
                        line.to_string() 
                    }
                ),
                step_number: index + 1,
                depends_on_previous,
                validation_result: None,
                execution_mode: None,
            });
        }
        
        Some(Self {
            total_steps: steps.len(),
            current_step: 0,
            steps,
        })
    }
    
    /// Get the next step to execute
    pub fn get_next_step(&mut self) -> Option<&mut ExecutableStep> {
        if self.current_step < self.steps.len() {
            Some(&mut self.steps[self.current_step])
        } else {
            None
        }
    }
    
    /// Mark current step as completed and move to next
    pub fn complete_current_step(&mut self, success: bool) -> bool {
        if self.current_step < self.steps.len() {
            self.current_step += 1;
            
            // If step failed and next step depends on it, skip remaining steps
            if !success && self.current_step < self.steps.len() {
                if self.steps[self.current_step].depends_on_previous {
                    return false; // Stop execution
                }
            }
            
            true
        } else {
            false
        }
    }
    
    /// Check if there are more steps to execute
    pub fn has_more_steps(&self) -> bool {
        self.current_step < self.steps.len()
    }
    
    /// Get progress information
    pub fn get_progress(&self) -> (usize, usize) {
        (self.current_step, self.total_steps)
    }
    
    /// Format steps for display
    pub fn format_steps_for_display(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("Multi-step execution ({} steps):\n", self.total_steps));
        
        for (index, step) in self.steps.iter().enumerate() {
            let status = if index < self.current_step {
                "✓"
            } else if index == self.current_step {
                "→"
            } else {
                " "
            };
            
            output.push_str(&format!("  {} Step {}: {}\n", 
                status, step.step_number, step.description));
        }
        
        output
    }
}

#[allow(dead_code)]
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
                    ExecutionMode::SuggestOnly
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
    
    /// Determine execution mode for multi-step commands
    pub fn determine_multi_step(_config: &Config, command_text: &str) -> Self {
        if let Some(handler) = MultiStepHandler::parse_multi_step_command(command_text) {
            ExecutionMode::MultiStep(handler.steps)
        } else {
            // Fall back to single command validation
            ExecutionMode::SuggestOnly
        }
    }

    /// Check if the command can be executed
    pub fn can_execute(&self) -> bool {
        matches!(self, 
            ExecutionMode::Safe | 
            ExecutionMode::RequiresConfirmation(_) |
            ExecutionMode::MultiStep(_)
        )
    }

    /// Check if confirmation is required
    pub fn requires_confirmation(&self) -> bool {
        matches!(self, 
            ExecutionMode::RequiresConfirmation(_) |
            ExecutionMode::MultiStep(_)
        )
    }

    /// Check if this is dry-run mode
    pub fn is_dry_run(&self) -> bool {
        matches!(self, ExecutionMode::DryRunOnly)
    }

    /// Check if the command is blocked
    pub fn is_blocked(&self) -> bool {
        matches!(self, ExecutionMode::Blocked(_))
    }
    
    /// Check if this is multi-step execution
    pub fn is_multi_step(&self) -> bool {
        matches!(self, ExecutionMode::MultiStep(_))
    }

    /// Get the display prefix for the command
    pub fn get_display_prefix(&self) -> Option<String> {
        match self {
            ExecutionMode::DryRunOnly => Some("DRY RUN: ".to_string()),
            ExecutionMode::Blocked(reason) => Some(format!("BLOCKED ({}): ", reason)),
            ExecutionMode::MultiStep(_) => Some("MULTI-STEP: ".to_string()),
            _ => None,
        }
    }
    
    /// Get the block reason if command is blocked
    pub fn get_block_reason(&self) -> Option<String> {
        match self {
            ExecutionMode::Blocked(reason) => Some(reason.clone()),
            _ => None,
        }
    }
}

/// Represents an executable command with metadata and execution context
#[derive(Debug, Clone)]
pub struct ExecutableCommand {
    pub command: String,
    pub explanation: String,
    pub execution_mode: ExecutionMode,
    pub warnings: Vec<String>,
}

impl ExecutableCommand {
    /// Create a new executable command
    pub fn new(command: String, explanation: String, execution_mode: ExecutionMode) -> Self {
        Self {
            command,
            explanation,
            execution_mode,
            warnings: Vec::new(),
        }
    }
    
    /// Add a warning to the command
    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }
    
    /// Get execution instructions for display
    pub fn get_execution_instructions(&self) -> Option<String> {
        match &self.execution_mode {
            ExecutionMode::SuggestOnly => {
                Some("Copy and paste the command above to execute it manually".to_string())
            }
            ExecutionMode::DryRunOnly => {
                Some("This is a dry run. Remove --dry-run flag to execute".to_string())
            }
            ExecutionMode::Blocked(reason) => {
                Some(format!("Command blocked: {}", reason))
            }
            ExecutionMode::RequiresConfirmation(_) => {
                Some("Use --auto-execute to run without confirmation".to_string())
            }
            ExecutionMode::MultiStep(_) => {
                Some("Multi-step command ready for execution".to_string())
            }
            ExecutionMode::Safe => None, // No instructions needed for safe execution
        }
    }
    
    /// Get the command explanation
    pub fn get_explanation(&self) -> &str {
        &self.explanation
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn test_multi_step_parsing() {
        // Test 1: Single command should return None
        let single_cmd = "ls -la";
        assert!(MultiStepHandler::parse_multi_step_command(single_cmd).is_none());

        // Test 2: Multi-step command should return Some
        let multi_cmd = "mkdir test-project\ncd test-project\ngit init\necho 'Hello World' > README.md\ngit add README.md\ngit commit -m 'Initial commit'";
        let handler = MultiStepHandler::parse_multi_step_command(multi_cmd);
        assert!(handler.is_some());
        
        let handler = handler.unwrap();
        assert_eq!(handler.total_steps, 6);
        assert_eq!(handler.current_step, 0);

        // Test 3: Empty command
        let empty_cmd = "";
        assert!(MultiStepHandler::parse_multi_step_command(empty_cmd).is_none());
        
        // Test 4: Comments should be filtered
        let commented_cmd = "# This is a comment\nmkdir test\n# Another comment\ncd test";
        let handler = MultiStepHandler::parse_multi_step_command(commented_cmd);
        assert!(handler.is_some());
        
        let handler = handler.unwrap();
        assert_eq!(handler.total_steps, 2); // Only non-comment lines
    }

    #[test]
    fn test_multi_step_progress() {
        let multi_cmd = "mkdir test\ncd test\ntouch file.txt";
        let mut handler = MultiStepHandler::parse_multi_step_command(multi_cmd).unwrap();
        
        // Initial state
        assert_eq!(handler.get_progress(), (0, 3));
        assert!(handler.has_more_steps());
        
        // Get first step
        let step = handler.get_next_step();
        assert!(step.is_some());
        assert_eq!(step.unwrap().command, "mkdir test");
        
        // Complete first step
        assert!(handler.complete_current_step(true));
        assert_eq!(handler.get_progress(), (1, 3));
        
        // Complete remaining steps
        assert!(handler.complete_current_step(true));
        assert_eq!(handler.get_progress(), (2, 3));
        
        assert!(handler.complete_current_step(true));
        assert_eq!(handler.get_progress(), (3, 3));
        
        // No more steps
        assert!(!handler.has_more_steps());
        assert!(!handler.complete_current_step(true));
    }

    #[test]
    fn test_execution_mode_multi_step() {
        let config = Config::load();
        
        // Test multi-step detection
        let multi_cmd = "mkdir test\ncd test\ntouch file.txt";
        let mode = ExecutionMode::determine_multi_step(&config, multi_cmd);
        
        match mode {
            ExecutionMode::MultiStep(steps) => {
                assert_eq!(steps.len(), 3);
            }
            _ => panic!("Expected MultiStep mode"),
        }
        
        // Test single command fallback
        let single_cmd = "ls -la";
        let mode = ExecutionMode::determine_multi_step(&config, single_cmd);
        assert_eq!(mode, ExecutionMode::SuggestOnly);
    }
}