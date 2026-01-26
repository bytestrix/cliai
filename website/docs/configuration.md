---
sidebar_position: 3
---

# Configuration Guide

This guide covers all configuration options available in CLIAI.

## 📁 Configuration Files

CLIAI uses a hierarchical configuration system:

1. **System defaults** (built into the application)
2. **User configuration** (`~/.config/cliai/config.toml`)
3. **Environment variables**
4. **Command-line flags** (highest priority)

### Configuration File Location

- **Linux/macOS**: `~/.config/cliai/config.toml`
- **Windows**: `%APPDATA%\cliai\config.toml`

## ⚙️ Configuration Options

### Basic Configuration

```toml
# ~/.config/cliai/config.toml

# Default AI model to use (for local Ollama)
model = "mistral"

# Custom command prefix (optional)
prefix = "cliai"

# Ollama server URL
ollama_url = "http://localhost:11434"

# Auto-execute safe commands
auto_execute = false

# Dry-run mode (show commands without executing)
dry_run = false

# Safety level: "low", "medium", "high"
safety_level = "medium"

# Context gathering timeout (milliseconds)
context_timeout = 5000

# Enable debug logging (requires user consent)
debug_mode = false
```

## 🔐 Professional Features & Subscription

CLIAI uses a platform-managed approach for professional online models. Direct cloud API keys (like OpenAI or Azure) are not supported to ensure security and platform sustainability.

### Login to CLIAI

To activate professional models and sync your subscription status:

```bash
cliai login
```

This command will open your browser to the CLIAI platform where you can sign in and manage your professional features.

## 🌍 Environment Variables

### Ollama Configuration

```bash
# Set a custom Ollama host
export OLLAMA_HOST="http://localhost:11434"
```

### CLIAI-Specific Variables

```bash
# Enable debug mode
export CLIAI_DEBUG=true

# Override config file location
export CLIAI_CONFIG_DIR="/custom/path"

# Set log level
export RUST_LOG=cliai=debug

# Disable color output
export NO_COLOR=1
```

## 🛡️ Safety Configuration

### Safety Levels

#### High Safety (Recommended)
- Blocks dangerous commands entirely
- Requires confirmation for system modifications
- Extensive command validation

```bash
cliai safety-level high
```

#### Medium Safety (Default)
- Confirms risky operations
- Allows most commands with warnings
- Balanced protection and usability

#### Low Safety (Experienced users)
- Minimal safety checks
- Trusts user judgment
- Maximum flexibility

## 📊 Logging Configuration

Enable debug mode with user consent:

```bash
# Enable debug logging
cliai debug-mode --enable

# Disable debug logging
cliai debug-mode --disable

# Check debug status
cliai log-status
```

## 🔄 Configuration Management

### Modify Configuration

```bash
# Set default model
cliai select mistral

# Set custom prefix
cliai set-prefix ai

# Enable auto-execution
cliai auto-execute --enable

# Set safety level
cliai safety-level high

# Set context timeout
cliai context-timeout 3000
```