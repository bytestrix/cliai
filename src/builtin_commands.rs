use crate::os_context::{OSContext, OSType, PackageManager};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;

/// A built-in command with strict pattern matching
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BuiltinCommand {
    pub pattern: Regex,
    pub command_template: String,
    pub description: String,
    pub category: CommandCategory,
    pub requires_confirmation: bool,
}

/// Categories for organizing built-in commands
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CommandCategory {
    FileOperations,
    SystemInfo,
    GitBasics,
    ProcessManagement,
    NetworkBasics,
}

/// Built-in command system with 20 essential commands
pub struct BuiltinCommands {
    pub commands: HashMap<String, BuiltinCommand>,
    pub os_context: OSContext,
}

#[allow(dead_code)]
impl BuiltinCommands {
    /// Create a new BuiltinCommands instance with all 20 essential commands
    pub fn new() -> Self {
        let mut commands = HashMap::new();

        // File Operations (8 commands)
        commands.insert(
            "list_files".to_string(),
            BuiltinCommand {
                pattern: Regex::new(r"^(ls|list files?|show files?)$").unwrap(),
                command_template: "ls -la".to_string(),
                description: "List all files including hidden ones".to_string(),
                category: CommandCategory::FileOperations,
                requires_confirmation: false,
            },
        );

        commands.insert(
            "current_directory".to_string(),
            BuiltinCommand {
                pattern: Regex::new(r"^(pwd|current directory|where am i|show path)$").unwrap(),
                command_template: "pwd".to_string(),
                description: "Show current working directory".to_string(),
                category: CommandCategory::FileOperations,
                requires_confirmation: false,
            },
        );

        commands.insert(
            "make_directory".to_string(),
            BuiltinCommand {
                pattern: Regex::new(r"^(mkdir|make directory|create directory)$").unwrap(),
                command_template: "mkdir".to_string(),
                description: "Create a new directory (requires directory name)".to_string(),
                category: CommandCategory::FileOperations,
                requires_confirmation: false,
            },
        );

        commands.insert(
            "create_file".to_string(),
            BuiltinCommand {
                pattern: Regex::new(r"^(touch|create file|make file)$").unwrap(),
                command_template: "touch".to_string(),
                description: "Create an empty file (requires filename)".to_string(),
                category: CommandCategory::FileOperations,
                requires_confirmation: false,
            },
        );

        commands.insert(
            "show_file".to_string(),
            BuiltinCommand {
                pattern: Regex::new(r"^(cat|show file|display file|read file)$").unwrap(),
                command_template: "cat".to_string(),
                description: "Display file contents (requires filename)".to_string(),
                category: CommandCategory::FileOperations,
                requires_confirmation: false,
            },
        );

        commands.insert(
            "count_lines".to_string(),
            BuiltinCommand {
                pattern: Regex::new(r"^(wc -l|count lines|line count)$").unwrap(),
                command_template: "wc -l".to_string(),
                description: "Count lines in a file (requires filename)".to_string(),
                category: CommandCategory::FileOperations,
                requires_confirmation: false,
            },
        );

        commands.insert(
            "find_files".to_string(),
            BuiltinCommand {
                pattern: Regex::new(r"^(find files?|search files?)$").unwrap(),
                command_template: "find . -name".to_string(),
                description: "Find files by name pattern (requires pattern)".to_string(),
                category: CommandCategory::FileOperations,
                requires_confirmation: false,
            },
        );

        commands.insert(
            "file_exists".to_string(),
            BuiltinCommand {
                pattern: Regex::new(r"^(file exists?|check file|test file)$").unwrap(),
                command_template: "test -f".to_string(),
                description: "Check if file exists (requires filename)".to_string(),
                category: CommandCategory::FileOperations,
                requires_confirmation: false,
            },
        );

        commands.insert("directory_exists".to_string(), BuiltinCommand {
            pattern: Regex::new(r"^(directory exists?|dir exists?|check directory|test directory|check dir|test dir)$").unwrap(),
            command_template: "test -d".to_string(),
            description: "Check if directory exists (requires directory name)".to_string(),
            category: CommandCategory::FileOperations,
            requires_confirmation: false,
        });

        // System Info (6 commands)
        commands.insert(
            "current_user".to_string(),
            BuiltinCommand {
                pattern: Regex::new(r"^(whoami|current user|who am i|username)$").unwrap(),
                command_template: "whoami".to_string(),
                description: "Show current username".to_string(),
                category: CommandCategory::SystemInfo,
                requires_confirmation: false,
            },
        );

        commands.insert(
            "hostname".to_string(),
            BuiltinCommand {
                pattern: Regex::new(r"^(hostname|computer name|system name)$").unwrap(),
                command_template: "hostname".to_string(),
                description: "Show system hostname".to_string(),
                category: CommandCategory::SystemInfo,
                requires_confirmation: false,
            },
        );

        commands.insert(
            "system_info".to_string(),
            BuiltinCommand {
                pattern: Regex::new(r"^(uname -a|system info|system information|os info)$")
                    .unwrap(),
                command_template: "uname -a".to_string(),
                description: "Show detailed system information".to_string(),
                category: CommandCategory::SystemInfo,
                requires_confirmation: false,
            },
        );

        commands.insert(
            "disk_usage".to_string(),
            BuiltinCommand {
                pattern: Regex::new(r"^(df -h|disk usage|disk space|free space)$").unwrap(),
                command_template: "df -h".to_string(),
                description: "Show disk usage in human-readable format".to_string(),
                category: CommandCategory::SystemInfo,
                requires_confirmation: false,
            },
        );

        commands.insert(
            "memory_usage".to_string(),
            BuiltinCommand {
                pattern: Regex::new(r"^(free -h|memory usage|ram usage|memory info)$").unwrap(),
                command_template: "free -h".to_string(),
                description: "Show memory usage in human-readable format".to_string(),
                category: CommandCategory::SystemInfo,
                requires_confirmation: false,
            },
        );

        commands.insert(
            "find_command".to_string(),
            BuiltinCommand {
                pattern: Regex::new(r"^(which|find command|locate command|command location)$")
                    .unwrap(),
                command_template: "which".to_string(),
                description: "Find location of a command (requires command name)".to_string(),
                category: CommandCategory::SystemInfo,
                requires_confirmation: false,
            },
        );

        // Process Management (2 commands)
        commands.insert(
            "list_processes".to_string(),
            BuiltinCommand {
                pattern: Regex::new(r"^(ps aux|list processes|show processes|running processes)$")
                    .unwrap(),
                command_template: "ps aux".to_string(),
                description: "List all running processes".to_string(),
                category: CommandCategory::ProcessManagement,
                requires_confirmation: false,
            },
        );

        commands.insert(
            "make_executable".to_string(),
            BuiltinCommand {
                pattern: Regex::new(r"^(chmod \+x|make executable|executable permission)$")
                    .unwrap(),
                command_template: "chmod +x".to_string(),
                description: "Make a file executable (requires filename)".to_string(),
                category: CommandCategory::ProcessManagement,
                requires_confirmation: false,
            },
        );

        // Git Basics (2 commands)
        commands.insert(
            "git_status".to_string(),
            BuiltinCommand {
                pattern: Regex::new(r"^(git status|repo status|repository status)$").unwrap(),
                command_template: "git status".to_string(),
                description: "Show git repository status".to_string(),
                category: CommandCategory::GitBasics,
                requires_confirmation: false,
            },
        );

        commands.insert(
            "git_log".to_string(),
            BuiltinCommand {
                pattern: Regex::new(r"^(git log|commit history|git history)$").unwrap(),
                command_template: "git log --oneline -10".to_string(),
                description: "Show recent git commit history".to_string(),
                category: CommandCategory::GitBasics,
                requires_confirmation: false,
            },
        );

        // Network Basics (2 commands) - completing the 20 essential commands
        commands.insert(
            "ping_test".to_string(),
            BuiltinCommand {
                pattern: Regex::new(r"^(ping|test connection|network test)$").unwrap(),
                command_template: "ping -c 4".to_string(),
                description: "Test network connectivity (requires hostname/IP)".to_string(),
                category: CommandCategory::NetworkBasics,
                requires_confirmation: false,
            },
        );

        commands.insert(
            "download_file".to_string(),
            BuiltinCommand {
                pattern: Regex::new(r"^(curl|download|fetch url)$").unwrap(),
                command_template: "curl -O".to_string(),
                description: "Download a file from URL (requires URL)".to_string(),
                category: CommandCategory::NetworkBasics,
                requires_confirmation: false,
            },
        );

        let os_context = OSContext::detect();

        Self {
            commands,
            os_context,
        }
    }

    /// Try to match a user input against built-in command patterns
    /// Returns the matched command if found, None otherwise
    pub fn match_command(&self, input: &str) -> Option<&BuiltinCommand> {
        let trimmed = input.trim().to_lowercase();

        // Only match simple, unambiguous requests
        // Reject overly complex inputs that should go to AI
        if trimmed.len() > 50 {
            return None;
        }

        // Reject inputs with multiple sentences or complex structure
        if trimmed.contains('.') && trimmed.split('.').count() > 2 {
            return None;
        }

        // Reject inputs with question words that suggest complex queries
        let complex_indicators = ["how", "why", "what", "when", "where", "explain", "help me"];
        if complex_indicators.iter().any(|&indicator| {
            // Use word boundaries to avoid false positives like "show" containing "how"
            trimmed.split_whitespace().any(|word| word == indicator)
        }) {
            return None;
        }

        // Reject complex folder creation requests that need AI processing
        if (trimmed.contains("folder") || trimmed.contains("directory"))
            && (trimmed.contains("inside")
                || trimmed.contains("nested")
                || trimmed.contains("multiple")
                || trimmed.contains("10")
                || trimmed.contains("several")
                || trimmed.contains("many")
                || trimmed.contains("and"))
        {
            return None; // Let AI handle complex folder structures
        }

        // Try to match against all command patterns
        self.commands
            .values()
            .find(|command| command.pattern.is_match(&trimmed))
    }

    /// Get all commands in a specific category
    pub fn get_commands_by_category(&self, category: &CommandCategory) -> Vec<&BuiltinCommand> {
        self.commands
            .values()
            .filter(|cmd| &cmd.category == category)
            .collect()
    }

    /// Get all available categories
    pub fn get_categories(&self) -> Vec<CommandCategory> {
        let mut categories: Vec<CommandCategory> = self
            .commands
            .values()
            .map(|cmd| cmd.category.clone())
            .collect();
        categories.sort_by_key(|c| format!("{:?}", c));
        categories.dedup();
        categories
    }

    /// Log built-in command usage for analytics
    pub fn log_usage(&self, command_id: &str, user_input: &str) {
        if let Some(mut path) = dirs::config_dir() {
            path.push("cliai");
            path.push("builtin_usage.log");

            if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
                let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
                let _ = writeln!(
                    file,
                    "[{}] Built-in Command Used: {} | Input: {} | Template: {}",
                    timestamp,
                    command_id,
                    user_input,
                    self.commands
                        .get(command_id)
                        .map(|cmd| cmd.command_template.as_str())
                        .unwrap_or("unknown")
                );
            }
        }
    }

    /// Generate a complete command from template and user input
    /// This handles cases where the template needs additional arguments
    pub fn generate_command(&self, builtin_cmd: &BuiltinCommand, user_input: &str) -> String {
        let template = &builtin_cmd.command_template;

        // For templates that need arguments, try to extract them from user input
        match template.as_str() {
            "mkdir" => {
                // Extract directory name from input like "mkdir test" or "create directory test"
                if let Some(dir_name) =
                    self.extract_argument_after_keywords(user_input, &["mkdir", "directory"])
                {
                    format!("mkdir {}", dir_name)
                } else {
                    format!("{} <directory_name>", template)
                }
            }
            "touch" => {
                // Extract filename from input like "touch file.txt" or "create file test.txt"
                if let Some(filename) =
                    self.extract_argument_after_keywords(user_input, &["touch", "file"])
                {
                    format!("touch {}", filename)
                } else {
                    format!("{} <filename>", template)
                }
            }
            "cat" => {
                // Extract filename from input like "cat file.txt" or "show file test.txt"
                if let Some(filename) =
                    self.extract_argument_after_keywords(user_input, &["cat", "file"])
                {
                    format!("cat {}", filename)
                } else {
                    format!("{} <filename>", template)
                }
            }
            "wc -l" => {
                // Extract filename from input like "wc -l file.txt" or "count lines file.txt"
                if let Some(filename) =
                    self.extract_argument_after_keywords(user_input, &["wc", "lines", "count"])
                {
                    format!("wc -l {}", filename)
                } else {
                    format!("{} <filename>", template)
                }
            }
            "find . -name" => {
                // Extract pattern from input like "find files *.rs" or "search files test*"
                if let Some(pattern) =
                    self.extract_argument_after_keywords(user_input, &["find", "search", "files"])
                {
                    // Ensure pattern is quoted if it contains wildcards
                    if pattern.contains('*') || pattern.contains('?') {
                        format!("find . -name \"{}\"", pattern)
                    } else {
                        format!("find . -name \"*{}*\"", pattern)
                    }
                } else {
                    format!("{} \"<pattern>\"", template)
                }
            }
            "test -f" => {
                // Extract filename and create full existence check with proper quoting
                if let Some(filename) = self.extract_argument_after_keywords(
                    user_input,
                    &["test", "file", "exists", "check"],
                ) {
                    let quoted_filename = self.quote_path_if_needed(&filename);
                    format!(
                        "test -f {} && echo 'exists' || echo 'not found'",
                        quoted_filename
                    )
                } else {
                    format!(
                        "{} <filename> && echo 'exists' || echo 'not found'",
                        template
                    )
                }
            }
            "test -d" => {
                // Extract directory name and create full existence check with proper quoting
                if let Some(dirname) = self.extract_argument_after_keywords(
                    user_input,
                    &["test", "directory", "dir", "exists", "check"],
                ) {
                    let quoted_dirname = self.quote_path_if_needed(&dirname);
                    format!(
                        "test -d {} && echo 'exists' || echo 'not found'",
                        quoted_dirname
                    )
                } else {
                    format!(
                        "{} <directory_name> && echo 'exists' || echo 'not found'",
                        template
                    )
                }
            }
            "which" => {
                // Extract command name from input like "which ls" or "find command ls"
                if let Some(cmd_name) =
                    self.extract_argument_after_keywords(user_input, &["which", "command"])
                {
                    format!("which {}", cmd_name)
                } else {
                    format!("{} <command_name>", template)
                }
            }
            "chmod +x" => {
                // Extract filename from input like "chmod +x script.sh" or "make executable script.sh"
                if let Some(filename) = self
                    .extract_argument_after_keywords(user_input, &["chmod", "executable", "file"])
                {
                    format!("chmod +x {}", filename)
                } else {
                    format!("{} <filename>", template)
                }
            }
            "ping -c 4" => {
                // Extract hostname/IP from input like "ping google.com" or "test connection google.com"
                if let Some(host) = self
                    .extract_argument_after_keywords(user_input, &["ping", "connection", "test"])
                {
                    format!("ping -c 4 {}", host)
                } else {
                    format!("{} <hostname_or_ip>", template)
                }
            }
            "curl -O" => {
                // Extract URL from input like "curl http://example.com/file" or "download http://example.com/file"
                if let Some(url) = self.extract_url_from_input(user_input) {
                    format!("curl -O {}", url)
                } else {
                    format!("{} <url>", template)
                }
            }
            _ => {
                // For commands that don't need arguments, return as-is
                template.clone()
            }
        }
    }

    /// Extract argument after specific keywords from user input
    fn extract_argument_after_keywords(&self, input: &str, keywords: &[&str]) -> Option<String> {
        let input_lower = input.to_lowercase();

        for keyword in keywords {
            if let Some(pos) = input_lower.find(keyword) {
                let after_keyword = &input[pos + keyword.len()..].trim();

                // Skip common words and get the actual argument
                let words: Vec<&str> = after_keyword.split_whitespace().collect();

                // Look for the meaningful part after skipping connecting words
                let mut meaningful_words = Vec::new();

                for word in words {
                    let word_lower = word.to_lowercase();

                    // Skip common connecting words
                    if ["the", "a", "an", "to", "for", "in", "on", "at", "by"]
                        .contains(&word_lower.as_str())
                    {
                        continue;
                    }

                    // Skip the word "file" or "directory" if it appears after the keyword
                    if ["file", "directory", "dir", "files"].contains(&word_lower.as_str())
                        && meaningful_words.is_empty()
                    {
                        continue;
                    }

                    meaningful_words.push(word);
                }

                // Join all meaningful words to handle filenames with spaces
                if !meaningful_words.is_empty() {
                    return Some(meaningful_words.join(" "));
                }
            }
        }

        None
    }

    /// Extract URL from user input (simple pattern matching)
    fn extract_url_from_input(&self, input: &str) -> Option<String> {
        let words: Vec<&str> = input.split_whitespace().collect();

        for word in words {
            if word.starts_with("http://")
                || word.starts_with("https://")
                || word.starts_with("ftp://")
            {
                return Some(word.to_string());
            }
        }

        None
    }

    /// Get total number of built-in commands
    pub fn count(&self) -> usize {
        self.commands.len()
    }

    /// Quote a path if it contains spaces or special characters
    fn quote_path_if_needed(&self, path: &str) -> String {
        // Check if the path needs quoting
        if path.contains(' ')
            || path.contains('\t')
            || path.contains('*')
            || path.contains('?')
            || path.contains('[')
            || path.contains(']')
            || path.contains('(')
            || path.contains(')')
            || path.contains('{')
            || path.contains('}')
            || path.contains('$')
            || path.contains('`')
            || path.contains('"')
            || path.contains('\'')
            || path.contains('\\')
            || path.contains('|')
            || path.contains('&')
            || path.contains(';')
            || path.contains('<')
            || path.contains('>')
        {
            // Use single quotes for safety, but handle single quotes in the path
            if path.contains('\'') {
                // If path contains single quotes, use double quotes and escape any double quotes
                format!("\"{}\"", path.replace('"', "\\\""))
            } else {
                // Use single quotes (safest for most cases)
                format!("'{}'", path)
            }
        } else {
            // No quoting needed
            path.to_string()
        }
    }

    /// Check if built-in commands should respect auto_execute configuration
    /// This is always true - built-in commands follow the same execution rules
    pub fn respects_auto_execute(&self) -> bool {
        true
    }

    /// Get OS-aware package installation command
    pub fn get_install_command(&self, package: &str) -> String {
        self.os_context.get_install_command(package)
    }

    /// Get OS-aware system update command
    pub fn get_update_command(&self) -> String {
        self.os_context.get_update_command()
    }

    /// Get OS-aware package search command
    pub fn get_package_search_command(&self, query: &str) -> String {
        self.os_context.get_package_search_command(query)
    }

    /// Get OS-aware system information command
    pub fn get_system_info_command(&self) -> String {
        self.os_context.get_system_info_command()
    }

    /// Get Arch Linux specific commands if running on Arch
    pub fn get_arch_commands(&self) -> Option<String> {
        if let Some(arch_commands) = self.os_context.get_arch_specific_commands() {
            Some(format!(
                "Arch Linux detected. Available commands:\n\
                - Package management: pacman -S <package>, pacman -Ss <query>, pacman -Syu\n\
                - AUR helper: {}\n\
                - Service management: systemctl start/stop/restart <service>\n\
                - System info: uname -a && cat /etc/os-release",
                arch_commands
                    .aur_helper
                    .as_deref()
                    .unwrap_or("Not detected (install yay or paru)")
            ))
        } else {
            None
        }
    }

    /// Check if running on Arch Linux
    pub fn is_arch_linux(&self) -> bool {
        matches!(self.os_context.os_type, OSType::ArchLinux)
    }

    /// Get OS-specific built-in commands based on detected OS
    pub fn get_os_specific_commands(&self) -> Vec<String> {
        let mut commands = Vec::new();

        match self.os_context.package_manager {
            PackageManager::Pacman => {
                commands.push("sudo pacman -S <package>".to_string());
                commands.push("pacman -Ss <query>".to_string());
                commands.push("sudo pacman -Syu".to_string());
                commands.push("pacman -Q".to_string());
                commands.push("systemctl status <service>".to_string());
            }
            PackageManager::Apt => {
                commands.push("sudo apt install <package>".to_string());
                commands.push("apt search <query>".to_string());
                commands.push("sudo apt update && sudo apt upgrade".to_string());
                commands.push("dpkg -l".to_string());
                commands.push("systemctl status <service>".to_string());
            }
            PackageManager::Brew => {
                commands.push("brew install <package>".to_string());
                commands.push("brew search <query>".to_string());
                commands.push("brew update && brew upgrade".to_string());
                commands.push("brew list".to_string());
            }
            PackageManager::Unknown => {
                commands.push("# Package manager not detected".to_string());
            }
            _ => {}
        }

        commands
    }

    /// Get OS context information for display
    pub fn get_os_info(&self) -> String {
        format!(
            "OS: {} ({})\nPackage Manager: {:?}\nShell: {:?}",
            self.os_context.version_info,
            self.os_context.architecture,
            self.os_context.package_manager,
            self.os_context.shell
        )
    }
}

impl Default for BuiltinCommands {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_commands_creation() {
        let builtin = BuiltinCommands::new();
        assert_eq!(
            builtin.count(),
            21,
            "Should have exactly 21 built-in commands (20 original + 1 directory_exists)"
        );
    }

    #[test]
    fn test_simple_command_matching() {
        let builtin = BuiltinCommands::new();

        // Test exact matches
        assert!(builtin.match_command("ls").is_some());
        assert!(builtin.match_command("pwd").is_some());
        assert!(builtin.match_command("whoami").is_some());
        assert!(builtin.match_command("git status").is_some());

        // Test case insensitivity
        assert!(builtin.match_command("LS").is_some());
        assert!(builtin.match_command("PWD").is_some());
        assert!(builtin.match_command("Git Status").is_some());
    }

    #[test]
    fn test_phrase_matching() {
        let builtin = BuiltinCommands::new();

        // Test phrase matches
        assert!(builtin.match_command("list files").is_some());
        assert!(builtin.match_command("show files").is_some());
        assert!(builtin.match_command("current directory").is_some());
        assert!(builtin.match_command("current user").is_some());
        assert!(builtin.match_command("system info").is_some());
    }

    #[test]
    fn test_complex_input_rejection() {
        let builtin = BuiltinCommands::new();

        // These should NOT match (too complex for built-in)
        assert!(builtin
            .match_command("how do I list files in a directory")
            .is_none());
        assert!(builtin
            .match_command("what is the current directory and show me files")
            .is_none());
        assert!(builtin
            .match_command("explain how to use ls command")
            .is_none());
        assert!(builtin
            .match_command("help me understand git status")
            .is_none());

        // Very long inputs should be rejected
        let long_input = "this is a very long input that should be rejected because it's too complex for built-in commands";
        assert!(builtin.match_command(long_input).is_none());
    }

    #[test]
    fn test_file_and_directory_existence_commands() {
        let builtin = BuiltinCommands::new();

        // Test file existence
        if let Some(cmd) = builtin.match_command("file exists") {
            let generated = builtin.generate_command(cmd, "test file test.txt");
            assert_eq!(
                generated,
                "test -f test.txt && echo 'exists' || echo 'not found'"
            );
        }

        // Test directory existence
        if let Some(cmd) = builtin.match_command("directory exists") {
            let generated = builtin.generate_command(cmd, "test directory test_dir");
            assert_eq!(
                generated,
                "test -d test_dir && echo 'exists' || echo 'not found'"
            );
        }

        // Test dir exists (alternative pattern)
        if let Some(cmd) = builtin.match_command("dir exists") {
            let generated = builtin.generate_command(cmd, "check dir my_directory");
            assert_eq!(
                generated,
                "test -d my_directory && echo 'exists' || echo 'not found'"
            );
        }
    }

    #[test]
    fn test_path_quoting() {
        let builtin = BuiltinCommands::new();

        // Test file with spaces - use more explicit input
        if let Some(cmd) = builtin.match_command("file exists") {
            let generated = builtin.generate_command(cmd, "check file my file.txt");
            assert_eq!(
                generated,
                "test -f 'my file.txt' && echo 'exists' || echo 'not found'"
            );
        }

        // Test directory with spaces
        if let Some(cmd) = builtin.match_command("directory exists") {
            let generated = builtin.generate_command(cmd, "check directory my directory");
            assert_eq!(
                generated,
                "test -d 'my directory' && echo 'exists' || echo 'not found'"
            );
        }

        // Test file with special characters
        if let Some(cmd) = builtin.match_command("file exists") {
            let generated = builtin.generate_command(cmd, "check file file$with*special.txt");
            assert_eq!(
                generated,
                "test -f 'file$with*special.txt' && echo 'exists' || echo 'not found'"
            );
        }
    }

    #[test]
    fn test_quote_path_if_needed() {
        let builtin = BuiltinCommands::new();

        // No quoting needed
        assert_eq!(
            builtin.quote_path_if_needed("simple_file.txt"),
            "simple_file.txt"
        );
        assert_eq!(builtin.quote_path_if_needed("file123"), "file123");

        // Quoting needed for spaces
        assert_eq!(builtin.quote_path_if_needed("my file.txt"), "'my file.txt'");
        assert_eq!(
            builtin.quote_path_if_needed("file with spaces"),
            "'file with spaces'"
        );

        // Quoting needed for special characters
        assert_eq!(builtin.quote_path_if_needed("file$var"), "'file$var'");
        assert_eq!(
            builtin.quote_path_if_needed("file*pattern"),
            "'file*pattern'"
        );
        assert_eq!(builtin.quote_path_if_needed("file[123]"), "'file[123]'");

        // Handle single quotes in path (use double quotes)
        assert_eq!(
            builtin.quote_path_if_needed("file's name"),
            "\"file's name\""
        );

        // Handle both single and double quotes (escape double quotes)
        assert_eq!(
            builtin.quote_path_if_needed("file's \"quoted\" name"),
            "\"file's \\\"quoted\\\" name\""
        );
    }

    #[test]
    fn test_categories() {
        let builtin = BuiltinCommands::new();
        let categories = builtin.get_categories();

        // Should have all expected categories
        assert!(categories.contains(&CommandCategory::FileOperations));
        assert!(categories.contains(&CommandCategory::SystemInfo));
        assert!(categories.contains(&CommandCategory::GitBasics));
        assert!(categories.contains(&CommandCategory::ProcessManagement));
        assert!(categories.contains(&CommandCategory::NetworkBasics));
    }

    #[test]
    fn test_file_operations_category() {
        let builtin = BuiltinCommands::new();
        let file_ops = builtin.get_commands_by_category(&CommandCategory::FileOperations);

        // Should have multiple file operation commands
        assert!(file_ops.len() >= 5);

        // Check that ls command is in file operations
        let ls_cmd = builtin.match_command("ls").unwrap();
        assert_eq!(ls_cmd.category, CommandCategory::FileOperations);
    }

    #[test]
    fn test_system_info_category() {
        let builtin = BuiltinCommands::new();
        let sys_info = builtin.get_commands_by_category(&CommandCategory::SystemInfo);

        // Should have multiple system info commands
        assert!(sys_info.len() >= 4);

        // Check that whoami command is in system info
        let whoami_cmd = builtin.match_command("whoami").unwrap();
        assert_eq!(whoami_cmd.category, CommandCategory::SystemInfo);
    }

    #[test]
    fn test_git_basics_category() {
        let builtin = BuiltinCommands::new();
        let git_cmds = builtin.get_commands_by_category(&CommandCategory::GitBasics);

        // Should have git commands
        assert!(git_cmds.len() >= 2);

        // Check that git status is in git basics
        let git_status_cmd = builtin.match_command("git status").unwrap();
        assert_eq!(git_status_cmd.category, CommandCategory::GitBasics);
    }

    #[test]
    fn test_strict_patterns() {
        let builtin = BuiltinCommands::new();

        // These should match
        assert!(builtin.match_command("ls").is_some());
        assert!(builtin.match_command("list files").is_some());

        // These should NOT match (to prevent false positives)
        assert!(builtin.match_command("ls -la /home").is_none()); // Too specific
        assert!(builtin
            .match_command("list all files in directory")
            .is_none()); // Too complex
        assert!(builtin.match_command("please list files").is_none()); // Contains "please"
    }

    #[test]
    fn test_argument_extraction() {
        let builtin = BuiltinCommands::new();

        // Test directory name extraction
        assert_eq!(
            builtin.extract_argument_after_keywords("mkdir test_dir", &["mkdir"]),
            Some("test_dir".to_string())
        );

        // Test filename extraction
        assert_eq!(
            builtin.extract_argument_after_keywords("cat file.txt", &["cat"]),
            Some("file.txt".to_string())
        );

        // Test with connecting words
        assert_eq!(
            builtin.extract_argument_after_keywords("create directory for test", &["directory"]),
            Some("test".to_string())
        );
    }

    #[test]
    fn test_url_extraction() {
        let builtin = BuiltinCommands::new();

        assert_eq!(
            builtin.extract_url_from_input("download https://example.com/file.txt"),
            Some("https://example.com/file.txt".to_string())
        );

        assert_eq!(
            builtin.extract_url_from_input("curl http://test.com/data"),
            Some("http://test.com/data".to_string())
        );

        assert_eq!(
            builtin.extract_url_from_input("just some text without url"),
            None
        );
    }

    #[test]
    fn test_respects_auto_execute() {
        let builtin = BuiltinCommands::new();
        assert!(builtin.respects_auto_execute());
    }

    #[test]
    fn test_no_confirmation_required() {
        let builtin = BuiltinCommands::new();

        // All built-in commands should be safe and not require confirmation
        for cmd in builtin.commands.values() {
            assert!(
                !cmd.requires_confirmation,
                "Built-in command '{}' should not require confirmation",
                cmd.description
            );
        }
    }

    #[test]
    fn test_find_files_pattern_generation() {
        let builtin = BuiltinCommands::new();

        if let Some(cmd) = builtin.match_command("find files") {
            // Test with wildcard pattern
            let generated = builtin.generate_command(cmd, "find files *.rs");
            assert_eq!(generated, "find . -name \"*.rs\"");

            // Test with simple name
            let generated = builtin.generate_command(cmd, "find files test");
            assert_eq!(generated, "find . -name \"*test*\"");
        }
    }
}
