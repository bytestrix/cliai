# Implementation Plan: CLIAI Reliability Improvements

## Overview

This implementation plan transforms CLIAI from a prototype into a production-ready CLI tool through systematic reliability improvements. The approach prioritizes safety-first architecture, command validation, and OS awareness while maintaining the existing multi-agent pipeline. 

**Implementation Strategy:**
1. **Safety First**: Core validation and safety systems
2. **Format Enforcement**: Strict command format requirements  
3. **Built-in Commands**: Instant responses for common tasks (trust & speed)
4. **Configuration**: Safe defaults and execution modes
5. **OS Awareness**: Platform-specific behavior
6. **Advanced Features**: Context management, provider abstraction, professional UX
7. **Phase 2**: Authentication and monetization (separate milestone)

**Performance Budget:**
- Built-in commands: <10ms response time
- Local Ollama: <2s response time
- Cloud providers: <800ms response time  
- Total system timeout: 5s maximum

## Tasks

- [ ] 1. Implement Core Safety and Validation Infrastructure
  - [x] 1.1 Create Command Validator with hallucinated flag detection
    - Implement `CommandValidator` trait and `DefaultCommandValidator` struct
    - Add regex patterns for common hallucinated flags (--hidden, --recursivee, etc.)
    - Implement command rewriting for common mistakes
    - Add placeholder detection and rejection logic
    - _Requirements: 3.1, 3.2, 3.3, 3.4_

  - [ ]* 1.2 Write property test for command validation
    - **Property 7: Command Validation**
    - **Validates: Requirements 3.1, 3.2, 3.3, 3.4**

  - [x] 1.3 Implement Enhanced Safety Checker with token-aware parsing
    - Create `SafetyChecker` struct with shell token parser
    - Add fork bomb detection patterns (:(){ :|:& };:)
    - Add pipe-to-shell detection (curl | sh, wget | bash)
    - Implement dangerous operation detection (dd, mkfs, chmod 777)
    - Add severity levels and warning messages
    - _Requirements: 8.1, 8.2, 8.3, 8.4, 8.5_

  - [ ]* 1.4 Write property tests for safety checking
    - **Property 17: Fork Bomb Detection**
    - **Property 18: Pipe-to-Shell Detection**
    - **Property 19: Dangerous Operation Detection**
    - **Validates: Requirements 8.1, 8.2, 8.3, 8.4, 8.5**

- [ ] 2. Implement Command Format Enforcement
  - [x] 2.1 Update ShellExpert agent to enforce "Command: " format
    - Modify agent prompts to require strict format
    - Add response validation and retry logic
    - Implement "(none)" handling for non-executable requests
    - Add command line validation for operators (&&, ||, |)
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5_

  - [ ]* 2.2 Write property tests for command format enforcement
    - **Property 1: Command Format Validation**
    - **Property 2: Format Enforcement and Retry**
    - **Property 3: Command Combination Logic**
    - **Validates: Requirements 1.1, 1.2, 1.3, 1.4, 1.5**

  - [x] 2.3 Implement Copy-Paste Safe Output System
    - Create `CommandOutput` struct with strict formatting
    - Ensure commands and explanations are never mixed
    - Remove markdown formatting from command output
    - Add clear separation between executable commands and explanations
    - _Requirements: 16.1, 16.2, 16.3, 16.4, 16.5_

  - [ ]* 2.4 Write property test for output formatting
    - **Property 32: Copy-Paste Safe Output**
    - **Validates: Requirements 16.1, 16.2, 16.3, 16.4, 16.5**

- [ ] 3. Implement Built-in Command System (Early Trust & Speed)
  - [x] 3.1 Create built-in command mapping with strict patterns
    - Implement `BuiltinCommands` struct with 20 essential commands
    - Add exact-match patterns to prevent false positives
    - Implement command categories: file operations, system info, git basics
    - Add logging for built-in command usage
    - Ensure built-in commands respect auto_execute configuration
    - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5_

  - [ ]* 3.2 Write property tests for built-in commands
    - **Property 10: Built-in Command System**
    - **Property 11: Built-in Command Logging**
    - **Validates: Requirements 5.1, 5.2, 5.3, 5.4, 5.5**

  - [x] 3.3 Implement file existence checking consistency
    - Standardize on "test -f filename && echo 'exists' || echo 'not found'" pattern
    - Add proper quoting for paths with spaces
    - Support directory existence checking with "test -d"
    - Allow equivalent safe methods like stat when appropriate
    - Handle multiple file existence checks
    - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5_

  - [ ]* 3.4 Write property tests for file existence checking
    - **Property 12: File Existence Checking Consistency**
    - **Property 13: File Existence Method Flexibility**
    - **Property 14: Multiple File Existence Handling**
    - **Validates: Requirements 6.1, 6.2, 6.3, 6.4, 6.5**

- [x] 4. Checkpoint - Ensure core safety systems work
  - Ensure all tests pass and unresolved errors are logged.

- [x] 5. Implement Safe Default Configuration
  - [x] 5.1 Update configuration system with new safety options
    - Add `auto_execute: false` as default
    - Add `dry_run`, `safety_level`, `context_timeout` options
    - Implement configuration validation and error handling
    - Add immediate configuration application without restart
    - _Requirements: 2.1, 2.2, 15.1, 15.2, 15.3, 15.4, 15.5_

  - [ ]* 5.2 Write property tests for configuration management
    - **Property 4: Safe Default Configuration**
    - **Property 31: Configuration Management**
    - **Validates: Requirements 2.1, 2.2, 15.1, 15.2, 15.3, 15.4, 15.5**

  - [x] 5.3 Implement execution mode system
    - Create `ExecutionMode` enum with SuggestOnly, Safe, RequiresConfirmation, DryRunOnly, Blocked
    - Update command execution logic to respect auto_execute setting
    - Add confirmation prompts for sensitive commands even when auto_execute is true
    - Implement dry-run mode with "DRY RUN:" prefix
    - _Requirements: 2.3, 2.4, 2.5, 17.1, 17.2, 17.3, 17.4, 17.5_

  - [ ]* 5.4 Write property tests for execution modes
    - **Property 5: Auto-execution Behavior**
    - **Property 6: Configuration Modification**
    - **Property 33: Dry-Run Mode**
    - **Validates: Requirements 2.3, 2.4, 2.5, 17.1, 17.2, 17.3, 17.4, 17.5**

- [x] 6. Implement OS Awareness System
  - [x] 6.1 Create OS Context Manager with detection logic
    - Implement `OSContext` struct with OS detection
    - Add support for Arch Linux, Ubuntu, Debian (primary focus)
    - Implement package manager detection (pacman, apt)
    - Add OS-specific path and command handling
    - Cache OS context for performance
    - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5_

  - [ ]* 6.2 Write property tests for OS detection and awareness
    - **Property 8: OS Detection and Context**
    - **Property 9: OS-Aware Command Generation**
    - **Validates: Requirements 4.1, 4.2, 4.3, 4.4, 4.5**

  - [x] 6.3 Implement Arch Linux native support
    - Add pacman-specific command generation
    - Implement Arch-specific paths and conventions
    - Add AUR installation instruction support
    - Update system information commands for Arch
    - _Requirements: 14.1, 14.2, 14.3, 14.4, 14.5_

  - [ ]* 6.4 Write property test for Arch Linux support
    - **Property 30: Arch Linux Native Support**
    - **Validates: Requirements 14.1, 14.2, 14.3, 14.4, 14.5**

- [x] 7. Checkpoint - Ensure built-ins and format enforcement work
  - Ensure all tests pass and unresolved errors are logged.

- [x] 8. Implement Context Management System
  - [x] 8.1 Create safe context gathering with whitelisted commands
    - Implement context command whitelist (uname -a, cat /etc/os-release, pwd, whoami)
    - Add configurable timeout for context commands (default 2 seconds)
    - Implement graceful failure handling for context gathering
    - Ensure context is included in AI prompts
    - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5_

  - [ ]* 8.2 Write property tests for context gathering
    - **Property 15: Safe Context Gathering**
    - **Property 16: Context Integration**
    - **Validates: Requirements 7.1, 7.2, 7.3, 7.4, 7.5**

  - [x] 8.3 Implement context window management for agents
    - Limit ShellExpert to last 3 turns plus working directory context
    - Provide full history to General agents
    - Implement automatic history truncation after 10 turns
    - Add context prioritization when size limits are reached
    - _Requirements: 9.1, 9.2, 9.3, 9.4, 9.5_

  - [ ]* 8.4 Write property tests for context window management
    - **Property 20: Context Window Management**
    - **Property 21: History Truncation**
    - **Validates: Requirements 9.1, 9.2, 9.3, 9.4, 9.5**

- [x] 9. Implement AI Provider Abstraction
  - [x] 9.1 Create provider trait and implementations
    - Implement `AIProvider` trait with common interface
    - Create `OllamaProvider` for local models
    - Create `CloudProvider` for hosted models
    - Add provider availability checking
    - _Requirements: 20.1, 20.5_

  - [ ]* 9.2 Write property tests for provider abstraction
    - **Property 37: Provider Abstraction**
    - **Property 38: Provider Interface Completeness**
    - **Validates: Requirements 20.1, 20.5**

  - [x] 9.3 Implement provider manager with fallback logic
    - Create `ProviderManager` with retry limits and circuit breaker
    - Implement deterministic fallback chain (Cloud → Local)
    - Add immediate fallback after 1 retry for cloud failures
    - Support runtime provider switching based on configuration
    - _Requirements: 20.2, 20.3, 20.4_

  - [ ]* 9.4 Write property tests for provider fallback
    - **Property 37: Provider Abstraction** (fallback aspects)
    - **Validates: Requirements 20.2, 20.3, 20.4**

- [x] 10. Implement Intent Classification and Safety
  - [x] 10.1 Add intent classification for explanatory vs actionable requests
    - Distinguish between "how to" questions and "do this" requests
    - Prevent destructive commands for explanatory requests
    - Add clarification prompts for ambiguous intent
    - Implement explicit confirmation for destructive actions
    - _Requirements: 18.1, 18.2, 18.3, 18.4, 18.5_

  - [ ]* 10.2 Write property tests for intent classification
    - **Property 34: Intent Classification**
    - **Property 35: Destructive Command Handling**
    - **Validates: Requirements 18.1, 18.2, 18.3, 18.4, 18.5**

  - [x] 10.3 Implement quoting and escaping correctness
    - Add proper quoting for file paths with spaces
    - Implement escaping for special characters
    - Avoid ambiguous shell globbing patterns
    - Ensure proper variable quoting to prevent injection
    - Add automatic correction of common quoting mistakes
    - _Requirements: 19.1, 19.2, 19.3, 19.4, 19.5_

  - [ ]* 10.4 Write property test for quoting correctness
    - **Property 36: Quoting and Escaping Correctness**
    - **Validates: Requirements 19.1, 19.2, 19.3, 19.4, 19.5**

- [x] 11. Checkpoint - Ensure advanced safety features work
  - Ensure all tests pass and unresolved errors are logged.

- [-] 12. Implement Local-First Architecture
  - [x] 12.1 Ensure complete offline functionality
    - Verify core command generation works without internet
    - Implement graceful fallback when cloud services unavailable
    - Ensure local mode never requires internet connectivity
    - _Requirements: 12.1, 12.2, 12.4_

  - [ ]* 12.2 Write property test for offline functionality
    - **Property 26: Offline Functionality**
    - **Validates: Requirements 12.1, 12.2, 12.4**

- [x] 13. Implement Professional UX Features
  - [x] 13.1 Enhance error handling and user experience
    - Implement clear, actionable error messages with suggested solutions
    - Maintain simple `cliai <natural language task>` interface
    - Ensure consistent behavior across terminal environments
    - Add immediate feedback for configuration changes
    - _Requirements: 13.1, 13.2, 13.4, 13.5_

  - [ ]* 13.2 Write property tests for professional UX
    - **Property 28: Interface Simplicity**
    - **Property 29: Error Handling Quality**
    - **Validates: Requirements 13.1, 13.2, 13.4, 13.5**

  - [x] 13.3 Implement comprehensive test suite with expected patterns
    - Create test suite with expected command patterns for 50 test questions
    - Add regex validation for generated commands
    - Implement detection of hallucinated flags in tests
    - Add safe command execution and result verification
    - Provide detailed failure analysis for debugging
    - _Requirements: 10.1, 10.2, 10.3, 10.4, 10.5_

  - [ ]* 13.4 Write property test for test suite validation
    - **Property 22: Test Suite Validation**
    - **Validates: Requirements 10.1, 10.2, 10.3, 10.4, 10.5**

- [-] 14. Implement Privacy-First Logging
  - [x] 14.1 Create privacy-preserving logging system
    - Implement error logging to ~/.config/cliai/error.log
    - Ensure commands and prompts are redacted by default
    - Add structured logging with context information
    - Implement debug mode with explicit user consent
    - Add clear marking for debug logs
    - _Requirements: Error Handling section_

  - [ ]* 14.2 Write unit tests for logging privacy
    - Test that user input is never logged in production mode
    - Test that debug mode requires explicit consent
    - Test log redaction functionality
    - _Requirements: Error Handling section_

- [-] 15. Implement Performance Budget System
  - [x] 15.1 Add performance monitoring and timeouts
    - Built-in command responses: <10ms target
    - Local Ollama responses: <2s target  
    - Cloud responses: <800ms target
    - Total system timeout: 5s maximum
    - Add performance logging and monitoring
    - Implement timeout handling with graceful degradation
    - _Requirements: Performance and reliability_

  - [ ]* 15.2 Write performance tests
    - Test built-in command response times
    - Test timeout handling for slow providers
    - Test graceful degradation when timeouts occur
    - _Requirements: Performance and reliability_

- [x] 16. Integration and Final Wiring
  - [x] 16.1 Integrate all components into main orchestrator
    - Wire command validator into orchestrator pipeline
    - Integrate safety checker with execution logic
    - Connect OS context to all AI agents
    - Wire provider manager with fallback logic
    - Connect built-in commands to main processing flow
    - _Requirements: All requirements integration_

  - [ ]* 16.2 Write integration tests for complete workflows
    - Test end-to-end command processing pipeline
    - Test provider fallback scenarios
    - Test safety checking integration
    - Test OS-aware command generation
    - _Requirements: All requirements integration_

  - [x] 16.3 Update existing agent profiles with new constraints
    - Update ShellExpert with strict command format requirements
    - Add OS context to all agent system prompts
    - Implement context window limits for different agents
    - Add safety constraints to agent responses
    - _Requirements: Agent system integration_

- [x] 17. Final checkpoint - Ensure complete system works
  - Ensure all tests pass and unresolved errors are logged.

## Phase 2: Monetization Features (Future Milestone)

- [ ] 18. Implement Authentication and Pro Features
  - [ ] 18.1 Implement device-code authentication flow
    - Add "cliai login" command with device-code flow
    - Display device code and URL for browser authentication
    - Implement secure token storage after successful authentication
    - Add subscription status display in configuration
    - _Requirements: 11.1, 11.2, 11.4_

  - [ ]* 18.2 Write unit tests for authentication flow
    - Test device-code generation and validation
    - Test token storage and retrieval
    - Test subscription status display
    - _Requirements: 11.1, 11.2, 11.4_

  - [ ] 18.3 Implement Pro mode with cloud provider integration
    - Add provider selection based on subscription status
    - Implement privacy-preserving usage tracking (aggregate metrics only)
    - Add graceful fallback from Pro to Local mode
    - Preserve user configuration and history during mode switches
    - _Requirements: 11.3, 11.5, 12.3, 12.5_

  - [ ]* 18.4 Write property tests for Pro mode features
    - **Property 24: Provider Selection**
    - **Property 25: Privacy-Preserving Usage Tracking**
    - **Property 27: Mode Switching Preservation**
    - **Validates: Requirements 11.3, 11.5, 12.3, 12.5**

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation throughout development
- Property tests validate universal correctness properties with minimum 100 iterations
- Unit tests validate specific examples, edge cases, and integration points
- Implementation prioritizes safety-first architecture with fail-safe defaults
- The existing multi-agent pipeline is preserved and enhanced rather than replaced
- **Cloud Provider Testing**: OpenAI API key available in .env file for testing cloud provider integration
- Authentication and monetization features moved to Phase 2 to focus on core reliability first