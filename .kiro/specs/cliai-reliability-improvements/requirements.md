# Requirements Document

## Introduction

CLIAI (Command Line AI) is a Rust-based CLI tool that provides intelligent terminal assistance through AI agents. This specification addresses critical reliability issues and adds professional features to transform CLIAI from a prototype into a production-ready developer tool. The improvements focus on command format enforcement, safety mechanisms, OS awareness, and monetization features while maintaining the simple `cliai <natural language task>` interface.

## Glossary

- **CLIAI**: The Command Line AI tool and main system
- **ShellExpert**: The AI agent responsible for generating shell commands
- **Orchestrator**: The main coordination component that routes requests to appropriate agents
- **Command_Validator**: A new component that validates commands before execution
- **OS_Context**: System information gathered at startup for OS-aware responses
- **Built_in_Commands**: Hardcoded command mappings for common tasks
- **Auto_Execute**: Configuration flag controlling automatic command execution
- **Sensitive_Command**: Commands that could cause system damage or data loss
- **Context_Window**: The conversation history provided to AI agents
- **Test_Suite**: Automated validation system for command correctness
- **Local_Mode**: Free tier using local Ollama models
- **Pro_Mode**: Paid tier using hosted cloud models

## Requirements

### Requirement 1: Command Format Enforcement

**User Story:** As a developer, I want ShellExpert to always output commands in a consistent format, so that I can reliably parse and execute them.

#### Acceptance Criteria

1. WHEN ShellExpert generates a response, THE System SHALL ensure it starts with "Command: " followed by a single valid command
2. WHEN ShellExpert provides a response without the "Command: " prefix, THE System SHALL reject the response and request a properly formatted one
3. WHEN multiple commands are needed, THE System SHALL combine them using shell operators (&&, ||, |) in a single "Command: " line
4. WHEN no executable command is appropriate, THE ShellExpert SHALL respond with "Command: (none)" followed by a plain text explanation, and THE System SHALL never attempt to execute "(none)"
5. THE System SHALL validate that the text after "Command: " contains one command line including operators (|, &&, ||, ;)

### Requirement 2: Safe Default Behavior

**User Story:** As a security-conscious developer, I want commands to be shown for manual confirmation by default, so that I can review them before execution.

#### Acceptance Criteria

1. WHEN a user runs CLIAI without explicit auto-execution configuration, THE System SHALL display the command without executing it
2. WHEN the configuration file is created, THE System SHALL set "auto_execute": false by default
3. WHEN auto_execute is false, THE System SHALL display the command with instructions for manual execution
4. WHEN a user wants to enable auto-execution, THE System SHALL provide a configuration option to set "auto_execute": true
5. WHEN auto_execute is true, THE System SHALL still prompt for confirmation on sensitive commands

### Requirement 3: Command Validation Layer

**User Story:** As a user, I want the system to validate commands before execution, so that I don't encounter errors from invalid syntax or options.

#### Acceptance Criteria

1. WHEN a command is generated, THE Command_Validator SHALL check for unknown command options and flags
2. WHEN placeholder text like "/path/to/file" is detected, THE Command_Validator SHALL reject the command and request a specific version
3. WHEN common command mistakes are detected, THE Command_Validator SHALL automatically rewrite them to correct versions
4. WHEN a command uses non-standard flags, THE Command_Validator SHALL replace them with widely-supported alternatives
5. THE Command_Validator SHALL maintain a focused ruleset for common command hallucinations and unsafe patterns

### Requirement 4: Operating System Awareness

**User Story:** As an Arch Linux user, I want CLIAI to understand my operating system, so that it provides OS-appropriate commands and paths.

#### Acceptance Criteria

1. WHEN CLIAI starts up, THE System SHALL detect the operating system and store it in OS_Context
2. WHEN generating commands, THE ShellExpert SHALL use OS-appropriate package managers (pacman for Arch, apt for Ubuntu)
3. WHEN referencing system paths, THE System SHALL use OS-appropriate locations (/etc/arch-release vs /etc/lsb-release)
4. WHEN suggesting installation commands, THE System SHALL use the correct package manager for the detected OS
5. THE OS_Context SHALL be included in the system prompt for all AI agents

### Requirement 5: Built-in Command Overrides

**User Story:** As a user performing common tasks, I want instant responses for repetitive commands, so that I don't wait for AI processing on simple requests.

#### Acceptance Criteria

1. THE System SHALL maintain a hardcoded mapping of 20 essential command patterns to their shell equivalents
2. WHEN a user request matches a built-in pattern, THE System SHALL return the mapped command immediately without AI processing
3. WHEN built-in commands are used, THE System SHALL log the activity but skip AI agent calls
4. THE Built_in_Commands SHALL include file listing, directory creation, file existence checks, and basic system information
5. WHEN a built-in command is executed, THE System SHALL still respect the auto_execute configuration

### Requirement 6: File Existence Validation

**User Story:** As a user checking file existence, I want consistent and reliable file existence commands, so that I get accurate results every time.

#### Acceptance Criteria

1. WHEN a user asks about file existence, THE ShellExpert SHALL always use the pattern "test -f filename && echo 'exists' || echo 'not found'"
2. WHEN checking directory existence, THE System SHALL use "test -d dirname && echo 'exists' || echo 'not found'"
3. THE System SHALL prefer "test -f filename && echo 'exists' || echo 'not found'" but allow equivalent safe methods like stat
4. THE System SHALL handle file paths with spaces by properly quoting them in the test command
5. WHEN multiple files need checking, THE System SHALL provide separate test commands for each file

### Requirement 7: Safe Context Commands

**User Story:** As a user, I want the system to gather helpful context information safely, so that AI responses are informed without risking system damage.

#### Acceptance Criteria

1. THE System SHALL maintain a whitelist of safe context-gathering commands (uname -a, cat /etc/os-release, pwd, whoami)
2. WHEN gathering context, THE System SHALL never execute commands that modify files or system state
3. WHEN context commands fail, THE System SHALL continue operation without the failed context information
4. THE System SHALL limit context gathering to read-only operations with configurable timeout (default 2 seconds)
5. WHEN context is gathered, THE System SHALL include it in the AI prompt to improve response accuracy

### Requirement 8: Enhanced Sensitive Command Detection

**User Story:** As a user, I want comprehensive protection against dangerous commands, so that I'm warned about all potentially harmful operations.

#### Acceptance Criteria

1. THE System SHALL detect fork bombs (patterns like :(){ :|:& };:) and warn users
2. WHEN pipe-to-shell patterns (curl | sh, wget | bash) are detected, THE System SHALL classify them as sensitive
3. WHEN disk write operations (dd, mkfs, fdisk) are suggested, THE System SHALL require explicit confirmation
4. THE System SHALL expand the sensitive command list to include chmod 777, chown -R, and recursive deletions
5. WHEN sensitive commands are detected, THE System SHALL display clear warnings about potential consequences

### Requirement 9: Context Window Management

**User Story:** As a user, I want appropriate conversation history for different agents, so that responses are contextually relevant without overwhelming the AI.

#### Acceptance Criteria

1. WHEN calling ShellExpert, THE System SHALL provide the last 3 conversation turns plus current working directory context
2. WHEN calling General agents, THE System SHALL provide the full conversation history for context continuity
3. WHEN the conversation history exceeds 10 turns, THE System SHALL automatically truncate older entries
4. THE System SHALL ensure that context provided to agents includes relevant system information
5. WHEN context becomes too large, THE System SHALL prioritize recent interactions over older ones

### Requirement 10: Test Suite Validation

**User Story:** As a developer, I want automated testing that verifies command correctness, so that I can ensure CLIAI reliability across updates.

#### Acceptance Criteria

1. THE Test_Suite SHALL include expected command patterns for each of the 50 test questions
2. WHEN running tests, THE System SHALL validate that generated commands match expected patterns using regex
3. WHEN commands contain hallucinated flags or options, THE Test_Suite SHALL mark them as failures
4. THE Test_Suite SHALL execute safe commands and verify their output matches expected results
5. WHEN test results are generated, THE System SHALL provide detailed failure analysis for debugging

### 11. Authentication and Monetization

**User Story:** As a user, I want to upgrade to premium features through simple authentication, so that I can access better AI models and faster responses.

#### Acceptance Criteria

1. WHEN a user runs "cliai login", THE System SHALL display a device code and URL for browser-based authentication
2. WHEN authentication is successful, THE System SHALL store secure tokens for API access
3. WHEN a user has Pro subscription, THE System SHALL use hosted cloud models instead of local Ollama
4. THE System SHALL provide clear indication of current subscription status in configuration display
5. THE System SHALL track only aggregate API usage metrics for billing, never command content or user data

**Authentication Flow:**
Preferred login UX uses device-code flow (like GitHub CLI):
```
$ cliai login
Go to: https://cliai.org/device
Enter code: PENG-UIN42
Waiting for authentication...
✓ Successfully authenticated as user@example.com
```

### Requirement 12: Local-First Architecture

**User Story:** As a developer, I want full functionality without internet connection, so that I can use CLIAI in any environment.

#### Acceptance Criteria

1. THE System SHALL work completely offline using local Ollama models in Local_Mode
2. WHEN internet is unavailable, THE System SHALL gracefully fall back to local processing
3. WHEN Pro_Mode is enabled but unavailable, THE System SHALL automatically use Local_Mode with user notification
4. THE System SHALL never require internet connectivity for core command generation functionality
5. WHEN switching between modes, THE System SHALL preserve user configuration and history

### Requirement 13: Professional User Experience

**User Story:** As a professional developer, I want a polished and reliable CLI experience, so that I can integrate CLIAI into my daily workflow.

#### Acceptance Criteria

1. THE System SHALL maintain the simple interface: `cliai <natural language task>` without requiring quotes or subcommands
2. WHEN errors occur, THE System SHALL provide clear, actionable error messages with suggested solutions
3. THE System SHOULD respond to common tasks within reasonable time depending on model size and hardware capabilities
4. THE System SHALL maintain consistent behavior across different terminal environments and shells
5. WHEN configuration changes are made, THE System SHALL provide immediate feedback confirming the changes

### Requirement 14: Arch Linux Native Support

**User Story:** As an Arch Linux user, I want native support for my distribution, so that all suggestions and commands work correctly in my environment.

#### Acceptance Criteria

1. WHEN running on Arch Linux, THE System SHALL use pacman for package management suggestions
2. WHEN referencing system files, THE System SHALL use Arch-specific paths and conventions
3. THE System SHALL understand Arch Linux package naming conventions and suggest correct package names
4. WHEN system information is needed, THE System SHALL use Arch-appropriate commands and file locations
5. THE System SHALL provide installation instructions specific to Arch Linux and AUR

### Requirement 15: Configuration Management

**User Story:** As a user, I want comprehensive configuration options, so that I can customize CLIAI behavior to match my preferences.

#### Acceptance Criteria

1. THE System SHALL store configuration in a standard location (~/.config/cliai/config.json)
2. WHEN configuration is invalid or corrupted, THE System SHALL recreate it with safe defaults
3. THE System SHALL provide commands to view, modify, and reset configuration settings
4. WHEN configuration changes affect behavior, THE System SHALL apply them immediately without restart
5. THE System SHALL validate configuration values and reject invalid settings with helpful error messages

### Requirement 16: Copy-Paste Safe Output

**User Story:** As a developer, I want command output that I can directly copy and paste, so that I can execute commands without manual cleanup.

#### Acceptance Criteria

1. WHEN ShellExpert provides a command, THE System SHALL output only the command text without markdown formatting
2. THE System SHALL not include backticks, asterisks, or other formatting characters in command output
3. WHEN displaying commands for manual execution, THE System SHALL present them in a format ready for terminal pasting
4. THE System SHALL separate explanatory text from executable commands clearly
5. WHEN multiple commands are suggested, THE System SHALL present them on separate lines only if they are independent operations

### Requirement 17: Dry-Run Mode

**User Story:** As a cautious user, I want to preview what CLIAI would do without executing anything, so that I can verify behavior safely.

#### Acceptance Criteria

1. THE System SHALL support a dry-run mode that shows commands without executing them
2. WHEN dry-run mode is enabled, THE System SHALL display "DRY RUN:" prefix before all command suggestions
3. THE System SHALL provide internal dry-run capability for testing and validation
4. WHEN in dry-run mode, THE System SHALL still perform all validation and processing steps
5. THE System SHALL allow toggling dry-run mode through configuration or command-line flags

### Requirement 18: No Implicit Side Effects

**User Story:** As a user, I want CLIAI to never suggest destructive commands unless I explicitly request them, so that I can safely ask for explanations without risk.

#### Acceptance Criteria

1. WHEN a user asks for explanation of a process, THE System SHALL not generate commands that modify the system
2. WHEN user intent is unclear about destructive actions, THE System SHALL ask for clarification before suggesting commands
3. WHEN destructive actions are explicitly requested, THE System SHALL always provide clear warnings about consequences
4. THE System SHALL distinguish between "how to" questions (explanatory) and "do this" requests (actionable)
5. WHEN suggesting potentially destructive commands, THE System SHALL require explicit confirmation of intent

### Requirement 19: Quoting and Escaping Correctness

**User Story:** As a user working with files that have spaces or special characters, I want commands that handle quoting correctly, so that they execute without syntax errors.

#### Acceptance Criteria

1. WHEN generating commands with file paths containing spaces, THE ShellExpert SHALL properly quote the paths
2. WHEN file names contain special characters, THE System SHALL use appropriate escaping or quoting
3. THE System SHALL avoid ambiguous shell globbing patterns unless explicitly required by user intent
4. WHEN generating commands with variables or expansions, THE System SHALL ensure proper quoting to prevent injection
5. THE Command_Validator SHALL check for common quoting mistakes and correct them automatically

### Requirement 20: Provider Abstraction

**User Story:** As a developer, I want CLIAI to support multiple AI backends through a clean interface, so that new providers can be added without major code changes.

#### Acceptance Criteria

1. THE System SHALL implement AI providers through a common trait or interface (OllamaProvider, CloudProvider)
2. WHEN adding a new AI provider, THE System SHALL not require changes to the Orchestrator core logic
3. THE System SHALL allow runtime switching between providers based on configuration
4. WHEN a provider fails, THE System SHALL gracefully fall back to alternative providers if available
5. THE Provider interface SHALL abstract model selection, request formatting, and response parsing