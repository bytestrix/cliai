use regex::Regex;

/// Intent classification for user requests
#[derive(Debug, Clone, PartialEq)]
pub enum UserIntent {
    /// User wants to perform an action (execute commands)
    Actionable,
    /// User wants to learn how to do something (explanation only)
    Explanatory,
    /// Intent is unclear and needs clarification
    Ambiguous,
}

/// Result of intent classification
#[derive(Debug, Clone)]
pub struct IntentAnalysis {
    pub intent: UserIntent,
    pub confidence: f32,
    pub reasoning: String,
    pub clarification_needed: Option<String>,
}

/// Intent classifier that distinguishes between explanatory and actionable requests
pub struct IntentClassifier {
    /// Patterns that strongly indicate explanatory intent
    explanatory_patterns: Vec<Regex>,
    /// Patterns that strongly indicate actionable intent
    actionable_patterns: Vec<Regex>,
    /// Keywords that suggest explanatory intent
    explanatory_keywords: Vec<String>,
    /// Keywords that suggest actionable intent
    actionable_keywords: Vec<String>,
    /// Destructive action patterns that require explicit confirmation
    destructive_patterns: Vec<Regex>,
}

impl IntentClassifier {
    pub fn new() -> Self {
        let explanatory_patterns = vec![
            // Learning patterns - these are clearly explanatory
            Regex::new(
                "\\b(explain|teach me|help me understand|tutorial|guide|example|demonstration)\\b",
            )
            .unwrap(),
            Regex::new("\\b(what does .* do|how does .* work)\\b").unwrap(),
            Regex::new("\\b(what is|what are|what's the difference|difference between)\\b")
                .unwrap(),
            // Conceptual questions
            Regex::new("\\b(why does|why is|why would|purpose of|meaning of)\\b").unwrap(),
            // How-to questions that are clearly educational
            Regex::new("\\bhow to\\b").unwrap(),
            Regex::new("\\bhow do i\\b.*\\?").unwrap(),
            // Show me how to patterns (educational)
            Regex::new("\\bshow me how to\\b").unwrap(),
        ];

        let actionable_patterns = vec![
            // Direct commands
            Regex::new("^(create|make|build|generate|install|remove|delete|copy|move|rename)\\s").unwrap(),
            Regex::new("^(run|execute|start|stop|restart|kill)\\s").unwrap(),
            Regex::new("^(find|search|list|show|display)\\s").unwrap(),
            // Imperative mood
            Regex::new("^(please\\s+)?(do|go|get|set|put|add|update|upgrade|download)\\s").unwrap(),
            // Action requests
            Regex::new("\\b(i want to|i need to|can you|could you)\\s").unwrap(),
            // File operations
            Regex::new("\\b(file|directory|folder|script|program)\\b.*\\b(create|make|delete|remove|copy|move)\\b").unwrap(),
            // Information gathering that requires commands
            Regex::new("^(what|which|how much|how many|how big)\\s.*\\b(files|directory|disk|space|memory|cpu|process|version|size)\\b").unwrap(),
            Regex::new("\\b(show me|list|find)\\s.*\\b(files|directories|processes|version|status)\\b").unwrap(),
            // Status and information queries that need commands
            Regex::new("\\b(check|test|verify)\\s").unwrap(),
            Regex::new("\\b(what version|what files|what's running|what's in|what's the)\\b").unwrap(),
        ];

        let explanatory_keywords = vec![
            "how".to_string(),
            "what".to_string(),
            "why".to_string(),
            "explain".to_string(),
            "difference".to_string(),
            "meaning".to_string(),
            "purpose".to_string(),
            "understand".to_string(),
            "learn".to_string(),
            "tutorial".to_string(),
            "guide".to_string(),
            "example".to_string(),
            "demonstration".to_string(),
        ];

        let actionable_keywords = vec![
            "create".to_string(),
            "make".to_string(),
            "build".to_string(),
            "install".to_string(),
            "remove".to_string(),
            "delete".to_string(),
            "copy".to_string(),
            "move".to_string(),
            "rename".to_string(),
            "run".to_string(),
            "execute".to_string(),
            "start".to_string(),
            "stop".to_string(),
            "find".to_string(),
            "search".to_string(),
            "list".to_string(),
            "download".to_string(),
            "upload".to_string(),
            "show".to_string(),
            "display".to_string(),
            "check".to_string(),
            "test".to_string(),
            "verify".to_string(),
            "count".to_string(),
            "compress".to_string(),
            "backup".to_string(),
        ];

        let destructive_patterns = vec![
            // File deletion patterns
            Regex::new("\\b(delete|remove|rm)\\b.*\\b(all|everything|entire|whole)\\b").unwrap(),
            Regex::new("\\brm\\s+-rf?\\b").unwrap(),
            // System modification patterns
            Regex::new("\\b(format|wipe|erase|destroy)\\b").unwrap(),
            Regex::new("\\b(chmod|chown)\\s+.*\\b(recursive|777)\\b").unwrap(),
            // Database operations
            Regex::new("\\b(drop|truncate|delete)\\b.*\\b(database|table|all)\\b").unwrap(),
            // System reset patterns
            Regex::new("\\b(reset|restore|reinstall|factory)\\b").unwrap(),
        ];

        Self {
            explanatory_patterns,
            actionable_patterns,
            explanatory_keywords,
            actionable_keywords,
            destructive_patterns,
        }
    }

    /// Classify the intent of a user request
    pub fn classify_intent(&self, request: &str) -> IntentAnalysis {
        let normalized = request.to_lowercase().trim().to_string();

        // "Vague" requests should generally be treated as ambiguous, even if they contain an action verb.
        // Examples: "install something", "delete stuff", "do anything".
        let vague_indicators = ["something", "anything", "stuff", "things"];
        let is_vague = vague_indicators
            .iter()
            .any(|indicator| normalized.contains(indicator));

        // Very short / vague requests should be treated as ambiguous.
        // This avoids defaulting to Actionable for inputs like "files", "git", "python script".
        let word_count = normalized.split_whitespace().count();
        if word_count <= 2 && !normalized.ends_with('?') {
            // If it's not clearly actionable or explanatory, ask for clarification.
            // We still compute scores below; this is an early safe-guard for extreme brevity.
            let has_strong_explanatory = self
                .explanatory_patterns
                .iter()
                .any(|p| p.is_match(&normalized));
            let has_strong_actionable = self
                .actionable_patterns
                .iter()
                .any(|p| p.is_match(&normalized));
            if !has_strong_explanatory && !has_strong_actionable {
                return IntentAnalysis {
                    intent: UserIntent::Ambiguous,
                    confidence: 0.5,
                    reasoning: "Request is too short/vague to infer intent confidently".to_string(),
                    clarification_needed: Some(self.generate_clarification_prompt(&normalized)),
                };
            }
        }

        // Check for strong patterns first
        let explanatory_score = self.calculate_explanatory_score(&normalized);
        let actionable_score = self.calculate_actionable_score(&normalized);

        // If the user is vague, prefer clarification unless intent is extremely strong.
        if is_vague && explanatory_score < 0.7 && actionable_score < 0.7 {
            return IntentAnalysis {
                intent: UserIntent::Ambiguous,
                confidence: 0.5,
                reasoning: format!(
                    "Request is vague; asking clarification (explanatory: {:.2}, actionable: {:.2})",
                    explanatory_score, actionable_score
                ),
                clarification_needed: Some(self.generate_clarification_prompt(&normalized)),
            };
        }

        // Determine intent based on scores with a lower threshold for better detection
        let (intent, confidence, reasoning) =
            if explanatory_score > actionable_score + 0.2 && explanatory_score > 0.4 {
                (
                    UserIntent::Explanatory,
                    explanatory_score,
                    format!(
                        "Request contains explanatory patterns (score: {:.2})",
                        explanatory_score
                    ),
                )
            } else if actionable_score > explanatory_score + 0.1 && actionable_score > 0.2 {
                (
                    UserIntent::Actionable,
                    actionable_score,
                    format!(
                        "Request contains actionable patterns (score: {:.2})",
                        actionable_score
                    ),
                )
            } else {
                // Default to actionable for ambiguous cases unless clearly explanatory
                if explanatory_score > 0.3 {
                    (
                        UserIntent::Explanatory,
                        explanatory_score,
                        format!(
                            "Defaulting to explanatory due to patterns (score: {:.2})",
                            explanatory_score
                        ),
                    )
                } else {
                    (
                        UserIntent::Actionable,
                        0.5,
                        format!(
                            "Defaulting to actionable - explanatory: {:.2}, actionable: {:.2}",
                            explanatory_score, actionable_score
                        ),
                    )
                }
            };

        // Check for clarification needs
        let clarification_needed = if intent == UserIntent::Ambiguous {
            Some(self.generate_clarification_prompt(&normalized))
        } else {
            None
        };

        IntentAnalysis {
            intent,
            confidence,
            reasoning,
            clarification_needed,
        }
    }

    /// Check if a request involves destructive actions
    pub fn is_destructive_action(&self, request: &str) -> bool {
        let normalized = request.to_lowercase();

        for pattern in &self.destructive_patterns {
            if pattern.is_match(&normalized) {
                return true;
            }
        }

        false
    }

    /// Generate a clarification prompt for ambiguous requests
    fn generate_clarification_prompt(&self, request: &str) -> String {
        if request.contains("file") || request.contains("directory") {
            "Do you want me to:\n1. Explain how to work with files/directories, or\n2. Actually create/modify files/directories?".to_string()
        } else if request.contains("install") || request.contains("setup") {
            "Do you want me to:\n1. Explain how to install/setup something, or\n2. Actually perform the installation/setup?".to_string()
        } else if request.contains("delete") || request.contains("remove") {
            "Do you want me to:\n1. Explain how deletion/removal works, or\n2. Actually delete/remove something?\n\n⚠️  If you want actual deletion, please be specific about what to delete.".to_string()
        } else {
            "Do you want me to:\n1. Explain how to do this, or\n2. Actually perform the action?"
                .to_string()
        }
    }

    /// Calculate explanatory intent score
    fn calculate_explanatory_score(&self, request: &str) -> f32 {
        let mut score: f32 = 0.0;

        // Check patterns (high weight)
        for pattern in &self.explanatory_patterns {
            if pattern.is_match(request) {
                score += 0.5; // Increased weight for patterns
            }
        }

        // Check keywords (medium weight)
        for keyword in &self.explanatory_keywords {
            if request.contains(keyword) {
                score += 0.3; // Increased weight for keywords
            }
        }

        // Question marks are strong indicators
        if request.ends_with('?') {
            score += 0.4; // Increased weight for question marks
        }

        // Normalize score to 0-1 range
        score.min(1.0)
    }

    /// Calculate actionable intent score
    fn calculate_actionable_score(&self, request: &str) -> f32 {
        let mut score: f32 = 0.0;

        // Check for vague requests first - these should be ambiguous
        let vague_indicators = ["something", "anything", "stuff", "things"];
        let is_vague = vague_indicators
            .iter()
            .any(|indicator| request.contains(indicator));

        // Check patterns (high weight)
        for pattern in &self.actionable_patterns {
            if pattern.is_match(request) {
                if is_vague {
                    score += 0.2; // Much lower weight for vague requests
                } else {
                    score += 0.6; // Increased weight for specific requests
                }
            }
        }

        // Check keywords (medium weight) - but be more selective
        for keyword in &self.actionable_keywords {
            if request.contains(keyword) {
                if is_vague {
                    score += 0.1; // Much lower weight for vague requests
                } else {
                    score += 0.4; // Increased weight for specific requests
                }
            }
        }

        // Imperative mood indicators
        if request.starts_with("please") || request.contains("i want") || request.contains("i need")
        {
            if is_vague {
                score += 0.1; // Lower weight for vague requests
            } else {
                score += 0.3; // Increased weight
            }
        }

        // Direct action verbs at the start get high score
        let action_starters = [
            "create", "make", "build", "install", "remove", "delete", "copy", "move", "rename",
            "run", "execute", "start", "stop", "find", "search", "list", "show", "display",
        ];
        let first_word = request.split_whitespace().next().unwrap_or("");
        if action_starters.contains(&first_word) {
            if is_vague {
                score += 0.2; // Lower weight when the request is vague ("install something")
            } else {
                score += 0.5; // High score for direct action verbs
            }
        }

        // Normalize score to 0-1 range
        score.min(1.0)
    }

    /// Validate that a command is appropriate for the detected intent
    pub fn validate_command_for_intent(
        &self,
        command: &str,
        intent: &UserIntent,
    ) -> Result<(), String> {
        match intent {
            UserIntent::Explanatory => {
                // For explanatory requests, commands should be safe and non-destructive
                if self.is_destructive_command(command) {
                    return Err(
                        "Destructive commands are not appropriate for explanatory requests"
                            .to_string(),
                    );
                }

                // Commands should be informational or demonstrative
                if !self.is_informational_command(command) {
                    return Err(
                        "Command should be informational for explanatory requests".to_string()
                    );
                }
            }
            UserIntent::Actionable => {
                // For actionable requests, any valid command is acceptable
                // but destructive commands should be flagged for confirmation
                if self.is_destructive_command(command) {
                    return Err("Destructive command requires explicit confirmation".to_string());
                }
            }
            UserIntent::Ambiguous => {
                // For ambiguous requests, prefer safe commands
                if self.is_destructive_command(command) {
                    return Err(
                        "Cannot suggest destructive commands for ambiguous requests".to_string()
                    );
                }
            }
        }

        Ok(())
    }

    /// Check if a command is destructive
    fn is_destructive_command(&self, command: &str) -> bool {
        let destructive_commands = [
            "rm -rf",
            "rm -r",
            "rmdir",
            "delete",
            "format",
            "mkfs",
            "dd if=",
            "fdisk",
            "chmod 777",
            "chown -R",
            "> /dev/",
            "truncate",
            "shred",
        ];

        let normalized = command.to_lowercase();
        destructive_commands
            .iter()
            .any(|cmd| normalized.contains(cmd))
    }

    /// Check if a command is informational (safe for explanatory purposes)
    fn is_informational_command(&self, command: &str) -> bool {
        let informational_commands = [
            "ls",
            "cat",
            "head",
            "tail",
            "grep",
            "find",
            "which",
            "whereis",
            "ps",
            "top",
            "df",
            "du",
            "free",
            "uname",
            "whoami",
            "id",
            "pwd",
            "git",
            "stat",
            "file",
            "wc",
            "sort",
            "uniq",
            "awk",
            "sed",
            "echo",
            "printf",
            "date",
            "cal",
            "uptime",
            "lsof",
            "netstat",
            "ss",
            "systemctl",
            "service",
            "rustc",
            "cargo",
            "node",
            "npm",
            "python",
            "java",
            "javac",
            "gcc",
            "make",
            "cmake",
            "zip",
            "tar",
            "gzip",
        ];

        let first_word = command.split_whitespace().next().unwrap_or("");

        // Allow informational commands
        if informational_commands.iter().any(|cmd| first_word == *cmd) {
            return true;
        }

        // Allow compound commands with informational parts
        if command.contains(" | ") || command.contains(" && ") || command.contains(" || ") {
            // Check if all parts are informational
            let parts: Vec<&str> = command.split(&[' ', '|', '&'][..]).collect();
            return parts.iter().any(|part| {
                let trimmed = part.trim();
                informational_commands
                    .iter()
                    .any(|cmd| trimmed.starts_with(cmd))
            });
        }

        false
    }
}

impl Default for IntentClassifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_explanatory_intent_detection() {
        let classifier = IntentClassifier::new();

        let explanatory_requests = vec![
            "How do I create a file?",
            "What does ls -la do?",
            "Explain the difference between rm and rmdir",
            "Show me how to use grep",
            "What is the purpose of chmod?",
            "Why does this command fail?",
            "Can you explain how pipes work?",
            "What are the options for find command?",
        ];

        for request in explanatory_requests {
            let analysis = classifier.classify_intent(request);
            assert_eq!(
                analysis.intent,
                UserIntent::Explanatory,
                "Request '{}' should be classified as explanatory",
                request
            );
        }
    }

    #[test]
    fn test_actionable_intent_detection() {
        let classifier = IntentClassifier::new();

        let actionable_requests = vec![
            "Create a file named test.txt",
            "Install vim",
            "Delete the old logs",
            "Find all .rs files",
            "List all running processes",
            "Copy file.txt to backup.txt",
            "I want to create a directory",
            "Please install the package",
            "Run the build script",
            "Start the web server",
        ];

        for request in actionable_requests {
            let analysis = classifier.classify_intent(request);
            assert_eq!(
                analysis.intent,
                UserIntent::Actionable,
                "Request '{}' should be classified as actionable",
                request
            );
        }
    }

    #[test]
    fn test_ambiguous_intent_detection() {
        let classifier = IntentClassifier::new();

        let ambiguous_requests = vec![
            "files",
            "git",
            "python script",
            "database connection",
            "server configuration",
        ];

        for request in ambiguous_requests {
            let analysis = classifier.classify_intent(request);
            assert_eq!(
                analysis.intent,
                UserIntent::Ambiguous,
                "Request '{}' should be classified as ambiguous",
                request
            );
            assert!(analysis.clarification_needed.is_some());
        }
    }

    #[test]
    fn test_destructive_action_detection() {
        let classifier = IntentClassifier::new();

        let destructive_requests = vec![
            "Delete all files",
            "Remove everything in the directory",
            "Format the disk",
            "rm -rf /tmp/*",
            "chmod 777 recursively",
            "Drop the database",
            "Factory reset the system",
        ];

        for request in destructive_requests {
            assert!(
                classifier.is_destructive_action(request),
                "Request '{}' should be detected as destructive",
                request
            );
        }
    }

    #[test]
    fn test_command_validation_for_intent() {
        let classifier = IntentClassifier::new();

        // Explanatory intent should reject destructive commands
        let result =
            classifier.validate_command_for_intent("rm -rf /tmp", &UserIntent::Explanatory);
        assert!(result.is_err());

        // Explanatory intent should accept informational commands
        let result = classifier.validate_command_for_intent("ls -la", &UserIntent::Explanatory);
        assert!(result.is_ok());

        // Actionable intent should flag destructive commands
        let result = classifier.validate_command_for_intent("rm -rf /tmp", &UserIntent::Actionable);
        assert!(result.is_err());

        // Actionable intent should accept safe commands
        let result = classifier.validate_command_for_intent("mkdir test", &UserIntent::Actionable);
        assert!(result.is_ok());
    }

    #[test]
    fn test_informational_command_detection() {
        let classifier = IntentClassifier::new();

        let informational_commands = vec![
            "ls -la",
            "cat file.txt",
            "grep pattern file.txt",
            "find . -name '*.rs'",
            "ps aux",
            "df -h",
            "git status",
        ];

        for cmd in informational_commands {
            assert!(
                classifier.is_informational_command(cmd),
                "Command '{}' should be informational",
                cmd
            );
        }

        let non_informational_commands = vec![
            "rm file.txt",
            "mkdir test",
            "cp file1 file2",
            "chmod 755 file",
        ];

        for cmd in non_informational_commands {
            assert!(
                !classifier.is_informational_command(cmd),
                "Command '{}' should not be informational",
                cmd
            );
        }
    }

    #[test]
    fn test_destructive_command_detection() {
        let classifier = IntentClassifier::new();

        let destructive_commands = vec![
            "rm -rf /tmp",
            "rm -r directory",
            "chmod 777 file",
            "dd if=/dev/zero of=/dev/sda",
            "mkfs.ext4 /dev/sdb1",
        ];

        for cmd in destructive_commands {
            assert!(
                classifier.is_destructive_command(cmd),
                "Command '{}' should be destructive",
                cmd
            );
        }

        let safe_commands = vec![
            "ls -la",
            "cat file.txt",
            "mkdir test",
            "cp file1 file2",
            "chmod 644 file",
        ];

        for cmd in safe_commands {
            assert!(
                !classifier.is_destructive_command(cmd),
                "Command '{}' should not be destructive",
                cmd
            );
        }
    }

    #[test]
    fn test_clarification_prompt_generation() {
        let classifier = IntentClassifier::new();

        let analysis = classifier.classify_intent("file operations");
        assert_eq!(analysis.intent, UserIntent::Ambiguous);
        assert!(analysis.clarification_needed.is_some());
        assert!(analysis
            .clarification_needed
            .unwrap()
            .contains("files/directories"));

        let analysis = classifier.classify_intent("install something");
        assert_eq!(analysis.intent, UserIntent::Ambiguous);
        assert!(analysis.clarification_needed.is_some());
        assert!(analysis
            .clarification_needed
            .unwrap()
            .contains("install/setup"));
    }

    #[test]
    fn test_confidence_scoring() {
        let classifier = IntentClassifier::new();

        // High confidence explanatory
        let analysis = classifier.classify_intent("How do I create a file?");
        assert_eq!(analysis.intent, UserIntent::Explanatory);
        assert!(analysis.confidence > 0.7);

        // High confidence actionable
        let analysis = classifier.classify_intent("Create a file named test.txt");
        assert_eq!(analysis.intent, UserIntent::Actionable);
        assert!(analysis.confidence > 0.7);

        // Low confidence (ambiguous)
        let analysis = classifier.classify_intent("files");
        assert_eq!(analysis.intent, UserIntent::Ambiguous);
        assert!(analysis.confidence <= 0.6);
    }
}
