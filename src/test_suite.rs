use regex::Regex;
use std::collections::HashMap;
use std::process::Command;
use std::time::{Duration, Instant};
use anyhow::{Result, anyhow};
use colored::*;
use crate::agents::Orchestrator;
use crate::config::Config;
use crate::history::History;
use chrono;

/// Comprehensive test suite for validating CLIAI command generation
pub struct TestSuite {
    test_questions: Vec<TestQuestion>,
    expected_patterns: HashMap<usize, Vec<ExpectedPattern>>,
    hallucinated_flags: Vec<String>,
    safe_commands: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct TestQuestion {
    pub id: usize,
    pub category: TestCategory,
    pub question: String,
    pub should_have_command: bool,
    pub is_safe_to_execute: bool,
    pub expected_command_type: Option<CommandType>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TestCategory {
    FileManagement,
    SystemInfo,
    GitOperations,
    Network,
    Programming,
    ProcessManagement,
    General,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CommandType {
    FileOperation,
    SystemQuery,
    NetworkCommand,
    GitCommand,
    ProcessCommand,
    Explanation,
}

#[derive(Debug, Clone)]
pub struct ExpectedPattern {
    pub pattern: Regex,
    pub description: String,
    pub is_required: bool,
}

#[derive(Debug, Clone)]
pub struct TestResult {
    pub question_id: usize,
    pub question: String,
    pub ai_response: String,
    pub extracted_command: Option<String>,
    pub execution_time_ms: u64,
    pub status: TestStatus,
    pub pattern_matches: Vec<PatternMatch>,
    pub hallucinated_flags_found: Vec<String>,
    pub execution_result: Option<ExecutionResult>,
    pub failure_details: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TestStatus {
    Passed,
    Failed,
    PartialSuccess,
    NotExecuted,
}

#[derive(Debug, Clone)]
pub struct PatternMatch {
    pub pattern_description: String,
    pub matched: bool,
    pub is_required: bool,
}

#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
    pub exit_code: i32,
}

impl TestSuite {
    pub fn new() -> Self {
        let mut suite = Self {
            test_questions: Vec::new(),
            expected_patterns: HashMap::new(),
            hallucinated_flags: Self::get_known_hallucinated_flags(),
            safe_commands: Self::get_safe_commands(),
        };
        
        suite.initialize_test_questions();
        suite.initialize_expected_patterns();
        suite
    }

    /// Initialize the 50 test questions based on the existing test script
    fn initialize_test_questions(&mut self) {
        // File Management (Questions 1-10)
        self.test_questions.extend(vec![
            TestQuestion {
                id: 1,
                category: TestCategory::FileManagement,
                question: "How do I list all files including hidden ones?".to_string(),
                should_have_command: true,
                is_safe_to_execute: true,
                expected_command_type: Some(CommandType::FileOperation),
            },
            TestQuestion {
                id: 2,
                category: TestCategory::FileManagement,
                question: "Create a directory called test_project".to_string(),
                should_have_command: true,
                is_safe_to_execute: true,
                expected_command_type: Some(CommandType::FileOperation),
            },
            TestQuestion {
                id: 3,
                category: TestCategory::FileManagement,
                question: "How can I find all .rs files in this folder?".to_string(),
                should_have_command: true,
                is_safe_to_execute: true,
                expected_command_type: Some(CommandType::FileOperation),
            },
            TestQuestion {
                id: 4,
                category: TestCategory::FileManagement,
                question: "Show me how to check if old.txt exists".to_string(),
                should_have_command: true,
                is_safe_to_execute: true,
                expected_command_type: Some(CommandType::FileOperation),
            },
            TestQuestion {
                id: 5,
                category: TestCategory::FileManagement,
                question: "Copy all contents of folder A to folder B".to_string(),
                should_have_command: true,
                is_safe_to_execute: false, // Uses placeholder paths
                expected_command_type: Some(CommandType::FileOperation),
            },
            TestQuestion {
                id: 6,
                category: TestCategory::FileManagement,
                question: "Count how many lines are in main.rs".to_string(),
                should_have_command: true,
                is_safe_to_execute: true,
                expected_command_type: Some(CommandType::FileOperation),
            },
            TestQuestion {
                id: 7,
                category: TestCategory::FileManagement,
                question: "Rename temp.js to app.js".to_string(),
                should_have_command: true,
                is_safe_to_execute: false, // File doesn't exist
                expected_command_type: Some(CommandType::FileOperation),
            },
            TestQuestion {
                id: 8,
                category: TestCategory::FileManagement,
                question: "Show the last 20 lines of the system log".to_string(),
                should_have_command: true,
                is_safe_to_execute: true,
                expected_command_type: Some(CommandType::FileOperation),
            },
            TestQuestion {
                id: 9,
                category: TestCategory::FileManagement,
                question: "Create a file named config.yaml with version 1.0 in it".to_string(),
                should_have_command: true,
                is_safe_to_execute: true,
                expected_command_type: Some(CommandType::FileOperation),
            },
            TestQuestion {
                id: 10,
                category: TestCategory::FileManagement,
                question: "What is the size of the current directory?".to_string(),
                should_have_command: true,
                is_safe_to_execute: true,
                expected_command_type: Some(CommandType::SystemQuery),
            },
        ]);

        // System Info (Questions 11-15)
        self.test_questions.extend(vec![
            TestQuestion {
                id: 11,
                category: TestCategory::SystemInfo,
                question: "How much free RAM do I have?".to_string(),
                should_have_command: true,
                is_safe_to_execute: true,
                expected_command_type: Some(CommandType::SystemQuery),
            },
            TestQuestion {
                id: 12,
                category: TestCategory::SystemInfo,
                question: "Show my current CPU usage".to_string(),
                should_have_command: true,
                is_safe_to_execute: true,
                expected_command_type: Some(CommandType::SystemQuery),
            },
            TestQuestion {
                id: 13,
                category: TestCategory::SystemInfo,
                question: "What is my hostname?".to_string(),
                should_have_command: true,
                is_safe_to_execute: true,
                expected_command_type: Some(CommandType::SystemQuery),
            },
            TestQuestion {
                id: 14,
                category: TestCategory::SystemInfo,
                question: "Show kernel version".to_string(),
                should_have_command: true,
                is_safe_to_execute: true,
                expected_command_type: Some(CommandType::SystemQuery),
            },
            TestQuestion {
                id: 15,
                category: TestCategory::SystemInfo,
                question: "List all block devices".to_string(),
                should_have_command: true,
                is_safe_to_execute: true,
                expected_command_type: Some(CommandType::SystemQuery),
            },
        ]);

        // Git Operations (Questions 16-20)
        self.test_questions.extend(vec![
            TestQuestion {
                id: 16,
                category: TestCategory::GitOperations,
                question: "What is the current git status?".to_string(),
                should_have_command: true,
                is_safe_to_execute: true,
                expected_command_type: Some(CommandType::GitCommand),
            },
            TestQuestion {
                id: 17,
                category: TestCategory::GitOperations,
                question: "Show me how to commit all changes with message initial commit".to_string(),
                should_have_command: true,
                is_safe_to_execute: false, // Don't execute git commits
                expected_command_type: Some(CommandType::GitCommand),
            },
            TestQuestion {
                id: 18,
                category: TestCategory::GitOperations,
                question: "Show the last 3 commits".to_string(),
                should_have_command: true,
                is_safe_to_execute: true,
                expected_command_type: Some(CommandType::GitCommand),
            },
            TestQuestion {
                id: 19,
                category: TestCategory::GitOperations,
                question: "How do I create a new branch called feature/ai?".to_string(),
                should_have_command: true,
                is_safe_to_execute: false, // Don't execute git branch creation
                expected_command_type: Some(CommandType::GitCommand),
            },
            TestQuestion {
                id: 20,
                category: TestCategory::GitOperations,
                question: "How do I merge main into the current branch?".to_string(),
                should_have_command: true,
                is_safe_to_execute: false, // Don't execute git merge
                expected_command_type: Some(CommandType::GitCommand),
            },
        ]);

        // Network (Questions 21-25)
        self.test_questions.extend(vec![
            TestQuestion {
                id: 21,
                category: TestCategory::Network,
                question: "What is my IP address?".to_string(),
                should_have_command: true,
                is_safe_to_execute: true,
                expected_command_type: Some(CommandType::NetworkCommand),
            },
            TestQuestion {
                id: 22,
                category: TestCategory::Network,
                question: "Ping google.com 4 times".to_string(),
                should_have_command: true,
                is_safe_to_execute: true,
                expected_command_type: Some(CommandType::NetworkCommand),
            },
            TestQuestion {
                id: 23,
                category: TestCategory::Network,
                question: "Which ports are open on my machine?".to_string(),
                should_have_command: true,
                is_safe_to_execute: true,
                expected_command_type: Some(CommandType::NetworkCommand),
            },
            TestQuestion {
                id: 24,
                category: TestCategory::Network,
                question: "How do I download a file from a URL?".to_string(),
                should_have_command: true,
                is_safe_to_execute: false, // Don't execute downloads
                expected_command_type: Some(CommandType::NetworkCommand),
            },
            TestQuestion {
                id: 25,
                category: TestCategory::Network,
                question: "Show my network connections".to_string(),
                should_have_command: true,
                is_safe_to_execute: true,
                expected_command_type: Some(CommandType::NetworkCommand),
            },
        ]);

        // Programming & Code (Questions 26-35)
        self.test_questions.extend(vec![
            TestQuestion {
                id: 26,
                category: TestCategory::Programming,
                question: "Write a python script that prints Hello World".to_string(),
                should_have_command: false, // Code generation, not shell command
                is_safe_to_execute: false,
                expected_command_type: Some(CommandType::Explanation),
            },
            TestQuestion {
                id: 27,
                category: TestCategory::Programming,
                question: "Explain what async/await does in Rust".to_string(),
                should_have_command: false, // Explanation, not shell command
                is_safe_to_execute: false,
                expected_command_type: Some(CommandType::Explanation),
            },
            TestQuestion {
                id: 28,
                category: TestCategory::Programming,
                question: "How do I install the requests library in Python?".to_string(),
                should_have_command: true,
                is_safe_to_execute: false, // Don't execute package installs
                expected_command_type: Some(CommandType::SystemQuery),
            },
            TestQuestion {
                id: 29,
                category: TestCategory::Programming,
                question: "What is the difference between let and const in JavaScript?".to_string(),
                should_have_command: false, // Explanation, not shell command
                is_safe_to_execute: false,
                expected_command_type: Some(CommandType::Explanation),
            },
            TestQuestion {
                id: 30,
                category: TestCategory::General,
                question: "What's the regex for matching an email address?".to_string(),
                should_have_command: false, // General knowledge
                is_safe_to_execute: false,
                expected_command_type: Some(CommandType::Explanation),
            },
            TestQuestion {
                id: 31,
                category: TestCategory::Programming,
                question: "How do I parse JSON in Python?".to_string(),
                should_have_command: false, // Code explanation
                is_safe_to_execute: false,
                expected_command_type: Some(CommandType::Explanation),
            },
            TestQuestion {
                id: 32,
                category: TestCategory::Programming,
                question: "Show me a basic Cargo.toml structure".to_string(),
                should_have_command: false, // Code example
                is_safe_to_execute: false,
                expected_command_type: Some(CommandType::Explanation),
            },
            TestQuestion {
                id: 33,
                category: TestCategory::Programming,
                question: "How do I use map in Javascript?".to_string(),
                should_have_command: false, // Code explanation
                is_safe_to_execute: false,
                expected_command_type: Some(CommandType::Explanation),
            },
            TestQuestion {
                id: 34,
                category: TestCategory::General,
                question: "How do I debug a Segmentation Fault error?".to_string(),
                should_have_command: false, // Troubleshooting advice
                is_safe_to_execute: false,
                expected_command_type: Some(CommandType::Explanation),
            },
            TestQuestion {
                id: 35,
                category: TestCategory::FileManagement,
                question: "Write a bash script to backup my docs folder".to_string(),
                should_have_command: true,
                is_safe_to_execute: false, // Don't execute backup scripts
                expected_command_type: Some(CommandType::FileOperation),
            },
        ]);

        // Process Management (Questions 36-40)
        self.test_questions.extend(vec![
            TestQuestion {
                id: 36,
                category: TestCategory::ProcessManagement,
                question: "List all running processes".to_string(),
                should_have_command: true,
                is_safe_to_execute: true,
                expected_command_type: Some(CommandType::ProcessCommand),
            },
            TestQuestion {
                id: 37,
                category: TestCategory::ProcessManagement,
                question: "How do I kill a process with PID 1234?".to_string(),
                should_have_command: true,
                is_safe_to_execute: false, // Don't execute kill commands
                expected_command_type: Some(CommandType::ProcessCommand),
            },
            TestQuestion {
                id: 38,
                category: TestCategory::ProcessManagement,
                question: "Find the PID of node".to_string(),
                should_have_command: true,
                is_safe_to_execute: true,
                expected_command_type: Some(CommandType::ProcessCommand),
            },
            TestQuestion {
                id: 39,
                category: TestCategory::ProcessManagement,
                question: "How do I see which process is using port 8080?".to_string(),
                should_have_command: true,
                is_safe_to_execute: true,
                expected_command_type: Some(CommandType::ProcessCommand),
            },
            TestQuestion {
                id: 40,
                category: TestCategory::SystemInfo,
                question: "Show system uptime".to_string(),
                should_have_command: true,
                is_safe_to_execute: true,
                expected_command_type: Some(CommandType::SystemQuery),
            },
        ]);

        // General & Context Memory (Questions 41-50)
        self.test_questions.extend(vec![
            TestQuestion {
                id: 41,
                category: TestCategory::General,
                question: "What did I ask you first?".to_string(),
                should_have_command: false, // Context/memory question
                is_safe_to_execute: false,
                expected_command_type: Some(CommandType::Explanation),
            },
            TestQuestion {
                id: 42,
                category: TestCategory::General,
                question: "Who are you?".to_string(),
                should_have_command: false, // Identity question
                is_safe_to_execute: false,
                expected_command_type: Some(CommandType::Explanation),
            },
            TestQuestion {
                id: 43,
                category: TestCategory::General,
                question: "How can I check the weather from terminal?".to_string(),
                should_have_command: true, // General advice but might include command
                is_safe_to_execute: false,
                expected_command_type: Some(CommandType::NetworkCommand),
            },
            TestQuestion {
                id: 44,
                category: TestCategory::General,
                question: "Tell me a programming joke".to_string(),
                should_have_command: false, // Entertainment
                is_safe_to_execute: false,
                expected_command_type: Some(CommandType::Explanation),
            },
            TestQuestion {
                id: 45,
                category: TestCategory::General,
                question: "Remember that my project name is Alpha".to_string(),
                should_have_command: false, // Memory instruction
                is_safe_to_execute: false,
                expected_command_type: Some(CommandType::Explanation),
            },
            TestQuestion {
                id: 46,
                category: TestCategory::General,
                question: "What was my project name?".to_string(),
                should_have_command: false, // Memory recall
                is_safe_to_execute: false,
                expected_command_type: Some(CommandType::Explanation),
            },
            TestQuestion {
                id: 47,
                category: TestCategory::General,
                question: "How do I exit the terminal?".to_string(),
                should_have_command: true,
                is_safe_to_execute: false, // Don't execute exit commands
                expected_command_type: Some(CommandType::SystemQuery),
            },
            TestQuestion {
                id: 48,
                category: TestCategory::General,
                question: "What is 2 plus 2?".to_string(),
                should_have_command: false, // Math question
                is_safe_to_execute: false,
                expected_command_type: Some(CommandType::Explanation),
            },
            TestQuestion {
                id: 49,
                category: TestCategory::General,
                question: "Help me with this error: Permission denied".to_string(),
                should_have_command: false, // Error analysis
                is_safe_to_execute: false,
                expected_command_type: Some(CommandType::Explanation),
            },
            TestQuestion {
                id: 50,
                category: TestCategory::General,
                question: "How do I clear our conversation?".to_string(),
                should_have_command: true, // System command
                is_safe_to_execute: false,
                expected_command_type: Some(CommandType::SystemQuery),
            },
        ]);
    }

    /// Initialize expected patterns for command validation
    fn initialize_expected_patterns(&mut self) {
        // File Management patterns
        self.expected_patterns.insert(1, vec![
            ExpectedPattern {
                pattern: Regex::new(r"ls\s+(-[la]+|--all)").unwrap(),
                description: "List command with all/hidden flag".to_string(),
                is_required: true,
            },
        ]);

        self.expected_patterns.insert(2, vec![
            ExpectedPattern {
                pattern: Regex::new(r"mkdir\s+test_project").unwrap(),
                description: "Create directory command".to_string(),
                is_required: true,
            },
        ]);

        self.expected_patterns.insert(3, vec![
            ExpectedPattern {
                pattern: Regex::new(r"find\s+.*\.rs").unwrap(),
                description: "Find command for .rs files".to_string(),
                is_required: true,
            },
        ]);

        self.expected_patterns.insert(4, vec![
            ExpectedPattern {
                pattern: Regex::new(r"test\s+-f\s+old\.txt").unwrap(),
                description: "File existence test command".to_string(),
                is_required: true,
            },
        ]);

        self.expected_patterns.insert(6, vec![
            ExpectedPattern {
                pattern: Regex::new(r"wc\s+-l\s+main\.rs").unwrap(),
                description: "Line count command".to_string(),
                is_required: true,
            },
        ]);

        // System Info patterns
        self.expected_patterns.insert(11, vec![
            ExpectedPattern {
                pattern: Regex::new(r"free\s+(-h|--human-readable)").unwrap(),
                description: "Memory usage command".to_string(),
                is_required: true,
            },
        ]);

        self.expected_patterns.insert(13, vec![
            ExpectedPattern {
                pattern: Regex::new(r"hostname").unwrap(),
                description: "Hostname command".to_string(),
                is_required: true,
            },
        ]);

        self.expected_patterns.insert(14, vec![
            ExpectedPattern {
                pattern: Regex::new(r"uname\s+(-r|-a)").unwrap(),
                description: "Kernel version command".to_string(),
                is_required: true,
            },
        ]);

        // Git patterns
        self.expected_patterns.insert(16, vec![
            ExpectedPattern {
                pattern: Regex::new(r"git\s+status").unwrap(),
                description: "Git status command".to_string(),
                is_required: true,
            },
        ]);

        self.expected_patterns.insert(17, vec![
            ExpectedPattern {
                pattern: Regex::new(r"git\s+commit.*-m.*initial commit").unwrap(),
                description: "Git commit command with message".to_string(),
                is_required: true,
            },
        ]);

        // Network patterns
        self.expected_patterns.insert(21, vec![
            ExpectedPattern {
                pattern: Regex::new(r"(ip\s+addr|ifconfig|hostname\s+-I)").unwrap(),
                description: "IP address command".to_string(),
                is_required: true,
            },
        ]);

        self.expected_patterns.insert(22, vec![
            ExpectedPattern {
                pattern: Regex::new(r"ping\s+(-c\s+4|--count=4).*google\.com").unwrap(),
                description: "Ping command with count".to_string(),
                is_required: true,
            },
        ]);

        // Process Management patterns
        self.expected_patterns.insert(36, vec![
            ExpectedPattern {
                pattern: Regex::new(r"ps\s+(aux|ef)").unwrap(),
                description: "Process list command".to_string(),
                is_required: true,
            },
        ]);

        self.expected_patterns.insert(37, vec![
            ExpectedPattern {
                pattern: Regex::new(r"kill\s+(-9\s+)?1234").unwrap(),
                description: "Kill process command".to_string(),
                is_required: true,
            },
        ]);
    }

    /// Get known hallucinated flags that should be detected
    fn get_known_hallucinated_flags() -> Vec<String> {
        vec![
            "--hidden".to_string(),
            "--recursivee".to_string(),
            "--all-files".to_string(),
            "--show-hidden".to_string(),
            "--include-hidden".to_string(),
            "--list-all".to_string(),
            "--verbose-output".to_string(),
            "--detailed".to_string(),
            "--full-info".to_string(),
            "--complete".to_string(),
            "--extended".to_string(),
            "--comprehensive".to_string(),
            "--show-all".to_string(),
            "--display-all".to_string(),
            "--include-all".to_string(),
        ]
    }

    /// Get list of commands that are safe to execute
    fn get_safe_commands() -> Vec<String> {
        vec![
            "ls".to_string(),
            "pwd".to_string(),
            "whoami".to_string(),
            "hostname".to_string(),
            "uname".to_string(),
            "date".to_string(),
            "uptime".to_string(),
            "free".to_string(),
            "df".to_string(),
            "ps".to_string(),
            "top".to_string(),
            "cat".to_string(),
            "head".to_string(),
            "tail".to_string(),
            "wc".to_string(),
            "find".to_string(),
            "grep".to_string(),
            "which".to_string(),
            "whereis".to_string(),
            "file".to_string(),
            "stat".to_string(),
            "test".to_string(),
            "echo".to_string(),
            "git status".to_string(),
            "git log".to_string(),
            "git branch".to_string(),
            "git diff".to_string(),
        ]
    }

    /// Extract command from AI response
    pub fn extract_command(&self, response: &str) -> Option<String> {
        // Method 1: Look for "Command:" prefix
        if let Some(cmd_start) = response.find("Command: ") {
            let after_prefix = &response[cmd_start + 9..];
            if let Some(newline_pos) = after_prefix.find('\n') {
                let cmd = after_prefix[..newline_pos].trim();
                if cmd != "(none)" && !cmd.is_empty() {
                    return Some(self.clean_command(cmd));
                }
            } else {
                let cmd = after_prefix.trim();
                if cmd != "(none)" && !cmd.is_empty() {
                    return Some(self.clean_command(cmd));
                }
            }
        }

        // Method 2: Look for backtick-wrapped commands
        let backtick_pattern = Regex::new(r"`([^`]+)`").unwrap();
        for captures in backtick_pattern.captures_iter(response) {
            if let Some(cmd) = captures.get(1) {
                let potential_cmd = cmd.as_str().trim();
                if self.looks_like_shell_command(potential_cmd) {
                    return Some(self.clean_command(potential_cmd));
                }
            }
        }

        // Method 3: Look for lines that start with common commands
        for line in response.lines() {
            let trimmed = line.trim();
            if self.looks_like_shell_command(trimmed) && !self.looks_like_sentence(trimmed) {
                return Some(self.clean_command(trimmed));
            }
        }

        None
    }

    /// Clean command of formatting artifacts
    fn clean_command(&self, cmd: &str) -> String {
        let mut result = cmd.trim().to_string();
        
        // Remove common prefixes
        result = result.trim_start_matches("$ ").to_string();
        result = result.trim_start_matches("> ").to_string();
        
        // Remove backticks
        result = result.trim_matches('`').to_string();
        
        // Remove markdown formatting
        result = result.replace("**", "");
        result = result.replace("__", "");
        
        result.trim().to_string()
    }

    /// Check if text looks like a shell command
    fn looks_like_shell_command(&self, text: &str) -> bool {
        let first_word = text.split_whitespace().next().unwrap_or("");
        
        // Common shell commands
        let shell_commands = vec![
            "ls", "find", "mkdir", "cat", "grep", "cp", "mv", "rm", "chmod", "ps", "kill",
            "git", "ping", "curl", "wget", "df", "du", "free", "top", "whoami", "hostname",
            "uptime", "uname", "which", "echo", "touch", "head", "tail", "wc", "sort", "uniq",
            "awk", "sed", "test", "stat", "file", "tar", "zip", "unzip", "ssh", "scp", "rsync",
        ];
        
        shell_commands.contains(&first_word)
    }

    /// Check if text looks like a sentence rather than a command
    fn looks_like_sentence(&self, text: &str) -> bool {
        let sentence_starters = vec![
            "You", "To", "The", "Use", "Try", "Here", "This", "For", "If", "When", "First",
            "Then", "Next", "Finally", "Also", "Additionally", "Alternatively", "However",
        ];
        
        let first_word = text.split_whitespace().next().unwrap_or("");
        sentence_starters.contains(&first_word)
    }

    /// Detect hallucinated flags in a command
    pub fn detect_hallucinated_flags(&self, command: &str) -> Vec<String> {
        let mut found_flags = Vec::new();
        
        for flag in &self.hallucinated_flags {
            if command.contains(flag) {
                found_flags.push(flag.clone());
            }
        }
        
        found_flags
    }

    /// Check if command is safe to execute
    pub fn is_safe_to_execute(&self, command: &str) -> bool {
        let first_word = command.split_whitespace().next().unwrap_or("");
        
        // Check against safe commands list
        for safe_cmd in &self.safe_commands {
            if command.starts_with(safe_cmd) {
                return true;
            }
        }
        
        // Check for dangerous patterns
        let dangerous_patterns = vec![
            "rm -rf", "sudo", "chmod 777", "dd if=", "mkfs", "fdisk", "reboot", "shutdown",
            "kill -9", ">/dev/", "format", "shred", "curl | sh", "wget | bash",
        ];
        
        for pattern in dangerous_patterns {
            if command.contains(pattern) {
                return false;
            }
        }
        
        // Default to safe for read-only operations
        matches!(first_word, "ls" | "cat" | "head" | "tail" | "grep" | "find" | "wc" | "stat" | "file" | "test" | "echo" | "pwd" | "whoami" | "hostname" | "uname" | "date" | "uptime" | "free" | "df" | "ps" | "git")
    }

    /// Execute a safe command and return the result
    pub fn execute_command(&self, command: &str) -> Result<ExecutionResult> {
        if !self.is_safe_to_execute(command) {
            return Err(anyhow!("Command is not safe to execute: {}", command));
        }

        let output = Command::new("sh")
            .arg("-c")
            .arg(command)
            .output()?;

        Ok(ExecutionResult {
            success: output.status.success(),
            output: String::from_utf8_lossy(&output.stdout).to_string(),
            error: if output.stderr.is_empty() {
                None
            } else {
                Some(String::from_utf8_lossy(&output.stderr).to_string())
            },
            exit_code: output.status.code().unwrap_or(-1),
        })
    }

    /// Validate command against expected patterns
    pub fn validate_patterns(&self, question_id: usize, command: &str) -> Vec<PatternMatch> {
        let mut matches = Vec::new();
        
        if let Some(patterns) = self.expected_patterns.get(&question_id) {
            for pattern in patterns {
                let matched = pattern.pattern.is_match(command);
                matches.push(PatternMatch {
                    pattern_description: pattern.description.clone(),
                    matched,
                    is_required: pattern.is_required,
                });
            }
        }
        
        matches
    }

    /// Run a single test question
    pub async fn run_test(&self, question: &TestQuestion, ai_response: String, execution_time: Duration) -> TestResult {
        let execution_time_ms = execution_time.as_millis() as u64;
        let extracted_command = self.extract_command(&ai_response);
        let hallucinated_flags_found = if let Some(cmd) = &extracted_command {
            self.detect_hallucinated_flags(cmd)
        } else {
            Vec::new()
        };

        let mut status = TestStatus::NotExecuted;
        let mut execution_result = None;
        let mut failure_details = None;
        let mut pattern_matches = Vec::new();

        // Validate command extraction
        if question.should_have_command && extracted_command.is_none() {
            status = TestStatus::Failed;
            failure_details = Some("Expected command but none was extracted".to_string());
        } else if !question.should_have_command && extracted_command.is_some() {
            status = TestStatus::PartialSuccess;
            failure_details = Some("Unexpected command found for explanation question".to_string());
        }

        // Check for hallucinated flags
        if !hallucinated_flags_found.is_empty() {
            status = TestStatus::Failed;
            failure_details = Some(format!("Hallucinated flags detected: {:?}", hallucinated_flags_found));
        }

        // Validate against expected patterns
        if let Some(cmd) = &extracted_command {
            pattern_matches = self.validate_patterns(question.id, cmd);
            
            let required_patterns_failed = pattern_matches.iter()
                .any(|m| m.is_required && !m.matched);
            
            if required_patterns_failed && status == TestStatus::NotExecuted {
                status = TestStatus::Failed;
                failure_details = Some("Required pattern validation failed".to_string());
            }

            // Execute command if safe and expected
            if question.is_safe_to_execute && status != TestStatus::Failed {
                match self.execute_command(cmd) {
                    Ok(exec_result) => {
                        execution_result = Some(exec_result.clone());
                        if exec_result.success {
                            if status == TestStatus::NotExecuted {
                                status = TestStatus::Passed;
                            }
                        } else {
                            status = TestStatus::PartialSuccess;
                            failure_details = Some(format!("Command execution failed: {}", 
                                exec_result.error.unwrap_or("Unknown error".to_string())));
                        }
                    }
                    Err(e) => {
                        status = TestStatus::Failed;
                        failure_details = Some(format!("Execution error: {}", e));
                    }
                }
            } else if status == TestStatus::NotExecuted {
                // Command extracted successfully but not executed
                status = TestStatus::Passed;
            }
        } else if !question.should_have_command && status == TestStatus::NotExecuted {
            // No command expected and none found - this is correct
            status = TestStatus::Passed;
        }

        TestResult {
            question_id: question.id,
            question: question.question.clone(),
            ai_response,
            extracted_command,
            execution_time_ms,
            status,
            pattern_matches,
            hallucinated_flags_found,
            execution_result,
            failure_details,
        }
    }

    /// Generate detailed failure analysis
    pub fn generate_failure_analysis(&self, results: &[TestResult]) -> String {
        let mut analysis = String::new();
        
        analysis.push_str("# CLIAI Test Suite - Detailed Failure Analysis\n\n");
        
        let failed_tests: Vec<_> = results.iter()
            .filter(|r| r.status == TestStatus::Failed)
            .collect();
        
        let partial_tests: Vec<_> = results.iter()
            .filter(|r| r.status == TestStatus::PartialSuccess)
            .collect();
        
        analysis.push_str(&format!("## Summary\n"));
        analysis.push_str(&format!("- Total Tests: {}\n", results.len()));
        analysis.push_str(&format!("- Passed: {}\n", results.iter().filter(|r| r.status == TestStatus::Passed).count()));
        analysis.push_str(&format!("- Failed: {}\n", failed_tests.len()));
        analysis.push_str(&format!("- Partial Success: {}\n", partial_tests.len()));
        analysis.push_str(&format!("- Not Executed: {}\n\n", results.iter().filter(|r| r.status == TestStatus::NotExecuted).count()));
        
        if !failed_tests.is_empty() {
            analysis.push_str("## Failed Tests\n\n");
            for test in failed_tests {
                analysis.push_str(&format!("### Question {}: {}\n", test.question_id, test.question));
                analysis.push_str(&format!("**Status:** Failed\n"));
                if let Some(details) = &test.failure_details {
                    analysis.push_str(&format!("**Failure Reason:** {}\n", details));
                }
                if let Some(cmd) = &test.extracted_command {
                    analysis.push_str(&format!("**Extracted Command:** `{}`\n", cmd));
                }
                if !test.hallucinated_flags_found.is_empty() {
                    analysis.push_str(&format!("**Hallucinated Flags:** {:?}\n", test.hallucinated_flags_found));
                }
                analysis.push_str(&format!("**Response Time:** {}ms\n", test.execution_time_ms));
                analysis.push_str("\n");
            }
        }
        
        if !partial_tests.is_empty() {
            analysis.push_str("## Partial Success Tests\n\n");
            for test in partial_tests {
                analysis.push_str(&format!("### Question {}: {}\n", test.question_id, test.question));
                analysis.push_str(&format!("**Status:** Partial Success\n"));
                if let Some(details) = &test.failure_details {
                    analysis.push_str(&format!("**Issue:** {}\n", details));
                }
                if let Some(cmd) = &test.extracted_command {
                    analysis.push_str(&format!("**Extracted Command:** `{}`\n", cmd));
                }
                analysis.push_str(&format!("**Response Time:** {}ms\n", test.execution_time_ms));
                analysis.push_str("\n");
            }
        }
        
        // Pattern validation analysis
        analysis.push_str("## Pattern Validation Analysis\n\n");
        let mut pattern_failures = 0;
        for test in results {
            for pattern_match in &test.pattern_matches {
                if pattern_match.is_required && !pattern_match.matched {
                    pattern_failures += 1;
                    analysis.push_str(&format!("- Question {}: Failed required pattern '{}'\n", 
                        test.question_id, pattern_match.pattern_description));
                }
            }
        }
        
        if pattern_failures == 0 {
            analysis.push_str("No required pattern validation failures detected.\n");
        }
        
        analysis.push_str("\n");
        
        // Hallucinated flags analysis
        analysis.push_str("## Hallucinated Flags Analysis\n\n");
        let mut total_hallucinated_flags = 0;
        for test in results {
            if !test.hallucinated_flags_found.is_empty() {
                total_hallucinated_flags += test.hallucinated_flags_found.len();
                analysis.push_str(&format!("- Question {}: {:?}\n", 
                    test.question_id, test.hallucinated_flags_found));
            }
        }
        
        if total_hallucinated_flags == 0 {
            analysis.push_str("No hallucinated flags detected.\n");
        } else {
            analysis.push_str(&format!("\nTotal hallucinated flags found: {}\n", total_hallucinated_flags));
        }
        
        analysis
    }

    /// Run the complete test suite against CLIAI
    pub async fn run_complete_test_suite(&self, config: Config) -> Result<Vec<TestResult>> {
        let mut results = Vec::new();
        let total_questions = self.test_questions.len();
        
        println!("{}", "🧪 Starting CLIAI Comprehensive Test Suite".bold().cyan());
        println!("Testing {} questions across all categories...\n", total_questions);
        
        for (index, question) in self.test_questions.iter().enumerate() {
            println!("{} Testing question {}/{}: {}", 
                "🔍".cyan(), 
                index + 1, 
                total_questions, 
                question.question.dimmed()
            );
            
            let start_time = Instant::now();
            
            // Create fresh orchestrator for each test to avoid state pollution
            let history = History::load();
            let mut orchestrator = Orchestrator::new(config.clone(), history);
            
            // Get AI response
            let ai_response = match orchestrator.process(&question.question).await {
                Ok(response) => response,
                Err(e) => {
                    println!("  {} Failed to get AI response: {}", "❌".red(), e);
                    let test_result = TestResult {
                        question_id: question.id,
                        question: question.question.clone(),
                        ai_response: format!("ERROR: {}", e),
                        extracted_command: None,
                        execution_time_ms: start_time.elapsed().as_millis() as u64,
                        status: TestStatus::Failed,
                        pattern_matches: Vec::new(),
                        hallucinated_flags_found: Vec::new(),
                        execution_result: None,
                        failure_details: Some(format!("AI processing failed: {}", e)),
                    };
                    results.push(test_result);
                    continue;
                }
            };
            
            let execution_time = start_time.elapsed();
            
            // Run the test analysis
            let test_result = self.run_test(question, ai_response, execution_time).await;
            
            // Display immediate result
            match test_result.status {
                TestStatus::Passed => println!("  {} Passed ({}ms)", "✅".green(), test_result.execution_time_ms),
                TestStatus::Failed => {
                    println!("  {} Failed ({}ms)", "❌".red(), test_result.execution_time_ms);
                    if let Some(details) = &test_result.failure_details {
                        println!("    {}", details.red());
                    }
                }
                TestStatus::PartialSuccess => {
                    println!("  {} Partial ({}ms)", "⚠️".yellow(), test_result.execution_time_ms);
                    if let Some(details) = &test_result.failure_details {
                        println!("    {}", details.yellow());
                    }
                }
                TestStatus::NotExecuted => println!("  {} Not executed", "⏸️".dimmed()),
            }
            
            results.push(test_result);
            
            // Small delay to avoid overwhelming the AI provider
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        
        println!();
        self.display_results_summary(&results);
        
        Ok(results)
    }

    /// Run a focused test on specific categories
    pub async fn run_category_tests(&self, config: Config, categories: Vec<TestCategory>) -> Result<Vec<TestResult>> {
        let filtered_questions: Vec<_> = self.test_questions.iter()
            .filter(|q| categories.contains(&q.category))
            .collect();
        
        let mut results = Vec::new();
        
        println!("{} Running tests for categories: {:?}", "🧪".cyan(), categories);
        println!("Testing {} questions...\n", filtered_questions.len());
        
        for (index, question) in filtered_questions.iter().enumerate() {
            println!("{} Testing {}/{}: {}", 
                "🔍".cyan(), 
                index + 1, 
                filtered_questions.len(), 
                question.question.dimmed()
            );
            
            let start_time = Instant::now();
            let history = History::load();
            let mut orchestrator = Orchestrator::new(config.clone(), history);
            
            let ai_response = match orchestrator.process(&question.question).await {
                Ok(response) => response,
                Err(e) => {
                    println!("  {} Failed: {}", "❌".red(), e);
                    continue;
                }
            };
            
            let execution_time = start_time.elapsed();
            let test_result = self.run_test(question, ai_response, execution_time).await;
            
            match test_result.status {
                TestStatus::Passed => println!("  {} Passed", "✅".green()),
                TestStatus::Failed => println!("  {} Failed", "❌".red()),
                TestStatus::PartialSuccess => println!("  {} Partial", "⚠️".yellow()),
                TestStatus::NotExecuted => println!("  {} Not executed", "⏸️".dimmed()),
            }
            
            results.push(test_result);
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        
        println!();
        self.display_results_summary(&results);
        
        Ok(results)
    }

    /// Generate a comprehensive test report
    pub fn generate_test_report(&self, results: &[TestResult]) -> String {
        let mut report = String::new();
        
        report.push_str("# CLIAI Comprehensive Test Suite Report\n\n");
        report.push_str(&format!("Generated: {}\n", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));
        report.push_str(&format!("Total Tests: {}\n\n", results.len()));
        
        // Executive Summary
        let passed = results.iter().filter(|r| r.status == TestStatus::Passed).count();
        let failed = results.iter().filter(|r| r.status == TestStatus::Failed).count();
        let partial = results.iter().filter(|r| r.status == TestStatus::PartialSuccess).count();
        let not_executed = results.iter().filter(|r| r.status == TestStatus::NotExecuted).count();
        
        let success_rate = if !results.is_empty() {
            (passed as f64 / results.len() as f64) * 100.0
        } else {
            0.0
        };
        
        report.push_str("## Executive Summary\n\n");
        report.push_str(&format!("- **Success Rate**: {:.1}%\n", success_rate));
        report.push_str(&format!("- **Passed**: {} tests\n", passed));
        report.push_str(&format!("- **Failed**: {} tests\n", failed));
        report.push_str(&format!("- **Partial Success**: {} tests\n", partial));
        report.push_str(&format!("- **Not Executed**: {} tests\n\n", not_executed));
        
        // Performance Metrics
        let total_time: u64 = results.iter().map(|r| r.execution_time_ms).sum();
        let avg_time = if !results.is_empty() { total_time / results.len() as u64 } else { 0 };
        let max_time = results.iter().map(|r| r.execution_time_ms).max().unwrap_or(0);
        let min_time = results.iter().map(|r| r.execution_time_ms).min().unwrap_or(0);
        
        report.push_str("## Performance Metrics\n\n");
        report.push_str(&format!("- **Total Execution Time**: {}ms\n", total_time));
        report.push_str(&format!("- **Average Response Time**: {}ms\n", avg_time));
        report.push_str(&format!("- **Fastest Response**: {}ms\n", min_time));
        report.push_str(&format!("- **Slowest Response**: {}ms\n\n", max_time));
        
        // Category Breakdown
        report.push_str("## Results by Category\n\n");
        let mut category_stats: HashMap<TestCategory, (usize, usize, usize, usize)> = HashMap::new();
        
        for result in results {
            if let Some(question) = self.test_questions.iter().find(|q| q.id == result.question_id) {
                let entry = category_stats.entry(question.category.clone()).or_insert((0, 0, 0, 0));
                match result.status {
                    TestStatus::Passed => entry.0 += 1,
                    TestStatus::Failed => entry.1 += 1,
                    TestStatus::PartialSuccess => entry.2 += 1,
                    TestStatus::NotExecuted => entry.3 += 1,
                }
            }
        }
        
        for (category, (passed, failed, partial, not_executed)) in category_stats {
            let total = passed + failed + partial + not_executed;
            let success_rate = if total > 0 { (passed as f64 / total as f64) * 100.0 } else { 0.0 };
            report.push_str(&format!("### {:?}\n", category));
            report.push_str(&format!("- Success Rate: {:.1}%\n", success_rate));
            report.push_str(&format!("- Passed: {}, Failed: {}, Partial: {}, Not Executed: {}\n\n", 
                passed, failed, partial, not_executed));
        }
        
        // Add detailed failure analysis
        report.push_str(&self.generate_failure_analysis(results));
        
        report
    }

    /// Save test results to file
    pub fn save_test_results(&self, results: &[TestResult], filename: &str) -> Result<()> {
        let report = self.generate_test_report(results);
        std::fs::write(filename, report)?;
        println!("{} Test report saved to: {}", "💾".green(), filename.green());
        Ok(())
    }

    /// Get all test questions
    pub fn get_test_questions(&self) -> &[TestQuestion] {
        &self.test_questions
    }

    /// Display test results summary
    pub fn display_results_summary(&self, results: &[TestResult]) {
        let passed = results.iter().filter(|r| r.status == TestStatus::Passed).count();
        let failed = results.iter().filter(|r| r.status == TestStatus::Failed).count();
        let partial = results.iter().filter(|r| r.status == TestStatus::PartialSuccess).count();
        let not_executed = results.iter().filter(|r| r.status == TestStatus::NotExecuted).count();
        
        let total_time: u64 = results.iter().map(|r| r.execution_time_ms).sum();
        let avg_time = if !results.is_empty() { total_time / results.len() as u64 } else { 0 };
        
        println!("{}", "🧪 CLIAI Test Suite Results".bold().cyan());
        println!();
        println!("📊 Summary:");
        println!("  {} Passed: {}", "✅".green(), passed.to_string().green());
        println!("  {} Failed: {}", "❌".red(), failed.to_string().red());
        println!("  {} Partial: {}", "⚠️".yellow(), partial.to_string().yellow());
        println!("  {} Not Executed: {}", "⏸️".dimmed(), not_executed.to_string().dimmed());
        println!();
        println!("⏱️  Performance:");
        println!("  Total Time: {}ms", total_time.to_string().cyan());
        println!("  Average Time: {}ms", avg_time.to_string().cyan());
        println!();
        
        let success_rate = if !results.is_empty() {
            (passed as f64 / results.len() as f64) * 100.0
        } else {
            0.0
        };
        
        println!("🎯 Success Rate: {:.1}%", success_rate.to_string().green());
        
        if failed > 0 || partial > 0 {
            println!();
            println!("{}", "🔍 Issues Found:".bold().yellow());
            
            let mut hallucinated_count = 0;
            let mut pattern_failures = 0;
            let mut execution_failures = 0;
            
            for result in results {
                if !result.hallucinated_flags_found.is_empty() {
                    hallucinated_count += 1;
                }
                
                if result.pattern_matches.iter().any(|m| m.is_required && !m.matched) {
                    pattern_failures += 1;
                }
                
                if let Some(exec_result) = &result.execution_result {
                    if !exec_result.success {
                        execution_failures += 1;
                    }
                }
            }
            
            if hallucinated_count > 0 {
                println!("  {} Tests with hallucinated flags: {}", "🚫".red(), hallucinated_count);
            }
            if pattern_failures > 0 {
                println!("  {} Tests with pattern validation failures: {}", "📋".yellow(), pattern_failures);
            }
            if execution_failures > 0 {
                println!("  {} Tests with execution failures: {}", "⚡".red(), execution_failures);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_suite_creation() {
        let suite = TestSuite::new();
        assert_eq!(suite.test_questions.len(), 50);
        assert!(!suite.expected_patterns.is_empty());
        assert!(!suite.hallucinated_flags.is_empty());
        assert!(!suite.safe_commands.is_empty());
    }

    #[test]
    fn test_extract_command_with_prefix() {
        let suite = TestSuite::new();
        let response = "Command: ls -la\nThis lists all files including hidden ones.";
        let cmd = suite.extract_command(response);
        assert_eq!(cmd, Some("ls -la".to_string()));
    }

    #[test]
    fn test_extract_command_with_none() {
        let suite = TestSuite::new();
        let response = "Command: (none)\nThis is just an explanation.";
        let cmd = suite.extract_command(response);
        assert_eq!(cmd, None);
    }

    #[test]
    fn test_extract_command_with_backticks() {
        let suite = TestSuite::new();
        let response = "You can use `ls -la` to list all files.";
        let cmd = suite.extract_command(response);
        assert_eq!(cmd, Some("ls -la".to_string()));
    }

    #[test]
    fn test_detect_hallucinated_flags() {
        let suite = TestSuite::new();
        let command = "ls --hidden --recursivee";
        let flags = suite.detect_hallucinated_flags(command);
        assert_eq!(flags, vec!["--hidden", "--recursivee"]);
    }

    #[test]
    fn test_is_safe_to_execute() {
        let suite = TestSuite::new();
        
        assert!(suite.is_safe_to_execute("ls -la"));
        assert!(suite.is_safe_to_execute("cat file.txt"));
        assert!(!suite.is_safe_to_execute("rm -rf /"));
        assert!(!suite.is_safe_to_execute("sudo rm file"));
    }

    #[test]
    fn test_looks_like_shell_command() {
        let suite = TestSuite::new();
        
        assert!(suite.looks_like_shell_command("ls -la"));
        assert!(suite.looks_like_shell_command("git status"));
        assert!(!suite.looks_like_shell_command("This is a sentence"));
        assert!(!suite.looks_like_shell_command("You should use ls"));
    }

    #[test]
    fn test_clean_command() {
        let suite = TestSuite::new();
        
        assert_eq!(suite.clean_command("$ ls -la"), "ls -la");
        assert_eq!(suite.clean_command("`pwd`"), "pwd");
        assert_eq!(suite.clean_command("**find . -name '*.rs'**"), "find . -name '*.rs'");
    }

    #[test]
    fn test_validate_patterns() {
        let suite = TestSuite::new();
        let matches = suite.validate_patterns(1, "ls -la");
        
        assert!(!matches.is_empty());
        assert!(matches[0].matched);
        assert!(matches[0].is_required);
    }

    #[test]
    fn test_question_categories() {
        let suite = TestSuite::new();
        
        let file_mgmt_count = suite.test_questions.iter()
            .filter(|q| q.category == TestCategory::FileManagement)
            .count();
        
        let system_info_count = suite.test_questions.iter()
            .filter(|q| q.category == TestCategory::SystemInfo)
            .count();
        
        assert!(file_mgmt_count > 0);
        assert!(system_info_count > 0);
    }

    #[test]
    fn test_expected_command_types() {
        let suite = TestSuite::new();
        
        let has_file_ops = suite.test_questions.iter()
            .any(|q| q.expected_command_type == Some(CommandType::FileOperation));
        
        let has_explanations = suite.test_questions.iter()
            .any(|q| q.expected_command_type == Some(CommandType::Explanation));
        
        assert!(has_file_ops);
        assert!(has_explanations);
    }
}