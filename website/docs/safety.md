---
sidebar_position: 5
---

# Safety & Security

CLIAI is designed with safety as a core principle. This guide explains all safety features and best practices.

## 🛡️ Safety Philosophy

CLIAI follows a **safety-first** approach:

1. **Local-First**: Your data never leaves your machine by default
2. **Multi-Layer Validation**: Commands are checked at multiple levels
3. **Fail-Safe Defaults**: Conservative settings out of the box
4. **User Control**: You decide what level of automation to allow
5. **Transparency**: All actions are explained and logged

## 🔒 Privacy Protection

### Local AI Processing
- **Primary**: Ollama runs locally on your machine
- **No Data Transmission**: Commands and responses stay local
- **Offline Capable**: Works without internet connection
- **No Telemetry**: No usage data is collected

### Professional Features
- **Secure Platform**: Professional models accessed through secure CLIAI platform
- **No Direct API Keys**: Users cannot set their own cloud API keys
- **Encrypted Communication**: All professional features use encrypted connections
- **Optional**: Professional features are opt-in only

### Logging and Privacy
```bash
# Check what's being logged
cliai log-status

# Enable debug mode (explicit consent required)
cliai debug-mode --enable

# Clear logs
cliai clear-logs
```

**What's Logged:**
- ✅ Error messages (no sensitive data)
- ✅ Performance metrics
- ✅ Configuration changes
- ❌ User commands or prompts
- ❌ AI responses
- ❌ Personal information

## 🚨 Command Safety

### Multi-Layer Validation

#### 1. Syntax Checking
```bash
# Invalid syntax is caught
cliai "delete files with rm -rf /"
# ❌ Blocked: Dangerous pattern detected
```

#### 2. Risk Assessment
Commands are categorized by risk level:

- **🟢 Safe**: `ls`, `pwd`, `date`, `whoami`
- **🟡 Medium**: `cp`, `mv`, `mkdir`, `chmod`
- **🔴 High**: `rm -rf`, `sudo`, `dd`, `mkfs`

#### 3. Placeholder Detection
```bash
# AI hallucinations are caught
cliai "backup database"
# Output might be: mysqldump -u [username] -p [database] > backup.sql
# ❌ Blocked: Contains placeholders [username], [database]
```

#### 4. Context Validation
- Commands are checked against current directory
- File existence is verified where relevant
- Permissions are considered

### Safety Levels

#### 🔴 High Safety (Recommended for Beginners)
```bash
cliai safety-level high
```

**Features:**
- Blocks dangerous commands entirely
- Requires confirmation for system changes
- Maximum protection against accidents
- Detailed explanations for all actions

**Example:**
```bash
cliai "delete all log files"
# ❌ Blocked: High-risk operation not allowed in high safety mode
# 💡 Suggestion: Use 'cliai safety-level medium' for more flexibility
```

#### 🟡 Medium Safety (Default)
```bash
cliai safety-level medium
```

**Features:**
- Balanced protection and functionality
- Confirms risky operations
- Allows most commands with warnings
- Good for experienced users

**Example:**
```bash
cliai "delete old log files"
# ⚠️  Warning: This command will delete files permanently
# Command: find /var/log -name "*.log" -mtime +30 -delete
# Continue? [y/N]
```

#### 🟢 Low Safety (Experts Only)
```bash
cliai safety-level low
```

**Features:**
- Minimal safety checks
- Allows most commands
- Fewer warnings and confirmations
- For system administrators and experts

**Example:**
```bash
cliai "delete old log files"
# Command: find /var/log -name "*.log" -mtime +30 -delete
# 💡 Deletes log files older than 30 days
```

## ⚙️ Execution Modes

### 1. Manual Mode (Safest)
```bash
# Default behavior - commands are displayed, not executed
cliai "list files"
# Output: ls -la
# 💡 You copy and paste to execute
```

### 2. Auto-Execute Mode
```bash
cliai auto-execute on
cliai "list files"
# Command executes automatically if deemed safe
```

**Safety with Auto-Execute:**
- Only safe commands execute automatically
- Risky commands still require confirmation
- Respects safety level settings
- Can be disabled instantly

### 3. Dry-Run Mode (Testing)
```bash
cliai dry-run on
cliai "delete old files"
# DRY RUN: find . -name "*.tmp" -mtime +7 -delete
# Shows what would happen without executing
```

## 🔍 Command Validation Examples

### ✅ Safe Commands (Auto-Allowed)
```bash
cliai "show current directory"
# ✅ pwd

cliai "list running processes"
# ✅ ps aux

cliai "check disk space"
# ✅ df -h
```

### ⚠️ Medium Risk (Confirmation Required)
```bash
cliai "copy file to backup"
# ⚠️  cp important.txt important.txt.bak
# This will copy a file. Continue? [y/N]

cliai "install package"
# ⚠️  sudo apt install vim
# This requires administrator privileges. Continue? [y/N]
```

### ❌ High Risk (Blocked or Heavily Restricted)
```bash
cliai "format disk"
# ❌ Blocked: Extremely dangerous operation

cliai "delete everything"
# ❌ Blocked: Mass deletion detected

cliai "change root password"
# ❌ Blocked: Critical system modification
```

## 🚨 Security Best Practices

### 1. Start with High Safety
```bash
# New users should begin here
cliai safety-level high
```

### 2. Understand Commands Before Executing
```bash
# Always read the explanation
cliai "complex system command"
# Read the 💡 explanation before proceeding
```

### 3. Use Dry-Run for Unfamiliar Commands
```bash
cliai dry-run on
cliai "unfamiliar command"
# Review what would happen
cliai dry-run off
# Execute only if you understand it
```

### 4. Regular Safety Audits
```bash
# Check your current settings
cliai config

# Review recent activity
cliai log-status
```

### 5. Keep Software Updated
```bash
# Update CLIAI regularly
# Update Ollama models
ollama pull mistral
```

## 🔧 Emergency Procedures

### Stop Auto-Execution Immediately
```bash
cliai auto-execute off
# Disables automatic command execution
```

### Reset to Safe Defaults
```bash
cliai safety-level high
cliai auto-execute off
cliai dry-run off
```

### Clear Potentially Sensitive Logs
```bash
cliai clear-logs
# Removes all log files
```

### Disable Debug Mode
```bash
cliai debug-mode --disable
# Stops detailed logging
```

## 🚨 Incident Response

### If a Dangerous Command Was Executed

1. **Stop immediately**: Press Ctrl+C if still running
2. **Assess damage**: Check what was affected
3. **Report issue**: Create GitHub issue with details
4. **Increase safety**: `cliai safety-level high`
5. **Review settings**: `cliai config`

### If Unexpected Behavior Occurs

1. **Enable debug mode**: `cliai debug-mode --enable`
2. **Reproduce issue**: Try the same command again
3. **Check logs**: `cliai log-status`
4. **Report with logs**: Include debug information in issue

## 📊 Safety Monitoring

### Built-in Safety Tests
```bash
# Test safety features
cliai test --categories "safety,validation"

# Quick safety check
cliai test --quick
```

### Performance Impact of Safety
- **High Safety**: ~10-20ms additional validation time
- **Medium Safety**: ~5-10ms additional validation time
- **Low Safety**: ~1-2ms additional validation time

### Safety Metrics
CLIAI tracks (locally):
- Commands blocked by safety features
- User confirmations requested
- Safety level changes
- Validation failures

## 🔮 Future Safety Enhancements

### Planned Features
- **Sandboxing**: Isolated command execution
- **Rollback**: Undo dangerous operations
- **Learning**: Adapt to user patterns safely
- **Audit Trail**: Detailed operation history

### Community Safety
- **Crowdsourced Patterns**: Community-identified dangerous patterns
- **Safety Database**: Shared knowledge of risky commands
- **Best Practices**: Community-driven safety guidelines

## 📞 Security Contact

For security-related issues:
- **Email**: security@cliai.com
- **GitHub**: [Security Issues](https://github.com/cliai-team/cliai/security)
- **Response Time**: 24-48 hours for critical issues

---

**Remember**: Safety is a shared responsibility. CLIAI provides the tools, but you make the final decisions. When in doubt, use dry-run mode and ask for help in our community discussions.