use anyhow::{anyhow, Result};
use keyring::Entry;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyManager {
    service_name: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub name: String,
    pub models: Vec<String>,
    pub url: String,
    pub requires_key: bool,
}

#[allow(dead_code)]
impl ApiKeyManager {
    pub fn new() -> Self {
        Self {
            service_name: "cliai".to_string(),
        }
    }

    /// Set an API key for a provider
    pub fn set_key(&self, provider: &str, api_key: &str) -> Result<()> {
        let entry = Entry::new(&self.service_name, provider)?;
        entry.set_password(api_key)?;
        println!("✅ API key set for {}", provider);
        Ok(())
    }

    /// Get an API key for a provider
    pub fn get_key(&self, provider: &str) -> Result<String> {
        let entry = Entry::new(&self.service_name, provider)?;
        match entry.get_password() {
            Ok(key) => Ok(key),
            Err(_) => Err(anyhow!("No API key found for provider: {}", provider)),
        }
    }

    /// Remove an API key for a provider
    pub fn remove_key(&self, provider: &str) -> Result<()> {
        let entry = Entry::new(&self.service_name, provider)?;
        entry.delete_password()?;
        println!("✅ API key removed for {}", provider);
        Ok(())
    }

    /// Test if an API key exists for a provider
    pub fn has_key(&self, provider: &str) -> bool {
        self.get_key(provider).is_ok()
    }

    /// List all providers that have API keys set
    pub fn list_configured_providers(&self) -> Vec<String> {
        let providers = vec!["openai", "anthropic", "google", "cohere"];
        providers
            .into_iter()
            .filter(|provider| self.has_key(provider))
            .map(|s| s.to_string())
            .collect()
    }

    /// Get supported providers configuration
    pub fn get_supported_providers() -> HashMap<String, ProviderConfig> {
        let mut providers = HashMap::new();

        providers.insert(
            "ollama".to_string(),
            ProviderConfig {
                name: "Ollama (Local)".to_string(),
                models: vec![
                    "mistral".to_string(),
                    "llama2".to_string(),
                    "codellama".to_string(),
                    "llama3".to_string(),
                    "gemma".to_string(),
                ],
                url: "http://localhost:11434".to_string(),
                requires_key: false,
            },
        );

        providers.insert(
            "openai".to_string(),
            ProviderConfig {
                name: "OpenAI".to_string(),
                models: vec![
                    "gpt-3.5-turbo".to_string(),
                    "gpt-4".to_string(),
                    "gpt-4-turbo".to_string(),
                    "gpt-4o".to_string(),
                ],
                url: "https://api.openai.com/v1".to_string(),
                requires_key: true,
            },
        );

        providers.insert(
            "anthropic".to_string(),
            ProviderConfig {
                name: "Anthropic".to_string(),
                models: vec![
                    "claude-3-haiku".to_string(),
                    "claude-3-sonnet".to_string(),
                    "claude-3-opus".to_string(),
                ],
                url: "https://api.anthropic.com/v1".to_string(),
                requires_key: true,
            },
        );

        providers
    }

    /// Test an API key by making a simple request
    pub async fn test_key(&self, provider: &str) -> Result<bool> {
        let api_key = self.get_key(provider)?;
        let providers = Self::get_supported_providers();

        let _provider_config = providers
            .get(provider)
            .ok_or_else(|| anyhow!("Unsupported provider: {}", provider))?;

        match provider {
            "openai" => self.test_openai_key(&api_key).await,
            "anthropic" => self.test_anthropic_key(&api_key).await,
            _ => Err(anyhow!(
                "Testing not implemented for provider: {}",
                provider
            )),
        }
    }

    async fn test_openai_key(&self, api_key: &str) -> Result<bool> {
        let client = reqwest::Client::new();
        let response = client
            .get("https://api.openai.com/v1/models")
            .header("Authorization", format!("Bearer {}", api_key))
            .send()
            .await?;

        Ok(response.status().is_success())
    }

    async fn test_anthropic_key(&self, api_key: &str) -> Result<bool> {
        let client = reqwest::Client::new();
        let response = client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&serde_json::json!({
                "model": "claude-3-haiku-20240307",
                "max_tokens": 1,
                "messages": [{"role": "user", "content": "test"}]
            }))
            .send()
            .await?;

        // Anthropic returns 400 for invalid requests but 401 for invalid keys
        Ok(response.status() != reqwest::StatusCode::UNAUTHORIZED)
    }
}

impl Default for ApiKeyManager {
    fn default() -> Self {
        Self::new()
    }
}
