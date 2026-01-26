use std::time::{Duration, Instant};
use std::collections::HashMap;
use anyhow::{Result, anyhow};
use crate::logging::{get_logger, LogCategory, LogContext};

/// Performance targets for different operation types
#[derive(Debug, Clone)]
pub struct PerformanceTargets {
    pub builtin_command: Duration,
    pub local_ollama: Duration,
    pub cloud_provider: Duration,
    pub total_system: Duration,
}

impl Default for PerformanceTargets {
    fn default() -> Self {
        Self {
            builtin_command: Duration::from_millis(10),  // <10ms target
            local_ollama: Duration::from_secs(2),        // <2s target
            cloud_provider: Duration::from_millis(800),  // <800ms target
            total_system: Duration::from_secs(5),        // 5s maximum
        }
    }
}

/// Types of operations being monitored
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OperationType {
    BuiltinCommand,
    LocalOllama,
    CloudProvider,
    TotalSystem,
    ContextGathering,
    CommandValidation,
    IntentClassification,
}

impl OperationType {
    pub fn get_target_duration(&self, targets: &PerformanceTargets) -> Duration {
        match self {
            OperationType::BuiltinCommand => targets.builtin_command,
            OperationType::LocalOllama => targets.local_ollama,
            OperationType::CloudProvider => targets.cloud_provider,
            OperationType::TotalSystem => targets.total_system,
            OperationType::ContextGathering => Duration::from_millis(500), // 500ms for context
            OperationType::CommandValidation => Duration::from_millis(50), // 50ms for validation
            OperationType::IntentClassification => Duration::from_millis(100), // 100ms for intent
        }
    }
}

/// Performance measurement result
#[derive(Debug, Clone)]
pub struct PerformanceMeasurement {
    pub operation_type: OperationType,
    pub duration: Duration,
    pub target: Duration,
    pub success: bool,
    pub error: Option<String>,
    pub timestamp: Instant,
}

impl PerformanceMeasurement {
    pub fn new(operation_type: OperationType, duration: Duration, target: Duration, success: bool) -> Self {
        Self {
            operation_type,
            duration,
            target,
            success,
            error: None,
            timestamp: Instant::now(),
        }
    }

    pub fn with_error(operation_type: OperationType, duration: Duration, target: Duration, error: String) -> Self {
        Self {
            operation_type,
            duration,
            target,
            success: false,
            error: Some(error),
            timestamp: Instant::now(),
        }
    }

    /// Check if the measurement exceeded the target
    pub fn exceeded_target(&self) -> bool {
        self.duration > self.target
    }

    /// Get performance ratio (actual/target)
    pub fn performance_ratio(&self) -> f64 {
        self.duration.as_millis() as f64 / self.target.as_millis() as f64
    }

    /// Format duration for display
    pub fn format_duration(&self) -> String {
        if self.duration.as_millis() < 1000 {
            format!("{}ms", self.duration.as_millis())
        } else {
            format!("{:.2}s", self.duration.as_secs_f64())
        }
    }

    /// Format target for display
    pub fn format_target(&self) -> String {
        if self.target.as_millis() < 1000 {
            format!("{}ms", self.target.as_millis())
        } else {
            format!("{:.2}s", self.target.as_secs_f64())
        }
    }
}

/// Performance monitor for tracking operation timings
pub struct PerformanceMonitor {
    targets: PerformanceTargets,
    measurements: Vec<PerformanceMeasurement>,
    active_timers: HashMap<String, (OperationType, Instant)>,
    max_measurements: usize,
}

impl PerformanceMonitor {
    /// Create a new performance monitor
    pub fn new() -> Self {
        Self {
            targets: PerformanceTargets::default(),
            measurements: Vec::new(),
            active_timers: HashMap::new(),
            max_measurements: 100, // Keep last 100 measurements
        }
    }

    /// Create with custom targets
    pub fn with_targets(targets: PerformanceTargets) -> Self {
        Self {
            targets,
            measurements: Vec::new(),
            active_timers: HashMap::new(),
            max_measurements: 100,
        }
    }

    /// Start timing an operation
    pub fn start_timer(&mut self, operation_id: String, operation_type: OperationType) {
        self.active_timers.insert(operation_id, (operation_type, Instant::now()));
    }

    /// Stop timing an operation and record the measurement
    pub fn stop_timer(&mut self, operation_id: &str, success: bool) -> Result<PerformanceMeasurement> {
        if let Some((operation_type, start_time)) = self.active_timers.remove(operation_id) {
            let duration = start_time.elapsed();
            let target = operation_type.get_target_duration(&self.targets);
            let measurement = PerformanceMeasurement::new(operation_type, duration, target, success);
            
            // Log performance measurement
            self.log_measurement(&measurement);
            
            // Store measurement
            self.add_measurement(measurement.clone());
            
            Ok(measurement)
        } else {
            Err(anyhow!("No active timer found for operation: {}", operation_id))
        }
    }

    /// Stop timing with error
    pub fn stop_timer_with_error(&mut self, operation_id: &str, error: String) -> Result<PerformanceMeasurement> {
        if let Some((operation_type, start_time)) = self.active_timers.remove(operation_id) {
            let duration = start_time.elapsed();
            let target = operation_type.get_target_duration(&self.targets);
            let measurement = PerformanceMeasurement::with_error(operation_type, duration, target, error);
            
            // Log performance measurement
            self.log_measurement(&measurement);
            
            // Store measurement
            self.add_measurement(measurement.clone());
            
            Ok(measurement)
        } else {
            Err(anyhow!("No active timer found for operation: {}", operation_id))
        }
    }

    /// Record a measurement directly (for operations measured externally)
    pub fn record_measurement(&mut self, operation_type: OperationType, duration: Duration, success: bool) -> PerformanceMeasurement {
        let target = operation_type.get_target_duration(&self.targets);
        let measurement = PerformanceMeasurement::new(operation_type, duration, target, success);
        
        // Log performance measurement
        self.log_measurement(&measurement);
        
        // Store measurement
        self.add_measurement(measurement.clone());
        
        measurement
    }

    /// Add measurement to history
    fn add_measurement(&mut self, measurement: PerformanceMeasurement) {
        self.measurements.push(measurement);
        
        // Keep only the most recent measurements
        if self.measurements.len() > self.max_measurements {
            self.measurements.remove(0);
        }
    }

    /// Log performance measurement
    fn log_measurement(&self, measurement: &PerformanceMeasurement) {
        if let Ok(logger) = get_logger() {
            if let Ok(logger_guard) = logger.lock() {
                let context = LogContext::new()
                    .with_operation_type(format!("{:?}", measurement.operation_type))
                    .with_duration_ms(measurement.duration.as_millis() as u64)
                    .with_target_ms(measurement.target.as_millis() as u64)
                    .with_success(measurement.success);

                let message = if measurement.success {
                    if measurement.exceeded_target() {
                        format!("Performance target exceeded: {} took {} (target: {})", 
                            format!("{:?}", measurement.operation_type),
                            measurement.format_duration(),
                            measurement.format_target())
                    } else {
                        format!("Performance within target: {} took {} (target: {})", 
                            format!("{:?}", measurement.operation_type),
                            measurement.format_duration(),
                            measurement.format_target())
                    }
                } else {
                    format!("Operation failed: {} took {} (error: {})", 
                        format!("{:?}", measurement.operation_type),
                        measurement.format_duration(),
                        measurement.error.as_deref().unwrap_or("unknown"))
                };

                let _ = logger_guard.log_with_context(LogCategory::Performance, &message, &context);
            }
        }
    }

    /// Get recent measurements for a specific operation type
    pub fn get_recent_measurements(&self, operation_type: &OperationType, count: usize) -> Vec<&PerformanceMeasurement> {
        self.measurements
            .iter()
            .rev()
            .filter(|m| m.operation_type == *operation_type)
            .take(count)
            .collect()
    }

    /// Get performance statistics for an operation type
    pub fn get_performance_stats(&self, operation_type: &OperationType) -> PerformanceStats {
        let measurements: Vec<&PerformanceMeasurement> = self.measurements
            .iter()
            .filter(|m| m.operation_type == *operation_type)
            .collect();

        if measurements.is_empty() {
            return PerformanceStats::empty(*operation_type);
        }

        let durations: Vec<Duration> = measurements.iter().map(|m| m.duration).collect();
        let success_count = measurements.iter().filter(|m| m.success).count();
        let target_exceeded_count = measurements.iter().filter(|m| m.exceeded_target()).count();

        let total_ms: u64 = durations.iter().map(|d| d.as_millis() as u64).sum();
        let avg_duration = Duration::from_millis(total_ms / durations.len() as u64);

        let mut sorted_durations = durations.clone();
        sorted_durations.sort();
        let median_duration = sorted_durations[sorted_durations.len() / 2];

        let min_duration = *sorted_durations.first().unwrap();
        let max_duration = *sorted_durations.last().unwrap();

        PerformanceStats {
            operation_type: *operation_type,
            total_operations: measurements.len(),
            successful_operations: success_count,
            target_exceeded_count,
            avg_duration,
            median_duration,
            min_duration,
            max_duration,
            success_rate: success_count as f64 / measurements.len() as f64,
            target_compliance_rate: (measurements.len() - target_exceeded_count) as f64 / measurements.len() as f64,
        }
    }

    /// Get overall system performance summary
    pub fn get_system_performance_summary(&self) -> SystemPerformanceSummary {
        let operation_types = [
            OperationType::BuiltinCommand,
            OperationType::LocalOllama,
            OperationType::CloudProvider,
            OperationType::TotalSystem,
            OperationType::ContextGathering,
            OperationType::CommandValidation,
            OperationType::IntentClassification,
        ];

        let mut stats = HashMap::new();
        for op_type in &operation_types {
            stats.insert(*op_type, self.get_performance_stats(op_type));
        }

        let total_operations = self.measurements.len();
        let successful_operations = self.measurements.iter().filter(|m| m.success).count();
        let overall_success_rate = if total_operations > 0 {
            successful_operations as f64 / total_operations as f64
        } else {
            0.0
        };

        SystemPerformanceSummary {
            stats,
            total_operations,
            successful_operations,
            overall_success_rate,
        }
    }

    /// Check if system is performing within acceptable limits
    pub fn is_system_healthy(&self) -> bool {
        let summary = self.get_system_performance_summary();
        
        // System is healthy if:
        // 1. Overall success rate > 90%
        // 2. Critical operations (builtin, total system) have good compliance
        
        if summary.overall_success_rate < 0.9 {
            return false;
        }

        // Check critical operation compliance
        let critical_ops = [OperationType::BuiltinCommand, OperationType::TotalSystem];
        for op_type in &critical_ops {
            if let Some(stats) = summary.stats.get(op_type) {
                if stats.total_operations > 0 && stats.target_compliance_rate < 0.8 {
                    return false;
                }
            }
        }

        true
    }

    /// Clear all measurements (for testing or reset)
    pub fn clear_measurements(&mut self) {
        self.measurements.clear();
        self.active_timers.clear();
    }

    /// Update performance targets
    pub fn update_targets(&mut self, targets: PerformanceTargets) {
        self.targets = targets;
    }

    /// Get current targets
    pub fn get_targets(&self) -> &PerformanceTargets {
        &self.targets
    }
}

/// Performance statistics for a specific operation type
#[derive(Debug, Clone)]
pub struct PerformanceStats {
    pub operation_type: OperationType,
    pub total_operations: usize,
    pub successful_operations: usize,
    pub target_exceeded_count: usize,
    pub avg_duration: Duration,
    pub median_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub success_rate: f64,
    pub target_compliance_rate: f64,
}

impl PerformanceStats {
    fn empty(operation_type: OperationType) -> Self {
        Self {
            operation_type,
            total_operations: 0,
            successful_operations: 0,
            target_exceeded_count: 0,
            avg_duration: Duration::from_millis(0),
            median_duration: Duration::from_millis(0),
            min_duration: Duration::from_millis(0),
            max_duration: Duration::from_millis(0),
            success_rate: 0.0,
            target_compliance_rate: 0.0,
        }
    }

    /// Format duration for display
    pub fn format_duration(duration: Duration) -> String {
        if duration.as_millis() < 1000 {
            format!("{}ms", duration.as_millis())
        } else {
            format!("{:.2}s", duration.as_secs_f64())
        }
    }
}

/// System-wide performance summary
#[derive(Debug)]
pub struct SystemPerformanceSummary {
    pub stats: HashMap<OperationType, PerformanceStats>,
    pub total_operations: usize,
    pub successful_operations: usize,
    pub overall_success_rate: f64,
}

/// Timeout handler for graceful degradation
pub struct TimeoutHandler {
    total_timeout: Duration,
    start_time: Instant,
}

impl TimeoutHandler {
    /// Create a new timeout handler
    pub fn new(total_timeout: Duration) -> Self {
        Self {
            total_timeout,
            start_time: Instant::now(),
        }
    }

    /// Check if the total timeout has been exceeded
    pub fn is_expired(&self) -> bool {
        self.start_time.elapsed() >= self.total_timeout
    }

    /// Get remaining time
    pub fn remaining_time(&self) -> Duration {
        let elapsed = self.start_time.elapsed();
        if elapsed >= self.total_timeout {
            Duration::from_millis(0)
        } else {
            self.total_timeout - elapsed
        }
    }

    /// Get elapsed time
    pub fn elapsed_time(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Check if we have enough time for an operation
    pub fn has_time_for(&self, operation_duration: Duration) -> bool {
        self.remaining_time() >= operation_duration
    }

    /// Create a timeout for a specific operation within the remaining time
    pub fn timeout_for_operation(&self, max_duration: Duration) -> Duration {
        let remaining = self.remaining_time();
        if remaining < max_duration {
            remaining
        } else {
            max_duration
        }
    }
}

/// Graceful degradation strategies
#[derive(Debug, Clone, PartialEq)]
pub enum DegradationStrategy {
    SkipOperation,
    UseCache,
    UseFallback,
    ReduceQuality,
    TimeoutEarly,
}

/// Graceful degradation manager
pub struct DegradationManager {
    strategies: HashMap<OperationType, Vec<DegradationStrategy>>,
}

impl DegradationManager {
    /// Create a new degradation manager with default strategies
    pub fn new() -> Self {
        let mut strategies = HashMap::new();
        
        // Built-in commands: should never degrade (they're already fast)
        strategies.insert(OperationType::BuiltinCommand, vec![]);
        
        // Context gathering: can skip or use cache
        strategies.insert(OperationType::ContextGathering, vec![
            DegradationStrategy::UseCache,
            DegradationStrategy::SkipOperation,
        ]);
        
        // Command validation: can reduce quality or timeout early
        strategies.insert(OperationType::CommandValidation, vec![
            DegradationStrategy::ReduceQuality,
            DegradationStrategy::TimeoutEarly,
        ]);
        
        // Intent classification: can use fallback or skip
        strategies.insert(OperationType::IntentClassification, vec![
            DegradationStrategy::UseFallback,
            DegradationStrategy::SkipOperation,
        ]);
        
        // Local Ollama: can timeout early or use fallback
        strategies.insert(OperationType::LocalOllama, vec![
            DegradationStrategy::TimeoutEarly,
            DegradationStrategy::UseFallback,
        ]);
        
        // Cloud provider: can use fallback or timeout early
        strategies.insert(OperationType::CloudProvider, vec![
            DegradationStrategy::UseFallback,
            DegradationStrategy::TimeoutEarly,
        ]);
        
        Self { strategies }
    }

    /// Get degradation strategies for an operation type
    pub fn get_strategies(&self, operation_type: &OperationType) -> Vec<DegradationStrategy> {
        self.strategies.get(operation_type).cloned().unwrap_or_default()
    }

    /// Apply degradation strategy
    pub fn apply_strategy(&self, strategy: &DegradationStrategy, operation_type: &OperationType) -> String {
        match strategy {
            DegradationStrategy::SkipOperation => {
                format!("Skipping {:?} due to time constraints", operation_type)
            }
            DegradationStrategy::UseCache => {
                format!("Using cached results for {:?}", operation_type)
            }
            DegradationStrategy::UseFallback => {
                format!("Using fallback method for {:?}", operation_type)
            }
            DegradationStrategy::ReduceQuality => {
                format!("Reducing quality for {:?} to meet time constraints", operation_type)
            }
            DegradationStrategy::TimeoutEarly => {
                format!("Applying early timeout for {:?}", operation_type)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_performance_targets_default() {
        let targets = PerformanceTargets::default();
        assert_eq!(targets.builtin_command, Duration::from_millis(10));
        assert_eq!(targets.local_ollama, Duration::from_secs(2));
        assert_eq!(targets.cloud_provider, Duration::from_millis(800));
        assert_eq!(targets.total_system, Duration::from_secs(5));
    }

    #[test]
    fn test_operation_type_target_duration() {
        let targets = PerformanceTargets::default();
        assert_eq!(OperationType::BuiltinCommand.get_target_duration(&targets), Duration::from_millis(10));
        assert_eq!(OperationType::LocalOllama.get_target_duration(&targets), Duration::from_secs(2));
        assert_eq!(OperationType::CloudProvider.get_target_duration(&targets), Duration::from_millis(800));
        assert_eq!(OperationType::TotalSystem.get_target_duration(&targets), Duration::from_secs(5));
    }

    #[test]
    fn test_performance_measurement() {
        let measurement = PerformanceMeasurement::new(
            OperationType::BuiltinCommand,
            Duration::from_millis(5),
            Duration::from_millis(10),
            true
        );
        
        assert!(!measurement.exceeded_target());
        assert_eq!(measurement.performance_ratio(), 0.5);
        assert_eq!(measurement.format_duration(), "5ms");
        assert_eq!(measurement.format_target(), "10ms");
    }

    #[test]
    fn test_performance_measurement_exceeded() {
        let measurement = PerformanceMeasurement::new(
            OperationType::BuiltinCommand,
            Duration::from_millis(15),
            Duration::from_millis(10),
            true
        );
        
        assert!(measurement.exceeded_target());
        assert_eq!(measurement.performance_ratio(), 1.5);
    }

    #[test]
    fn test_performance_monitor_timer() {
        let mut monitor = PerformanceMonitor::new();
        
        monitor.start_timer("test_op".to_string(), OperationType::BuiltinCommand);
        thread::sleep(Duration::from_millis(1));
        let measurement = monitor.stop_timer("test_op", true).unwrap();
        
        assert!(measurement.success);
        assert!(measurement.duration >= Duration::from_millis(1));
        assert_eq!(measurement.operation_type, OperationType::BuiltinCommand);
    }

    #[test]
    fn test_performance_monitor_record_measurement() {
        let mut monitor = PerformanceMonitor::new();
        
        let measurement = monitor.record_measurement(
            OperationType::BuiltinCommand,
            Duration::from_millis(5),
            true
        );
        
        assert!(measurement.success);
        assert!(!measurement.exceeded_target());
        assert_eq!(monitor.measurements.len(), 1);
    }

    #[test]
    fn test_performance_stats() {
        let mut monitor = PerformanceMonitor::new();
        
        // Record some measurements
        monitor.record_measurement(OperationType::BuiltinCommand, Duration::from_millis(5), true);
        monitor.record_measurement(OperationType::BuiltinCommand, Duration::from_millis(8), true);
        monitor.record_measurement(OperationType::BuiltinCommand, Duration::from_millis(15), false); // Exceeded target
        
        let stats = monitor.get_performance_stats(&OperationType::BuiltinCommand);
        
        assert_eq!(stats.total_operations, 3);
        assert_eq!(stats.successful_operations, 2);
        assert_eq!(stats.target_exceeded_count, 1);
        assert_eq!(stats.success_rate, 2.0 / 3.0);
        assert_eq!(stats.target_compliance_rate, 2.0 / 3.0);
    }

    #[test]
    fn test_timeout_handler() {
        let handler = TimeoutHandler::new(Duration::from_millis(100));
        
        assert!(!handler.is_expired());
        assert!(handler.remaining_time() <= Duration::from_millis(100));
        assert!(handler.has_time_for(Duration::from_millis(50)));
        
        thread::sleep(Duration::from_millis(50));
        assert!(handler.elapsed_time() >= Duration::from_millis(50));
        assert!(handler.remaining_time() <= Duration::from_millis(50));
    }

    #[test]
    fn test_timeout_handler_expired() {
        let handler = TimeoutHandler::new(Duration::from_millis(1));
        thread::sleep(Duration::from_millis(2));
        
        assert!(handler.is_expired());
        assert_eq!(handler.remaining_time(), Duration::from_millis(0));
        assert!(!handler.has_time_for(Duration::from_millis(1)));
    }

    #[test]
    fn test_degradation_manager() {
        let manager = DegradationManager::new();
        
        let strategies = manager.get_strategies(&OperationType::ContextGathering);
        assert!(!strategies.is_empty());
        assert!(strategies.contains(&DegradationStrategy::UseCache));
        assert!(strategies.contains(&DegradationStrategy::SkipOperation));
        
        let builtin_strategies = manager.get_strategies(&OperationType::BuiltinCommand);
        assert!(builtin_strategies.is_empty()); // Built-in commands shouldn't degrade
    }

    #[test]
    fn test_system_performance_summary() {
        let mut monitor = PerformanceMonitor::new();
        
        // Record measurements for different operation types
        monitor.record_measurement(OperationType::BuiltinCommand, Duration::from_millis(5), true);
        monitor.record_measurement(OperationType::LocalOllama, Duration::from_millis(1500), true);
        monitor.record_measurement(OperationType::CloudProvider, Duration::from_millis(600), true);
        
        let summary = monitor.get_system_performance_summary();
        
        assert_eq!(summary.total_operations, 3);
        assert_eq!(summary.successful_operations, 3);
        assert_eq!(summary.overall_success_rate, 1.0);
        assert!(summary.stats.contains_key(&OperationType::BuiltinCommand));
        assert!(summary.stats.contains_key(&OperationType::LocalOllama));
        assert!(summary.stats.contains_key(&OperationType::CloudProvider));
    }

    #[test]
    fn test_system_health_check() {
        let mut monitor = PerformanceMonitor::new();
        
        // Record good measurements
        monitor.record_measurement(OperationType::BuiltinCommand, Duration::from_millis(5), true);
        monitor.record_measurement(OperationType::TotalSystem, Duration::from_secs(3), true);
        
        assert!(monitor.is_system_healthy());
        
        // Record many failed measurements
        for _ in 0..10 {
            monitor.record_measurement(OperationType::TotalSystem, Duration::from_secs(6), false);
        }
        
        assert!(!monitor.is_system_healthy());
    }
}