---
sidebar_position: 4
---

# Usage Guide

Learn how to use CLIAI effectively with this comprehensive guide.

## 🚀 Basic Usage

### Simple Questions
```bash
cliai "how do I list files?"
# Output: ls -la
# 💡 Lists all files including hidden ones with detailed information

cliai "what's my IP address?"
# Output: curl -s ifconfig.me
# 💡 Shows your public IP address
```

### System Administration
```bash
cliai "check disk usage"
# Output: df -h
# 💡 Shows disk usage in human-readable format

cliai "find large files"
# Output: find . -type f -size +100M -exec ls -lh {} \;
# 💡 Finds files larger than 100MB in current directory
```

### File Operations
```bash
cliai "compress this folder"
# Output: tar -czf folder.tar.gz folder/
# 💡 Creates a compressed archive of the folder

cliai "extract zip file"
# Output: unzip filename.zip
# 💡 Extracts contents of a zip file
```

## ⚙️ Configuration Commands

### Model Management
```bash
# List available models
cliai list-models

# Switch to a different model
cliai select mistral

# Check provider status
cliai provider-status
```

### Safety Settings
```bash
# Set safety level
cliai safety-level high    # Maximum safety
cliai safety-level medium  # Balanced (default)
cliai safety-level low     # Minimal checks

# Enable/disable auto-execution
cliai auto-execute on      # Execute safe commands automatically
cliai auto-execute off     # Always show commands first

# Enable dry-run mode
cliai dry-run on          # Show commands but never execute
cliai dry-run off         # Normal execution behavior
```

### Performance Tuning
```bash
# Set timeouts
cliai context-timeout 5000    # 5 seconds for context gathering
cliai ai-timeout 120000       # 2 minutes for AI responses

# Check performance status
cliai performance-status
```

## 🛡️ Safety Features

### Command Validation
CLIAI validates all commands through multiple layers:

1. **Syntax Checking**: Ensures valid command syntax
2. **Risk Assessment**: Categorizes commands by potential impact
3. **Placeholder Detection**: Catches AI hallucinations
4. **User Confirmation**: Prompts for dangerous operations

### Safety Levels

#### High Safety
- Blocks dangerous commands entirely
- Requires confirmation for system changes
- Maximum protection for beginners

#### Medium Safety (Default)
- Balanced protection
- Confirms risky operations
- Good for most users

#### Low Safety
- Minimal validation
- Allows most commands
- For experienced users only

## 🎯 Advanced Features

### Custom Prefix
Set a custom command prefix for easier access:

```bash
cliai set-prefix jarvis
# Now you can use: jarvis "list running processes"
```

### Execution Modes

#### Auto-Execute Mode
```bash
cliai auto-execute on
cliai "list files"
# Command executes automatically if deemed safe
```

#### Dry-Run Mode
```bash
cliai dry-run on
cliai "delete old logs"
# Shows: DRY RUN: find /var/log -name "*.log" -mtime +30 -delete
# Never actually executes the command
```

#### Manual Mode (Default)
```bash
cliai "compress folder"
# Shows command and explanation
# You copy and paste to execute
```

### Context Awareness

CLIAI automatically gathers system context:
- Current working directory
- Operating system and distribution
- Available package managers
- Shell type and version
- System architecture

## 📊 Testing and Validation

### Built-in Test Suite
```bash
# Run comprehensive tests
cliai test

# Run specific categories
cliai test --categories "file-management,system-info"

# Quick validation
cliai test --quick

# Save results
cliai test --save results.md
```

### Test Categories
- **File Management**: File operations, permissions, searching
- **System Info**: System monitoring, process management
- **Git Operations**: Version control commands
- **Network**: Connectivity, downloads, API calls
- **Programming**: Development tools, compilation
- **Process Management**: Service control, monitoring

## 🔧 Troubleshooting Commands

### Debug Mode
```bash
# Enable debug logging (requires consent)
cliai debug-mode --enable

# Check log status
cliai log-status

# Clear logs
cliai clear-logs
```

### Provider Issues
```bash
# Check provider status
cliai provider-status

# List available models
cliai list-models

# Test with a simple command
cliai "echo hello"
```

### Configuration Issues
```bash
# Show current configuration
cliai config

# Reset to defaults (backup first!)
rm ~/.config/cliai/config.json
cliai config  # Will recreate with defaults
```

## 💡 Tips and Best Practices

### 1. Start with High Safety
```bash
cliai safety-level high
# Learn CLIAI's behavior before reducing safety
```

### 2. Use Descriptive Prompts
```bash
# Good
cliai "find all Python files modified in the last week"

# Less specific
cliai "find files"
```

### 3. Leverage Context
```bash
# CLIAI knows your OS and package manager
cliai "install vim"
# On Arch: sudo pacman -S vim
# On Ubuntu: sudo apt install vim
```

### 4. Test Before Production
```bash
# Use dry-run for dangerous operations
cliai dry-run on
cliai "delete all log files older than 30 days"
# Review the command before executing
```

### 5. Regular Testing
```bash
# Validate your setup periodically
cliai test --quick
```

## 🚨 Common Pitfalls

### 1. Auto-Execute with Low Safety
- **Risk**: Dangerous commands may execute automatically
- **Solution**: Use medium or high safety with auto-execute

### 2. Ignoring Warnings
- **Risk**: Missing important safety information
- **Solution**: Read all warnings and explanations

### 3. Not Testing Commands
- **Risk**: Unexpected behavior in production
- **Solution**: Use dry-run mode for unfamiliar commands

### 4. Outdated Models
- **Risk**: Poor command suggestions
- **Solution**: Keep Ollama models updated

## 📚 Next Steps

- [Configuration Guide](./configuration) - Customize CLIAI settings
- [Safety & Security](./safety) - Understand safety features
- [Troubleshooting](./troubleshooting) - Solve common issues
- [Architecture](./architecture) - Learn how CLIAI works