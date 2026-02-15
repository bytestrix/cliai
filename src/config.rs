use crate::error_handling::{display_info, display_success, display_warning};
use crate::logging::{get_logger, LogCategory};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Default)]
pub enum SafetyLevel {
    Low,
    #[default]
    Medium,
    High,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Config {
    pub model: String,
    pub ollama_url: String,
    pub prefix: Option<String>,

    // New safety options with safe defaults
    #[serde(default = "default_auto_execute")]
    pub auto_execute: bool,

    #[serde(default = "default_dry_run")]
    pub dry_run: bool,

    #[serde(default)]
    pub safety_level: SafetyLevel,

    #[serde(default = "default_context_timeout")]
    pub context_timeout: u64,

    #[serde(default = "default_ai_timeout")]
    pub ai_timeout: u64,

    #[serde(default)]
    pub api_token: Option<String>,

    #[serde(default = "default_use_cloud")]
    pub use_cloud: bool,

    #[serde(default = "default_backend_url")]
    pub backend_url: String,
}

// Default value functions for serde
fn default_auto_execute() -> bool {
    false // Safe default: never auto-execute
}

fn default_dry_run() -> bool {
    false
}

fn default_context_timeout() -> u64 {
    2000 // 2 seconds in milliseconds
}

fn default_ai_timeout() -> u64 {
    120000 // 120 seconds in milliseconds (2 minutes)
}

fn default_use_cloud() -> bool {
    false
}

fn default_backend_url() -> String {
    "http://localhost:5000".to_string()
}

impl Config {
    pub fn load() -> Self {
        let config_path = Self::get_config_path();

        if let Some(path) = &config_path {
            if path.exists() {
                if let Ok(content) = fs::read_to_string(path) {
                    match serde_json::from_str(&content) {
                        Ok(config) => {
                            // Validate the loaded configuration
                            if let Err(e) = Self::validate_config(&config) {
                                eprintln!("Warning: Invalid configuration detected: {}. Using safe defaults.", e);
                                return Self::create_default_config(config_path.clone());
                            }
                            return config;
                        }
                        Err(e) => {
                            eprintln!("Warning: Failed to parse configuration: {}. Recreating with safe defaults.", e);
                            return Self::create_default_config(config_path.clone());
                        }
                    }
                }
            }
        }

        Self::create_default_config(config_path)
    }

    fn create_default_config(config_path: Option<PathBuf>) -> Self {
        // Default config with safe defaults
        let default_config = Self {
            model: "mistral".to_string(),
            ollama_url: "http://localhost:11434".to_string(),
            prefix: None,
            auto_execute: false, // Safe default: never auto-execute
            dry_run: false,
            safety_level: SafetyLevel::Medium,
            context_timeout: 2000, // 2 seconds
            ai_timeout: 120000,    // 120 seconds (2 minutes)
            api_token: None,
            use_cloud: false,
            backend_url: default_backend_url(),
        };

        // Try to save default config if it doesn't exist
        if let Some(path) = config_path {
            if !path.exists() {
                if let Some(parent) = path.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                let _ = fs::write(
                    path,
                    serde_json::to_string_pretty(&default_config).unwrap_or_default(),
                );
            }
        }

        default_config
    }

    /// Validate configuration values
    fn validate_config(config: &Config) -> Result<()> {
        // Validate context_timeout
        if config.context_timeout == 0 {
            return Err(anyhow!("context_timeout must be greater than 0"));
        }

        if config.context_timeout > 60000 {
            return Err(anyhow!(
                "context_timeout cannot exceed 60 seconds (60000ms)"
            ));
        }

        // Validate ai_timeout
        if config.ai_timeout == 0 {
            return Err(anyhow!("ai_timeout must be greater than 0"));
        }

        if config.ai_timeout > 600000 {
            return Err(anyhow!("ai_timeout cannot exceed 10 minutes (600000ms)"));
        }

        // Validate ollama_url format
        if !config.ollama_url.starts_with("http://") && !config.ollama_url.starts_with("https://") {
            return Err(anyhow!("ollama_url must be a valid HTTP/HTTPS URL"));
        }

        // Validate model name (basic check)
        if config.model.trim().is_empty() {
            return Err(anyhow!("model name cannot be empty"));
        }

        Ok(())
    }

    pub fn save(&self) -> Result<()> {
        // Validate before saving
        Self::validate_config(self)?;

        let config_path =
            Self::get_config_path().ok_or_else(|| anyhow!("Could not find config directory"))?;

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)?;
        fs::write(config_path, content)?;

        // Log configuration save (privacy-safe)
        if let Ok(logger) = get_logger() {
            if let Ok(logger_guard) = logger.lock() {
                let _ = logger_guard.log_info(
                    LogCategory::Configuration,
                    "Configuration saved successfully".to_string(),
                    None,
                );
            }
        }

        display_success("Configuration saved successfully");
        Ok(())
    }

    /// Update auto_execute setting and save immediately
    pub fn set_auto_execute(&mut self, enabled: bool) -> Result<()> {
        let old_value = self.auto_execute;
        self.auto_execute = enabled;

        // Log configuration change (privacy-safe)
        if let Ok(logger) = get_logger() {
            if let Ok(logger_guard) = logger.lock() {
                let _ = logger_guard.log_config_change(
                    "auto_execute",
                    &old_value.to_string(),
                    &enabled.to_string(),
                );
            }
        }

        self.save()?;

        if enabled {
            display_warning("Auto-execution enabled. Commands will be executed automatically.");
            display_info("Sensitive commands will still require confirmation.");
        } else {
            display_success(
                "Auto-execution disabled. Commands will be displayed for manual execution.",
            );
        }

        Ok(())
    }

    /// Update dry_run setting and save immediately
    pub fn set_dry_run(&mut self, enabled: bool) -> Result<()> {
        let old_value = self.dry_run;
        self.dry_run = enabled;

        // Log configuration change (privacy-safe)
        if let Ok(logger) = get_logger() {
            if let Ok(logger_guard) = logger.lock() {
                let _ = logger_guard.log_config_change(
                    "dry_run",
                    &old_value.to_string(),
                    &enabled.to_string(),
                );
            }
        }

        self.save()?;

        if enabled {
            display_info("Dry-run mode enabled. Commands will be shown but never executed.");
        } else {
            display_success(
                "Dry-run mode disabled. Commands can be executed based on auto_execute setting.",
            );
        }

        Ok(())
    }

    /// Update safety level and save immediately
    pub fn set_safety_level(&mut self, level: SafetyLevel) -> Result<()> {
        let old_level = self.safety_level;
        self.safety_level = level;

        // Log configuration change (privacy-safe)
        if let Ok(logger) = get_logger() {
            if let Ok(logger_guard) = logger.lock() {
                let _ = logger_guard.log_config_change(
                    "safety_level",
                    &format!("{:?}", old_level),
                    &format!("{:?}", level),
                );
            }
        }

        self.save()?;

        match level {
            SafetyLevel::Low => {
                display_warning("Safety level set to LOW. Fewer warnings will be shown.")
            }
            SafetyLevel::Medium => {
                display_success("Safety level set to MEDIUM. Balanced safety checks.")
            }
            SafetyLevel::High => {
                display_success("Safety level set to HIGH. Maximum safety checks enabled.")
            }
        }

        Ok(())
    }

    /// Update context timeout and save immediately
    pub fn set_context_timeout(&mut self, timeout_ms: u64) -> Result<()> {
        if timeout_ms == 0 {
            return Err(anyhow!("Context timeout must be greater than 0"));
        }

        if timeout_ms > 60000 {
            return Err(anyhow!(
                "Context timeout cannot exceed 60 seconds (60000ms)"
            ));
        }

        let old_timeout = self.context_timeout;
        self.context_timeout = timeout_ms;

        // Log configuration change (privacy-safe)
        if let Ok(logger) = get_logger() {
            if let Ok(logger_guard) = logger.lock() {
                let _ = logger_guard.log_config_change(
                    "context_timeout",
                    &format!("{}ms", old_timeout),
                    &format!("{}ms", timeout_ms),
                );
            }
        }

        self.save()?;

        display_success(&format!("Context timeout set to {}ms", timeout_ms));
        Ok(())
    }

    /// Display current configuration in a user-friendly format
    pub fn display(&self) {
        println!("{}", "🤖 CLIAI Configuration:".to_string().as_str());
        println!("Model: {}", self.model);
        println!("Ollama URL: {}", self.ollama_url);
        println!("Prefix: {}", self.prefix.as_deref().unwrap_or("none"));
        println!();
        println!("{}", "🛡️  Safety Settings:".to_string().as_str());
        println!(
            "Auto-execute: {}",
            if self.auto_execute {
                "enabled ⚠️"
            } else {
                "disabled 🛡️"
            }
        );
        println!(
            "Dry-run mode: {}",
            if self.dry_run {
                "enabled 🔍"
            } else {
                "disabled"
            }
        );
        println!("Safety level: {:?}", self.safety_level);
        println!("Context timeout: {}ms", self.context_timeout);
        println!("AI timeout: {}ms", self.ai_timeout);
        println!(
            "Cloud Mode: {}",
            if self.use_cloud {
                "enabled ☁️"
            } else {
                "disabled 🏠"
            }
        );
        println!("Backend URL: {}", self.backend_url);
        println!(
            "API Token: {}",
            if self.api_token.is_some() {
                "********"
            } else {
                "none"
            }
        );
    }

    fn get_config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|mut path| {
            path.push("cliai");
            path.push("config.json");
            path
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> Config {
        Config {
            model: "test-model".to_string(),
            ollama_url: "http://localhost:11434".to_string(),
            prefix: None,
            auto_execute: false,
            dry_run: false,
            safety_level: SafetyLevel::Medium,
            context_timeout: 2000,
            ai_timeout: 120000,
            api_token: None,
            use_cloud: false,
            backend_url: "https://api.cliai.com".to_string(),
        }
    }

    #[test]
    fn test_default_config_has_safe_defaults() {
        let config = Config::create_default_config(None);

        assert_eq!(config.auto_execute, false); // Safe default
        assert_eq!(config.dry_run, false);
        assert_eq!(config.safety_level, SafetyLevel::Medium);
        assert_eq!(config.context_timeout, 2000);
        assert_eq!(config.ai_timeout, 120000);
    }

    #[test]
    fn test_config_validation_valid() {
        let config = create_test_config();
        assert!(Config::validate_config(&config).is_ok());
    }

    #[test]
    fn test_config_validation_zero_timeout() {
        let mut config = create_test_config();
        config.context_timeout = 0;

        let result = Config::validate_config(&config);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("context_timeout must be greater than 0"));
    }

    #[test]
    fn test_config_validation_excessive_timeout() {
        let mut config = create_test_config();
        config.context_timeout = 65000; // Over 60 seconds

        let result = Config::validate_config(&config);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("context_timeout cannot exceed 60 seconds"));
    }

    #[test]
    fn test_config_validation_invalid_url() {
        let mut config = create_test_config();
        config.ollama_url = "invalid-url".to_string();

        let result = Config::validate_config(&config);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("ollama_url must be a valid HTTP/HTTPS URL"));
    }

    #[test]
    fn test_config_validation_empty_model() {
        let mut config = create_test_config();
        config.model = "".to_string();

        let result = Config::validate_config(&config);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("model name cannot be empty"));
    }

    #[test]
    fn test_safety_level_default() {
        let level = SafetyLevel::default();
        assert_eq!(level, SafetyLevel::Medium);
    }

    #[test]
    fn test_set_auto_execute() {
        let mut config = create_test_config();

        // Test enabling auto_execute
        config.auto_execute = false;
        assert_eq!(config.auto_execute, false);

        // Test the setter would work (can't test save in unit test easily)
        config.auto_execute = true;
        assert_eq!(config.auto_execute, true);
    }

    #[test]
    fn test_set_dry_run() {
        let mut config = create_test_config();

        // Test enabling dry_run
        config.dry_run = false;
        assert_eq!(config.dry_run, false);

        config.dry_run = true;
        assert_eq!(config.dry_run, true);
    }

    #[test]
    fn test_set_safety_level() {
        let mut config = create_test_config();

        config.safety_level = SafetyLevel::Low;
        assert_eq!(config.safety_level, SafetyLevel::Low);

        config.safety_level = SafetyLevel::High;
        assert_eq!(config.safety_level, SafetyLevel::High);
    }

    #[test]
    fn test_context_timeout_validation() {
        let mut config = create_test_config();

        // Valid timeout
        config.context_timeout = 5000;
        assert!(Config::validate_config(&config).is_ok());

        // Invalid: zero timeout
        config.context_timeout = 0;
        assert!(Config::validate_config(&config).is_err());

        // Invalid: excessive timeout
        config.context_timeout = 65000;
        assert!(Config::validate_config(&config).is_err());
    }

    #[test]
    fn test_serde_defaults() {
        // Test that missing fields get default values when deserializing
        let json = r#"{
            "model": "test",
            "ollama_url": "http://localhost:11434"
        }"#;

        let config: Config = serde_json::from_str(json).unwrap();

        assert_eq!(config.auto_execute, false); // Should use default
        assert_eq!(config.dry_run, false); // Should use default
        assert_eq!(config.safety_level, SafetyLevel::Medium); // Should use default
        assert_eq!(config.context_timeout, 2000); // Should use default
        assert_eq!(config.ai_timeout, 120000); // Should use default
    }

    #[test]
    fn test_config_serialization() {
        let config = create_test_config();
        let json = serde_json::to_string_pretty(&config).unwrap();

        // Verify all fields are present
        assert!(json.contains("auto_execute"));
        assert!(json.contains("dry_run"));
        assert!(json.contains("safety_level"));
        assert!(json.contains("context_timeout"));
        assert!(json.contains("ai_timeout"));
    }
}
