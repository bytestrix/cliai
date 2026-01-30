use regex::Regex;

/// Quoting and escaping utilities for shell commands
pub struct QuotingCorrector {
}

/// Result of quoting analysis
#[derive(Debug, Clone)]
pub struct QuotingAnalysis {
    pub needs_correction: bool,
    pub corrected_command: String,
    pub issues_found: Vec<QuotingIssue>,
    pub corrections_applied: Vec<String>,
}

/// Types of quoting issues
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum QuotingIssue {
    /// Unquoted path with spaces
    UnquotedSpaces,
    /// Unquoted special characters
    UnquotedSpecialChars,
    /// Improper variable quoting
    ImproperVariableQuoting,
    /// Ambiguous globbing pattern
    AmbiguousGlobbing(String),
    /// Injection vulnerability
    InjectionRisk(String),
    /// Inconsistent quoting style
    InconsistentQuoting,
}

#[allow(dead_code)]
impl QuotingCorrector {
    pub fn new() -> Self {
        Self {
        }
    }

    /// Analyze and correct quoting issues in a command
    pub fn analyze_and_correct(&self, command: &str) -> QuotingAnalysis {
        let mut issues = Vec::new();
        let corrections = Vec::new();
        let corrected = command.to_string();

        /* 
        // Temporarily disabled due to aggressive quoting of flags and commands
        if let Some((fixed, correction)) = self.fix_unquoted_spaces(&corrected) {
            corrected = fixed;
            issues.push(QuotingIssue::UnquotedSpaces);
            corrections.push(correction);
        }
        */

        // Check for injection risks
        self.check_injection_risks(&corrected, &mut issues);

        QuotingAnalysis {
            needs_correction: !issues.is_empty() || corrected != command,
            corrected_command: corrected,
            issues_found: issues,
            corrections_applied: corrections,
        }
    }

    /// Fix unquoted paths with spaces
    fn fix_unquoted_spaces(&self, command: &str) -> Option<(String, String)> {
        // Simple pattern: look for unquoted sequences with spaces
        // Note: look-arounds are not supported by the Rust regex crate
        let space_pattern = Regex::new(r#"(?:\s|^)([^\s"']+\s+[^\s"']+)(?:\s|$)"#).unwrap();
        
        if let Some(captures) = space_pattern.captures(command) {
            if let Some(unquoted) = captures.get(1) {
                let quoted = format!("'{}'", unquoted.as_str());
                let fixed = command.replace(unquoted.as_str(), &quoted);
                let correction = format!("Quoted path with spaces: {} -> {}", unquoted.as_str(), quoted);
                return Some((fixed, correction));
            }
        }
        
        None
    }

    /// Check for injection risks
    fn check_injection_risks(&self, command: &str, issues: &mut Vec<QuotingIssue>) {
        // Look for patterns that could lead to command injection
        let injection_patterns = vec![
            Regex::new(r";.*\$").unwrap(), // Command separator followed by variable
            Regex::new(r"\|\s*\$").unwrap(), // Pipe to variable
            Regex::new(r"`.*\$.*`").unwrap(), // Command substitution with variable
        ];

        for pattern in &injection_patterns {
            if let Some(match_obj) = pattern.find(command) {
                issues.push(QuotingIssue::InjectionRisk(match_obj.as_str().to_string()));
            }
        }
    }

    /// Get a summary of quoting best practices
    pub fn get_quoting_guidelines(&self) -> Vec<String> {
        vec![
            "Use single quotes for literal strings (no variable expansion)".to_string(),
            "Use double quotes when you need variable expansion".to_string(),
            "Always quote paths with spaces or special characters".to_string(),
            "Quote variables to prevent word splitting: \"$var\" not $var".to_string(),
            "Avoid mixing single and double quotes unnecessarily".to_string(),
        ]
    }
}

impl Default for QuotingCorrector {
    fn default() -> Self {
        Self::new()
    }
}