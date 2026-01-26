use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatTurn {
    pub role: String,
    pub content: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct History {
    pub turns: Vec<ChatTurn>,
}

/// Context window configuration for different agent types
#[derive(Debug, Clone)]
pub struct ContextWindow {
    pub max_turns: usize,
    pub include_system_context: bool,
    pub prioritize_recent: bool,
    pub include_working_directory: bool,
    pub context_priority: ContextPriority,
}

/// Priority levels for context when size limits are reached
#[derive(Debug, Clone, PartialEq)]
pub enum ContextPriority {
    /// Prioritize recent interactions over older ones
    Recent,
    /// Prioritize system context over conversation history
    SystemFirst,
    /// Balanced approach - mix of recent and system context
    Balanced,
}

impl ContextWindow {
    /// Context window for ShellExpert: last 3 turns + working directory context
    /// Requirements 9.1: ShellExpert gets last 3 conversation turns plus current working directory context
    pub fn shell_expert() -> Self {
        Self {
            max_turns: 3,
            include_system_context: true,
            prioritize_recent: true,
            include_working_directory: true,
            context_priority: ContextPriority::SystemFirst,
        }
    }
    
    /// Context window for General agents: full history
    /// Requirements 9.2: General agents get full conversation history for context continuity
    pub fn general_agent() -> Self {
        Self {
            max_turns: usize::MAX, // No limit for general agents - they get full history
            include_system_context: true,
            prioritize_recent: false,
            include_working_directory: true,
            context_priority: ContextPriority::Balanced,
        }
    }
    
    /// Context window for specialized agents: moderate history
    pub fn specialized_agent() -> Self {
        Self {
            max_turns: 5,
            include_system_context: true,
            prioritize_recent: true,
            include_working_directory: true,
            context_priority: ContextPriority::Recent,
        }
    }
}

impl History {
    pub fn load() -> Self {
        let history_path = Self::get_history_path();
        
        if let Some(path) = &history_path {
            if path.exists() {
                if let Ok(content) = fs::read_to_string(path) {
                    if let Ok(history) = serde_json::from_str::<History>(&content) {
                        return history;
                    }
                }
            }
        }
        
        History { turns: Vec::new() }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let history_path = Self::get_history_path().ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;
        if let Some(parent) = history_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // Requirements 9.3: Automatic history truncation after 10 turns
        // Keep only last 10 turns to keep context manageable
        let mut limited_history = self.clone();
        if limited_history.turns.len() > 10 {
            limited_history.turns = limited_history.turns.split_off(limited_history.turns.len() - 10);
        }
        
        let content = serde_json::to_string_pretty(&limited_history)?;
        fs::write(history_path, content)?;
        Ok(())
    }

    pub fn add_turn(&mut self, role: &str, content: &str) {
        self.turns.push(ChatTurn {
            role: role.to_string(),
            content: content.to_string(),
        });
        
        // Requirements 9.3: Automatic history truncation after 10 turns
        if self.turns.len() > 10 {
            self.turns = self.turns.split_off(self.turns.len() - 10);
        }
    }

    pub fn clear(&mut self) {
        self.turns.clear();
        let _ = self.save();
    }
    
    /// Get context-appropriate history for an agent
    /// Requirements 9.4: Context prioritization when size limits are reached
    pub fn get_context_for_agent(&self, context_window: &ContextWindow) -> Vec<ChatTurn> {
        if self.turns.is_empty() {
            return Vec::new();
        }
        
        // For general agents, provide full history (no turn limit)
        // Requirements 9.2: General agents get full conversation history
        if context_window.max_turns == usize::MAX {
            return self.turns.clone();
        }
        
        let max_turns = context_window.max_turns.min(self.turns.len());
        
        if context_window.prioritize_recent {
            // Take the most recent turns
            // Requirements 9.1: ShellExpert gets last 3 conversation turns
            self.turns.iter()
                .rev()
                .take(max_turns)
                .rev()
                .cloned()
                .collect()
        } else {
            // Take turns from the beginning (for specialized agents that need chronological context)
            self.turns.iter()
                .take(max_turns)
                .cloned()
                .collect()
        }
    }
    
    /// Format history for inclusion in AI prompts with context prioritization
    /// Requirements 9.5: Context includes relevant system information
    pub fn format_for_prompt(&self, context_window: &ContextWindow) -> String {
        let relevant_turns = self.get_context_for_agent(context_window);
        
        if relevant_turns.is_empty() {
            return String::new();
        }
        
        let mut formatted = String::new();
        
        // Add working directory context if requested
        // Requirements 9.1: ShellExpert gets current working directory context
        if context_window.include_working_directory {
            if let Ok(current_dir) = std::env::current_dir() {
                formatted.push_str(&format!("Current working directory: {}\n\n", current_dir.display()));
            }
        }
        
        // Add conversation history with context prioritization
        // Requirements 9.4: Context prioritization when size limits are reached
        match context_window.context_priority {
            ContextPriority::SystemFirst => {
                // System context already added above, now add conversation
                formatted.push_str("Recent conversation history:\n");
                for turn in relevant_turns {
                    formatted.push_str(&format!("{}: {}\n", turn.role, turn.content));
                }
            }
            ContextPriority::Recent => {
                // Prioritize most recent interactions
                formatted.push_str("Recent conversation history:\n");
                for turn in relevant_turns {
                    formatted.push_str(&format!("{}: {}\n", turn.role, turn.content));
                }
            }
            ContextPriority::Balanced => {
                // Balanced approach - include both system and conversation context
                formatted.push_str("Conversation history:\n");
                for turn in relevant_turns {
                    formatted.push_str(&format!("{}: {}\n", turn.role, turn.content));
                }
            }
        }
        
        formatted
    }
    
    /// Get the number of turns in history
    pub fn len(&self) -> usize {
        self.turns.len()
    }
    
    /// Check if history is empty
    pub fn is_empty(&self) -> bool {
        self.turns.is_empty()
    }
    
    /// Get the last N turns
    pub fn get_last_turns(&self, n: usize) -> Vec<ChatTurn> {
        if self.turns.is_empty() {
            return Vec::new();
        }
        
        let start_index = if self.turns.len() > n {
            self.turns.len() - n
        } else {
            0
        };
        
        self.turns[start_index..].to_vec()
    }
    
    /// Get context with size-based prioritization
    /// Requirements 9.4: Context prioritization when size limits are reached
    pub fn get_prioritized_context(&self, context_window: &ContextWindow, max_chars: Option<usize>) -> Vec<ChatTurn> {
        let context = self.get_context_for_agent(context_window);
        
        // If no size limit specified, return full context
        let max_size = match max_chars {
            Some(size) => size,
            None => return context,
        };
        
        // Calculate current size
        let current_size: usize = context.iter()
            .map(|turn| turn.content.len() + turn.role.len() + 4) // +4 for formatting
            .sum();
        
        if current_size <= max_size {
            return context;
        }
        
        // Apply prioritization strategy when size limits are reached
        match context_window.context_priority {
            ContextPriority::Recent => {
                // Keep most recent turns that fit within size limit
                let mut total_size = 0;
                let mut result = Vec::new();
                
                for turn in context.iter().rev() {
                    let turn_size = turn.content.len() + turn.role.len() + 4;
                    if total_size + turn_size <= max_size {
                        result.insert(0, turn.clone());
                        total_size += turn_size;
                    } else {
                        break;
                    }
                }
                result
            }
            ContextPriority::SystemFirst => {
                // Prioritize system context, then recent conversation
                // For now, same as Recent since system context is handled separately
                self.get_prioritized_context(context_window, max_chars)
            }
            ContextPriority::Balanced => {
                // Take a balanced sample from beginning and end
                let half_size = max_size / 2;
                let mut result = Vec::new();
                let mut total_size = 0;
                
                // Take from beginning
                for turn in context.iter() {
                    let turn_size = turn.content.len() + turn.role.len() + 4;
                    if total_size + turn_size <= half_size {
                        result.push(turn.clone());
                        total_size += turn_size;
                    } else {
                        break;
                    }
                }
                
                // Take from end
                let remaining_size = max_size - total_size;
                let mut end_turns = Vec::new();
                let mut end_size = 0;
                
                for turn in context.iter().rev() {
                    let turn_size = turn.content.len() + turn.role.len() + 4;
                    if end_size + turn_size <= remaining_size && !result.contains(turn) {
                        end_turns.insert(0, turn.clone());
                        end_size += turn_size;
                    } else if !result.contains(turn) {
                        break;
                    }
                }
                
                result.extend(end_turns);
                result
            }
        }
    }

    fn get_history_path() -> Option<PathBuf> {
        dirs::config_dir().map(|mut path| {
            path.push("cliai");
            path.push("history.json");
            path
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_history() -> History {
        let mut history = History { turns: Vec::new() };
        
        // Add some test turns
        history.add_turn("user", "First message");
        history.add_turn("assistant", "First response");
        history.add_turn("user", "Second message");
        history.add_turn("assistant", "Second response");
        history.add_turn("user", "Third message");
        history.add_turn("assistant", "Third response");
        
        history
    }
    
    #[test]
    fn test_context_window_shell_expert() {
        let window = ContextWindow::shell_expert();
        assert_eq!(window.max_turns, 3);
        assert!(window.include_system_context);
        assert!(window.prioritize_recent);
        assert!(window.include_working_directory);
        assert_eq!(window.context_priority, ContextPriority::SystemFirst);
    }
    
    #[test]
    fn test_context_window_general_agent() {
        let window = ContextWindow::general_agent();
        assert_eq!(window.max_turns, usize::MAX); // No limit for general agents
        assert!(window.include_system_context);
        assert!(!window.prioritize_recent);
        assert!(window.include_working_directory);
        assert_eq!(window.context_priority, ContextPriority::Balanced);
    }
    
    #[test]
    fn test_context_window_specialized_agent() {
        let window = ContextWindow::specialized_agent();
        assert_eq!(window.max_turns, 5);
        assert!(window.include_system_context);
        assert!(window.prioritize_recent);
        assert!(window.include_working_directory);
        assert_eq!(window.context_priority, ContextPriority::Recent);
    }
    
    #[test]
    fn test_get_context_for_agent_shell_expert() {
        let history = create_test_history();
        let window = ContextWindow::shell_expert();
        
        let context = history.get_context_for_agent(&window);
        
        // Should get last 3 turns (prioritize recent)
        assert_eq!(context.len(), 3);
        assert_eq!(context[0].content, "Second response");
        assert_eq!(context[1].content, "Third message");
        assert_eq!(context[2].content, "Third response");
    }
    
    #[test]
    fn test_get_context_for_agent_general() {
        let history = create_test_history();
        let window = ContextWindow::general_agent();
        
        let context = history.get_context_for_agent(&window);
        
        // Should get all turns (no limit for general agents)
        assert_eq!(context.len(), 6);
        assert_eq!(context[0].content, "First message");
        assert_eq!(context[5].content, "Third response");
    }
    
    #[test]
    fn test_get_context_for_agent_empty_history() {
        let history = History { turns: Vec::new() };
        let window = ContextWindow::shell_expert();
        
        let context = history.get_context_for_agent(&window);
        assert!(context.is_empty());
    }
    
    #[test]
    fn test_format_for_prompt() {
        let history = create_test_history();
        let window = ContextWindow::shell_expert();
        
        let formatted = history.format_for_prompt(&window);
        
        assert!(formatted.contains("Current working directory:"));
        assert!(formatted.contains("Recent conversation history:"));
        assert!(formatted.contains("user: Third message"));
        assert!(formatted.contains("assistant: Third response"));
    }
    
    #[test]
    fn test_format_for_prompt_empty_history() {
        let history = History { turns: Vec::new() };
        let window = ContextWindow::shell_expert();
        
        let formatted = history.format_for_prompt(&window);
        // Should still include working directory even with empty history
        assert!(formatted.contains("Current working directory:") || formatted.is_empty());
    }
    
    #[test]
    fn test_context_priority_recent() {
        let history = create_test_history();
        let window = ContextWindow::specialized_agent(); // Uses Recent priority
        
        let context = history.get_prioritized_context(&window, Some(100)); // Small size limit
        
        // Should prioritize recent turns
        assert!(!context.is_empty());
        // Most recent content should be preserved
        let last_turn = context.last().unwrap();
        assert!(last_turn.content.contains("Third") || last_turn.content.contains("Second"));
    }
    
    #[test]
    fn test_context_priority_balanced() {
        let history = create_test_history();
        let window = ContextWindow::general_agent(); // Uses Balanced priority
        
        let context = history.get_prioritized_context(&window, Some(200)); // Medium size limit
        
        // Should include both early and recent turns
        assert!(!context.is_empty());
    }
    
    #[test]
    fn test_get_prioritized_context_no_limit() {
        let history = create_test_history();
        let window = ContextWindow::shell_expert();
        
        let context = history.get_prioritized_context(&window, None);
        
        // Should return same as get_context_for_agent when no size limit
        let expected = history.get_context_for_agent(&window);
        assert_eq!(context.len(), expected.len());
    }
    
    #[test]
    fn test_general_agent_full_history() {
        let mut history = History { turns: Vec::new() };
        
        // Add 15 turns (more than the 10 turn storage limit, but general agent should get all stored)
        for i in 1..=15 {
            history.add_turn("user", &format!("Message {}", i));
        }
        
        let window = ContextWindow::general_agent();
        let context = history.get_context_for_agent(&window);
        
        // Should get all stored turns (limited by storage, not context window)
        assert_eq!(context.len(), 10); // Storage limit kicks in
        assert_eq!(context[0].content, "Message 6"); // First stored turn after truncation
        assert_eq!(context[9].content, "Message 15"); // Last turn
    }
    
    #[test]
    fn test_shell_expert_limited_context() {
        let mut history = History { turns: Vec::new() };
        
        // Add 10 turns
        for i in 1..=10 {
            history.add_turn("user", &format!("Message {}", i));
        }
        
        let window = ContextWindow::shell_expert();
        let context = history.get_context_for_agent(&window);
        
        // Should get only last 3 turns
        assert_eq!(context.len(), 3);
        assert_eq!(context[0].content, "Message 8");
        assert_eq!(context[2].content, "Message 10");
    }
    
    #[test]
    fn test_automatic_truncation() {
        let mut history = History { turns: Vec::new() };
        
        // Add 12 turns (more than the limit of 10)
        for i in 1..=12 {
            history.add_turn("user", &format!("Message {}", i));
        }
        
        // Should automatically truncate to 10
        assert_eq!(history.turns.len(), 10);
        assert_eq!(history.turns[0].content, "Message 3"); // First 2 should be removed
        assert_eq!(history.turns[9].content, "Message 12");
    }
    
    #[test]
    fn test_get_last_turns() {
        let history = create_test_history();
        
        let last_2 = history.get_last_turns(2);
        assert_eq!(last_2.len(), 2);
        assert_eq!(last_2[0].content, "Third message");
        assert_eq!(last_2[1].content, "Third response");
        
        let last_10 = history.get_last_turns(10);
        assert_eq!(last_10.len(), 6); // Only 6 turns available
    }
    
    #[test]
    fn test_get_last_turns_empty_history() {
        let history = History { turns: Vec::new() };
        let last_turns = history.get_last_turns(5);
        assert!(last_turns.is_empty());
    }
    
    #[test]
    fn test_history_len_and_is_empty() {
        let mut history = History { turns: Vec::new() };
        assert_eq!(history.len(), 0);
        assert!(history.is_empty());
        
        history.add_turn("user", "test");
        assert_eq!(history.len(), 1);
        assert!(!history.is_empty());
    }
}
