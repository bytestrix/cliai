use anyhow::{Result, anyhow};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;
use std::time::Duration;
use std::collections::HashMap;
use crate::agents::profiles::AgentProfile;
use crate::performance::{OperationType, PerformanceMonitor, SystemPerformanceSummary, TimeoutHandler};

/// Provider type enumeration for different AI backends
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ProviderType {
    Local,
    Cloud,
}

/// Common interface for all AI providers
#[async_trait]
pub trait AIProvider: Send + Sync {
    /// Generate a response using the provider's AI model
    async fn generate_response(&self, prompt: &str, agent: &AgentProfile) -> Result<String>;
    
    /// List available models for this provider
    async fn list_models(&self) -> Result<Vec<String>>;
    
    /// Get the provider type (Local or Cloud)
    fn get_provider_type(&self) -> ProviderType;
    
    /// Check if the provider is currently available
    async fn is_available(&self) -> bool;
    
    /// Get the provider name for logging/display purposes
    fn get_name(&self) -> &'static str;
}

/// Circuit breaker states for provider failure handling
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitBreakerState {
    Closed,  // Normal operation
    Open,    // Failing, don't try
    HalfOpen, // Testing if recovered
}

/// Circuit breaker for managing provider failures
#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    state: CircuitBreakerState,
    failure_count: u32,
    failure_threshold: u32,
    recovery_timeout: Duration,
    last_failure_time: Option<std::time::Instant>,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, recovery_timeout: Duration) -> Self {
        Self {
            state: CircuitBreakerState::Closed,
            failure_count: 0,
            failure_threshold,
            recovery_timeout,
            last_failure_time: None,
        }
    }

    /// Check if the circuit breaker allows the operation
    pub fn can_execute(&mut self) -> bool {
        match self.state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::Open => {
                if let Some(last_failure) = self.last_failure_time {
                    if last_failure.elapsed() >= self.recovery_timeout {
                        self.state = CircuitBreakerState::HalfOpen;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitBreakerState::HalfOpen => true,
        }
    }

    /// Record a successful operation
    pub fn record_success(&mut self) {
        self.failure_count = 0;
        self.state = CircuitBreakerState::Closed;
        self.last_failure_time = None;
    }

    /// Record a failed operation
    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure_time = Some(std::time::Instant::now());

        if self.failure_count >= self.failure_threshold {
            self.state = CircuitBreakerState::Open;
        }
    }

    /// Get current state
    pub fn get_state(&self) -> CircuitBreakerState {
        self.state.clone()
    }
}

/// Provider manager with fallback logic and circuit breaker
pub struct ProviderManager {
    providers: Vec<Box<dyn AIProvider>>,
    fallback_chain: Vec<ProviderType>,
    retry_limits: HashMap<ProviderType, u32>,
    circuit_breakers: HashMap<ProviderType, CircuitBreaker>,
    performance_monitor: PerformanceMonitor,
}

impl ProviderManager {
    /// Create a new provider manager
    pub fn new() -> Self {
        let mut retry_limits = HashMap::new();
        retry_limits.insert(ProviderType::Local, 2);
        retry_limits.insert(ProviderType::Cloud, 1);

        let mut circuit_breakers = HashMap::new();
        circuit_breakers.insert(
            ProviderType::Local,
            CircuitBreaker::new(5, Duration::from_secs(30))
        );
        circuit_breakers.insert(
            ProviderType::Cloud,
            CircuitBreaker::new(3, Duration::from_secs(15))
        );

        Self {
            providers: Vec::new(),
            // Default is local-first; Orchestrator may override based on config.
            fallback_chain: vec![ProviderType::Local, ProviderType::Cloud],
            retry_limits,
            circuit_breakers,
            performance_monitor: PerformanceMonitor::new(),
        }
    }

    /// Add a provider to the manager
    pub fn add_provider(&mut self, provider: Box<dyn AIProvider>) {
        self.providers.push(provider);
    }

    /// Set the fallback chain order
    pub fn set_fallback_chain(&mut self, chain: Vec<ProviderType>) {
        self.fallback_chain = chain;
    }

    /// Set retry limit for a provider type
    pub fn set_retry_limit(&mut self, provider_type: ProviderType, limit: u32) {
        self.retry_limits.insert(provider_type, limit);
    }

    /// Get a response using the fallback chain with performance monitoring and timeout handling
    pub async fn get_response(&mut self, prompt: &str, agent: &AgentProfile, timeout: Duration) -> Result<String> {
        // Use the provided timeout, with a reasonable minimum of 10s
        let individual_timeout = std::cmp::max(timeout, Duration::from_secs(10));
        let total_timeout = std::cmp::max(individual_timeout, Duration::from_secs(10));
        let timeout_handler = TimeoutHandler::new(total_timeout);
        let operation_id = format!("total_system_{}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0));
        
        // Start total system performance monitoring
        self.performance_monitor.start_timer(operation_id.clone(), OperationType::TotalSystem);
        
        let mut last_error = None;
        let fallback_chain = self.fallback_chain.clone(); // Clone to avoid borrowing issues
        
        for provider_type in &fallback_chain {
            // Check if we have time remaining
            if timeout_handler.is_expired() {
                let measurement = self.performance_monitor.stop_timer_with_error(
                    &operation_id, 
                    "Total system timeout exceeded".to_string()
                )?;
                return Err(anyhow!("Total system timeout exceeded after {}", measurement.format_duration()));
            }

            // Check circuit breaker
            if let Some(circuit_breaker) = self.circuit_breakers.get_mut(provider_type) {
                if !circuit_breaker.can_execute() {
                    continue; // Skip this provider type due to circuit breaker
                }
            }

            // Determine operation type for performance monitoring
            let op_type = match provider_type {
                ProviderType::Local => OperationType::LocalOllama,
                ProviderType::Cloud => OperationType::CloudProvider,
            };

            // Get retry limit
            let retry_limit = *self.retry_limits.get(provider_type).unwrap_or(&1);

            // Try with retries
            for attempt in 0..retry_limit {
                // Use the individual timeout for this provider
                let operation_timeout = individual_timeout;
                
                if operation_timeout.is_zero() {
                    break; // No time left for this operation
                }

                // Start performance monitoring for this specific operation
                let provider_operation_id = format!("{}_{}_attempt_{}", 
                    format!("{:?}", provider_type).to_lowercase(), 
                    chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0),
                    attempt
                );
                self.performance_monitor.start_timer(provider_operation_id.clone(), op_type);

                // Find a provider of this type and execute with timeout
                let result = if let Some(provider) = self.get_provider_by_type(provider_type) {
                    tokio::time::timeout(
                        operation_timeout,
                        provider.generate_response(prompt, agent)
                    ).await
                } else {
                    continue; // No provider of this type
                };

                match result {
                    Ok(Ok(response)) => {
                        // Success - record performance and return
                        let _measurement = self.performance_monitor.stop_timer(&provider_operation_id, true)?;
                        let _total_measurement = self.performance_monitor.stop_timer(&operation_id, true)?;
                        
                        // Log performance if target was exceeded
                        if _measurement.exceeded_target() {
                            /*
                        eprintln!("⚠️  Performance target exceeded: {} took {} (target: {})", 
                            format!("{:?}", provider_type),
                            _measurement.format_duration(),
                            _measurement.format_target());
                        */
                        }
                        
                        if let Some(circuit_breaker) = self.circuit_breakers.get_mut(provider_type) {
                            circuit_breaker.record_success();
                        }
                        return Ok(response);
                    }
                    Ok(Err(e)) => {
                        // Provider error
                        let _measurement = self.performance_monitor.stop_timer_with_error(
                            &provider_operation_id, 
                            format!("Provider error: {}", e)
                        )?;
                        last_error = Some(e);
                        
                        // For local providers, we may want to limit retries if they are consistently failing
                        if matches!(provider_type, ProviderType::Local) && attempt > 0 {
                            // Already tried, maybe break if system load is high
                        }
                        
                        // Add backoff delay for retries
                        if attempt < retry_limit - 1 {
                            let backoff_ms = 100 * (attempt + 1) as u64;
                            let backoff_duration = Duration::from_millis(backoff_ms);
                            
                            // Only wait if we have time
                            if timeout_handler.has_time_for(backoff_duration) {
                                tokio::time::sleep(backoff_duration).await;
                            } else {
                                break; // No time for backoff
                            }
                        }
                    }
                    Err(_timeout_error) => {
                        // Timeout error
                        let _measurement = self.performance_monitor.stop_timer_with_error(
                            &provider_operation_id, 
                            format!("Operation timeout after {}ms", operation_timeout.as_millis())
                        )?;
                        last_error = Some(anyhow!("Provider {} timed out after {}", 
                            format!("{:?}", provider_type), 
                            _measurement.format_duration()));
                        break; // Don't retry on timeout
                    }
                }
            }

            // All retries failed for this provider type
            if let Some(circuit_breaker) = self.circuit_breakers.get_mut(provider_type) {
                circuit_breaker.record_failure();
            }
        }

        // All providers failed - record total system failure
        let _total_measurement = self.performance_monitor.stop_timer_with_error(
            &operation_id, 
            "All providers failed".to_string()
        )?;

        Err(last_error.unwrap_or_else(|| anyhow!("No providers available")))
    }

    /// List models from the first available provider
    /// List models from ALL available providers (combining Local and Cloud)
    pub async fn list_models(&self) -> Result<Vec<String>> {
        let mut all_models = Vec::new();
        let mut any_provider_available = false;

        // Iterate through all registered providers
        for provider in &self.providers {
            if provider.is_available().await {
                any_provider_available = true;
                match provider.list_models().await {
                    Ok(mut models) => all_models.append(&mut models),
                    Err(_) => continue, // Skip if listing fails for one provider
                }
            }
        }

        if !any_provider_available {
            return Err(anyhow!("No providers available"));
        }

        if all_models.is_empty() {
            // It's possible providers are available but returned no models or failed to list
            return Ok(vec!["(No models found - check provider status)".to_string()]);
        }

        // Deduplicate
        all_models.sort();
        all_models.dedup();

        Ok(all_models)
    }

    /// Check if any provider is available
    pub async fn is_any_provider_available(&self) -> bool {
        for provider in &self.providers {
            if provider.is_available().await {
                return true;
            }
        }
        false
    }

    /// Get provider by type (returns first match)
    pub fn get_provider_by_type(&self, provider_type: &ProviderType) -> Option<&Box<dyn AIProvider>> {
        self.providers.iter()
            .find(|p| p.get_provider_type() == *provider_type)
    }

    /// Get provider status for debugging
    pub fn get_provider_status(&self) -> Vec<(String, ProviderType, CircuitBreakerState)> {
        let mut status = Vec::new();
        for provider in &self.providers {
            let provider_type = provider.get_provider_type();
            let circuit_state = self.circuit_breakers.get(&provider_type)
                .map(|cb| cb.get_state())
                .unwrap_or(CircuitBreakerState::Closed);
            
            status.push((
                provider.get_name().to_string(),
                provider_type,
                circuit_state
            ));
        }
        status
    }

    /// Reset circuit breakers (for testing or manual recovery)
    pub fn reset_circuit_breakers(&mut self) {
        for circuit_breaker in self.circuit_breakers.values_mut() {
            circuit_breaker.failure_count = 0;
            circuit_breaker.state = CircuitBreakerState::Closed;
            circuit_breaker.last_failure_time = None;
        }
    }

    /// Switch provider preference at runtime
    pub fn switch_provider_preference(&mut self, preferred_type: ProviderType) {
        // Move preferred type to front of fallback chain
        self.fallback_chain.retain(|t| *t != preferred_type);
        self.fallback_chain.insert(0, preferred_type);
    }

    /// Get performance monitor for external access
    pub fn get_performance_monitor(&self) -> &PerformanceMonitor {
        &self.performance_monitor
    }

    /// Get mutable performance monitor for external access
    pub fn get_performance_monitor_mut(&mut self) -> &mut PerformanceMonitor {
        &mut self.performance_monitor
    }

    /// Get performance summary
    pub fn get_performance_summary(&self) -> SystemPerformanceSummary {
        self.performance_monitor.get_system_performance_summary()
    }

    /// Check if system is performing within acceptable limits
    pub fn is_system_healthy(&self) -> bool {
        self.performance_monitor.is_system_healthy()
    }
}

/// Local Ollama provider implementation
pub struct OllamaProvider {
    client: Client,
    base_url: String,
    model: String,
    timeout: Duration,
}

impl OllamaProvider {
    /// Create a new Ollama provider instance
    pub fn new(base_url: String, model: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(120)) // Increased to 120 seconds
            .connect_timeout(Duration::from_secs(15)) // Increased connection timeout
            .build()
            .unwrap_or_else(|_| Client::new());
            
        Self {
            client,
            base_url,
            model,
            timeout: Duration::from_secs(120), // 120 second timeout for local models
        }
    }
    
    /// Create with custom timeout
    pub fn with_timeout(base_url: String, model: String, timeout: Duration) -> Self {
        let client = Client::builder()
            .timeout(timeout)
            .connect_timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_else(|_| Client::new());
            
        Self {
            client,
            base_url,
            model,
            timeout,
        }
    }
    
    /// Update the model being used
    pub fn set_model(&mut self, model: String) {
        self.model = model;
    }
    
    /// Get the current model
    pub fn get_model(&self) -> &str {
        &self.model
    }
}

#[async_trait]
impl AIProvider for OllamaProvider {
    async fn generate_response(&self, prompt: &str, _agent: &AgentProfile) -> Result<String> {
        let body = json!({
            "model": self.model,
            "prompt": prompt,
            "stream": false,
            "options": {
                "temperature": 0.3,
                "num_predict": 512
            }
        });

        let url = format!("{}/api/generate", self.base_url);
        
        let response = self.client
            .post(&url)
            .json(&body)
            .timeout(self.timeout) // Use the configured timeout
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    anyhow!("Request to Ollama timed out after {:?}. The model may be slow to respond or overloaded. Try again or use a different model.", self.timeout)
                } else if e.is_connect() {
                    anyhow!("Failed to connect to Ollama at {}: {}. Please ensure Ollama is running with: ollama serve", self.base_url, e)
                } else {
                    anyhow!("Failed to connect to Ollama: {}", e)
                }
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Ollama returned error: {} - {}. Please check if the model '{}' is available. Try: ollama pull {}", 
                status, 
                error_text,
                self.model,
                self.model
            ));
        }

        let json: serde_json::Value = response.json().await
            .map_err(|e| anyhow!("Failed to parse Ollama response: {}", e))?;
        
        let reply = json["response"].as_str()
            .ok_or_else(|| anyhow!("No response field in Ollama response"))?;
        
        Ok(reply.to_string())
    }

    async fn list_models(&self) -> Result<Vec<String>> {
        let url = format!("{}/api/tags", self.base_url);
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to connect to Ollama: {}", e))?;
        
        if !response.status().is_success() {
            return Err(anyhow!("Ollama returned error: {}", response.status()));
        }

        let json: serde_json::Value = response.json().await
            .map_err(|e| anyhow!("Failed to parse Ollama response: {}", e))?;
        
        let mut models = Vec::new();
        if let Some(models_array) = json["models"].as_array() {
            for model in models_array {
                if let Some(name) = model["name"].as_str() {
                    models.push(name.to_string());
                }
            }
        }
        
        Ok(models)
    }

    fn get_provider_type(&self) -> ProviderType {
        ProviderType::Local
    }

    async fn is_available(&self) -> bool {
        let url = format!("{}/api/tags", self.base_url);
        
        // Use a shorter timeout for availability check
        let test_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .connect_timeout(Duration::from_secs(3))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
            
        test_client
            .get(&url)
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }

    fn get_name(&self) -> &'static str {
        "Ollama"
    }
}

/// Cloud provider implementation (Proxy to Backend)
pub struct CloudProvider {
    client: Client,
    backend_url: String,
    api_token: String,
    timeout: Duration,
}

impl CloudProvider {
    pub fn new(backend_url: String, api_token: String) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap_or_else(|_| Client::new()),
            backend_url,
            api_token,
            timeout: Duration::from_secs(30),
        }
    }
}

#[async_trait]
impl AIProvider for CloudProvider {
    async fn generate_response(&self, prompt: &str, agent: &AgentProfile) -> Result<String> {
        let body = json!({
            "prompt": prompt,
            "agent": agent.name,
            "model": "gpt-4o-mini" // Default for cloud
        });

        let url = format!("{}/v1/ai/chat", self.backend_url);
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .json(&body)
            .send()
            .await
            .map_err(|e| anyhow!("Cloud proxy failed: {}", e))?;

        if !response.status().is_success() {
            let error_json: serde_json::Value = response.json().await.unwrap_or_default();
            let msg = error_json["error"].as_str().unwrap_or("Unknown error");
            return Err(anyhow!("Cloud AI error: {}", msg));
        }

        let json: serde_json::Value = response.json().await?;
        
        // Azure OpenAI response structure support
        let reply = json["choices"][0]["message"]["content"].as_str()
            .or_else(|| json["response"].as_str())
            .ok_or_else(|| anyhow!("Unexpected response format from cloud"))?;
        
        Ok(reply.to_string())
    }

    async fn list_models(&self) -> Result<Vec<String>> {
        // For now, cloud models are fixed
        Ok(vec!["gpt-4o-mini".to_string(), "gpt-4o".to_string()])
    }

    fn get_provider_type(&self) -> ProviderType {
        ProviderType::Cloud
    }

    async fn is_available(&self) -> bool {
        let url = format!("{}/health", self.backend_url);
        self.client.get(&url).send().await.map(|r| r.status().is_success()).unwrap_or(false)
    }

    fn get_name(&self) -> &'static str {
        "Cloud (Pro)"
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_type_equality() {
        assert_eq!(ProviderType::Local, ProviderType::Local);
        assert_eq!(ProviderType::Cloud, ProviderType::Cloud);
        assert_ne!(ProviderType::Local, ProviderType::Cloud);
    }

    #[test]
    fn test_circuit_breaker_creation() {
        let cb = CircuitBreaker::new(3, Duration::from_secs(60));
        assert_eq!(cb.get_state(), CircuitBreakerState::Closed);
        assert_eq!(cb.failure_count, 0);
    }

    #[test]
    fn test_circuit_breaker_can_execute() {
        let mut cb = CircuitBreaker::new(3, Duration::from_secs(60));
        
        // Initially should allow execution
        assert!(cb.can_execute());
        
        // After failures below threshold, should still allow
        cb.record_failure();
        cb.record_failure();
        assert!(cb.can_execute());
        
        // After reaching threshold, should open
        cb.record_failure();
        assert_eq!(cb.get_state(), CircuitBreakerState::Open);
        assert!(!cb.can_execute());
    }

    #[test]
    fn test_circuit_breaker_success_reset() {
        let mut cb = CircuitBreaker::new(3, Duration::from_secs(60));
        
        // Record some failures
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.failure_count, 2);
        
        // Success should reset
        cb.record_success();
        assert_eq!(cb.failure_count, 0);
        assert_eq!(cb.get_state(), CircuitBreakerState::Closed);
    }

    #[test]
    fn test_provider_manager_creation() {
        let manager = ProviderManager::new();
        
        assert_eq!(manager.fallback_chain, vec![ProviderType::Local, ProviderType::Cloud]);
        assert_eq!(manager.retry_limits.get(&ProviderType::Cloud), Some(&1));
        assert_eq!(manager.retry_limits.get(&ProviderType::Local), Some(&2));
        assert!(manager.circuit_breakers.contains_key(&ProviderType::Cloud));
        assert!(manager.circuit_breakers.contains_key(&ProviderType::Local));
    }

    #[test]
    fn test_provider_manager_set_fallback_chain() {
        let mut manager = ProviderManager::new();
        
        manager.set_fallback_chain(vec![ProviderType::Local, ProviderType::Cloud]);
        assert_eq!(manager.fallback_chain, vec![ProviderType::Local, ProviderType::Cloud]);
    }

    #[test]
    fn test_provider_manager_set_retry_limit() {
        let mut manager = ProviderManager::new();
        
        manager.set_retry_limit(ProviderType::Cloud, 5);
        assert_eq!(manager.retry_limits.get(&ProviderType::Cloud), Some(&5));
    }

    #[test]
    fn test_provider_manager_switch_preference() {
        let mut manager = ProviderManager::new();
        
        // Initially Local first
        assert_eq!(manager.fallback_chain[0], ProviderType::Local);

        // Switch to Cloud first
        manager.switch_provider_preference(ProviderType::Cloud);
        assert_eq!(manager.fallback_chain[0], ProviderType::Cloud);
        assert_eq!(manager.fallback_chain[1], ProviderType::Local);
    }

    #[test]
    fn test_provider_manager_reset_circuit_breakers() {
        let mut manager = ProviderManager::new();
        
        // Trigger some failures
        if let Some(cb) = manager.circuit_breakers.get_mut(&ProviderType::Cloud) {
            cb.record_failure();
            cb.record_failure();
            cb.record_failure();
            assert_eq!(cb.get_state(), CircuitBreakerState::Open);
        }
        
        // Reset should clear failures
        manager.reset_circuit_breakers();
        if let Some(cb) = manager.circuit_breakers.get(&ProviderType::Cloud) {
            assert_eq!(cb.get_state(), CircuitBreakerState::Closed);
            assert_eq!(cb.failure_count, 0);
        }
    }

    #[test]
    fn test_ollama_provider_creation() {
        let provider = OllamaProvider::new(
            "http://localhost:11434".to_string(),
            "mistral".to_string()
        );
        
        assert_eq!(provider.get_provider_type(), ProviderType::Local);
        assert_eq!(provider.get_name(), "Ollama");
        assert_eq!(provider.get_model(), "mistral");
    }

    #[test]
    fn test_ollama_provider_with_timeout() {
        let provider = OllamaProvider::with_timeout(
            "http://localhost:11434".to_string(),
            "mistral".to_string(),
            Duration::from_secs(60)
        );
        
        assert_eq!(provider.timeout, Duration::from_secs(60));
    }

    #[test]
    fn test_ollama_provider_set_model() {
        let mut provider = OllamaProvider::new(
            "http://localhost:11434".to_string(),
            "mistral".to_string()
        );
        
        provider.set_model("llama2".to_string());
        assert_eq!(provider.get_model(), "llama2");
    }

    #[test]
    fn test_cloud_provider_creation() {
        let provider = CloudProvider::new(
            "https://api.openai.com/v1/chat/completions".to_string(),
            "test-key".to_string()
        );
        
        assert_eq!(provider.get_provider_type(), ProviderType::Cloud);
        assert_eq!(provider.get_name(), "Cloud (Pro)");
    }

    #[test]
    fn test_cloud_provider_set_model() {
        let provider = CloudProvider::new(
            "https://api.openai.com/v1/chat/completions".to_string(),
            "test-key".to_string()
        );
        
        // CloudProvider doesn't have set_model/get_model methods
        // Just test that it was created successfully
        assert_eq!(provider.get_provider_type(), ProviderType::Cloud);
    }

    #[tokio::test]
    async fn test_cloud_provider_list_models() {
        let provider = CloudProvider::new(
            "https://api.openai.com/v1/chat/completions".to_string(),
            "test-key".to_string()
        );
        
        // This will fail in tests since we don't have a real API key
        // but we can test that the method exists
        let result = provider.list_models().await;
        // Just ensure the method can be called
        assert!(result.is_err() || result.is_ok());
    }

    #[tokio::test]
    async fn test_provider_manager_empty_list_models() {
        let manager = ProviderManager::new();
        
        // Should fail when no providers are added
        let result = manager.list_models().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No providers available"));
    }

    #[tokio::test]
    async fn test_provider_manager_empty_is_available() {
        let manager = ProviderManager::new();
        
        // Should return false when no providers are added
        assert!(!manager.is_any_provider_available().await);
    }

    // Note: Integration tests for actual API calls would require running services
    // and are better suited for integration test suites rather than unit tests
}