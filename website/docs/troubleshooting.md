---
sidebar_position: 6
---

# Troubleshooting

Common issues and solutions for CLIAI users.

## 🚨 Common Issues

### Installation Problems

#### "Command not found: cliai"
**Problem**: CLIAI is not in your PATH

**Solutions:**
```bash
# Check if cliai exists
which cliai

# If installed via script, add to PATH
echo 'export PATH="$PATH:$HOME/.local/bin"' >> ~/.bashrc
source ~/.bashrc

# If installed manually, copy to system location
sudo cp cliai /usr/local/bin/
```

#### "Permission denied" on Linux/macOS
**Problem**: Binary doesn't have execute permissions

**Solution:**
```bash
chmod +x cliai
# Or if installed system-wide
sudo chmod +x /usr/local/bin/cliai
```

#### "Cannot be opened" on macOS
**Problem**: macOS Gatekeeper blocking unsigned binary

**Solution:**
```bash
# Remove quarantine attribute
xattr -d com.apple.quarantine cliai

# Or allow in System Preferences > Security & Privacy
```

### Ollama Connection Issues

#### "Failed to connect to Ollama"
**Problem**: Ollama is not running or unreachable

**Solutions:**
```bash
# Check if Ollama is running
ollama list

# Start Ollama service
ollama serve

# Check Ollama status
curl http://localhost:11434/api/tags

# Verify CLIAI configuration
cliai config
```

#### "No models found"
**Problem**: No Ollama models are installed

**Solutions:**
```bash
# Install a recommended model
ollama pull mistral

# List available models
ollama list

# Check CLIAI can see models
cliai list-models
```

#### "Model 'xyz' not found"
**Problem**: Trying to use a model that isn't installed

**Solutions:**
```bash
# Install the specific model
ollama pull xyz

# Or switch to an available model
cliai list-models
cliai select mistral
```

### Performance Issues

#### "Requests are very slow"
**Problem**: AI responses taking too long

**Solutions:**
```bash
# Check system resources
htop
# or
top

# Increase AI timeout
cliai ai-timeout 300000  # 5 minutes

# Try a smaller/faster model
ollama pull mistral:7b
cliai select mistral:7b

# Check provider status
cliai provider-status
```

#### "Context gathering timeout"
**Problem**: System context collection is slow

**Solutions:**
```bash
# Increase context timeout
cliai context-timeout 10000  # 10 seconds

# Check system performance
cliai performance-status

# Disable context gathering temporarily
# (This reduces AI accuracy but improves speed)
```

### Configuration Issues

#### "Configuration file corrupted"
**Problem**: Invalid configuration file

**Solutions:**
```bash
# Backup current config
cp ~/.config/cliai/config.json ~/.config/cliai/config.json.bak

# Reset to defaults
rm ~/.config/cliai/config.json
cliai config  # Will recreate with defaults

# Or manually fix the JSON
nano ~/.config/cliai/config.json
```

#### "Settings not persisting"
**Problem**: Configuration changes don't save

**Solutions:**
```bash
# Check file permissions
ls -la ~/.config/cliai/

# Fix permissions
chmod 644 ~/.config/cliai/config.json
chmod 755 ~/.config/cliai/

# Check disk space
df -h
```

### Safety and Validation Issues

#### "All commands are blocked"
**Problem**: Safety level too high for your needs

**Solutions:**
```bash
# Check current safety level
cliai config

# Reduce safety level
cliai safety-level medium
# or
cliai safety-level low  # For experts only
```

#### "Commands execute without confirmation"
**Problem**: Auto-execute enabled unexpectedly

**Solutions:**
```bash
# Disable auto-execute
cliai auto-execute off

# Check current settings
cliai config

# Enable dry-run for safety
cliai dry-run on
```

#### "Placeholder errors in commands"
**Problem**: AI generating commands with [placeholders]

**Solutions:**
```bash
# Be more specific in your request
# Instead of: "backup database"
# Try: "backup mysql database named 'myapp' to /backup/"

# Check if model needs updating
ollama pull mistral

# Try a different model
cliai select llama2
```

## 🔧 Diagnostic Commands

### System Information
```bash
# Check CLIAI version and config
cliai config

# Check provider status
cliai provider-status

# Check performance metrics
cliai performance-status

# Run diagnostic tests
cliai test --quick
```

### Debug Mode
```bash
# Enable detailed logging
cliai debug-mode --enable

# Check log location and status
cliai log-status

# View recent logs
tail -f ~/.config/cliai/error.log

# Disable debug mode
cliai debug-mode --disable
```

### Network Diagnostics
```bash
# Test Ollama connection
curl -X POST http://localhost:11434/api/generate \
  -H "Content-Type: application/json" \
  -d '{"model": "mistral", "prompt": "test", "stream": false}'

# Check if port is open
netstat -an | grep 11434
# or
ss -an | grep 11434
```

## 🐛 Reporting Issues

### Before Reporting
1. **Update CLIAI**: Make sure you're using the latest version
2. **Update Ollama**: `ollama pull mistral`
3. **Check existing issues**: [GitHub Issues](https://github.com/cliai-team/cliai/issues)
4. **Try basic troubleshooting**: Follow steps above

### Information to Include
```bash
# System information
uname -a
cliai config
cliai provider-status

# If debug mode is enabled
cliai log-status
# Include relevant log excerpts (remove sensitive data)
```

### Issue Template
```markdown
**Environment:**
- OS: [Linux/macOS/Windows]
- CLIAI Version: [run `cliai --version`]
- Ollama Version: [run `ollama --version`]

**Problem:**
[Describe what's happening]

**Expected Behavior:**
[What should happen instead]

**Steps to Reproduce:**
1. Run command: `cliai "example"`
2. See error: [error message]

**Configuration:**
[Output of `cliai config`]

**Logs:**
[Relevant log entries if available]
```

## 🔄 Recovery Procedures

### Complete Reset
```bash
# Backup current configuration
cp -r ~/.config/cliai ~/.config/cliai.backup

# Remove all CLIAI data
rm -rf ~/.config/cliai
rm -rf ~/.cache/cliai

# Reinstall CLIAI
# [Use your preferred installation method]

# Restore custom settings if needed
# [Manually reconfigure based on backup]
```

### Ollama Reset
```bash
# Stop Ollama
pkill ollama

# Remove models (if needed)
rm -rf ~/.ollama/models

# Restart Ollama
ollama serve &

# Reinstall models
ollama pull mistral
```

### Performance Reset
```bash
# Clear performance data
rm -rf ~/.cache/cliai/performance

# Reset timeouts to defaults
cliai context-timeout 2000
cliai ai-timeout 120000

# Test performance
cliai test --quick
```

## 📊 Performance Optimization

### System Requirements
- **Minimum RAM**: 4GB (8GB recommended)
- **Disk Space**: 2GB for models
- **CPU**: Any modern processor (faster = better)

### Optimization Tips
```bash
# Use smaller models for better performance
ollama pull mistral:7b  # Instead of larger variants

# Adjust timeouts based on your system
cliai context-timeout 1000   # Faster context gathering
cliai ai-timeout 60000       # Shorter AI timeout

# Monitor system resources
htop  # Check CPU and memory usage
```

### Model Recommendations by System
```bash
# High-end systems (16GB+ RAM)
ollama pull mistral
ollama pull llama2:13b

# Mid-range systems (8-16GB RAM)
ollama pull mistral:7b
ollama pull llama2:7b

# Low-end systems (4-8GB RAM)
ollama pull mistral:7b
# Consider using quantized models
```

## 🆘 Emergency Contacts

### Critical Issues
- **Security Issues**: security@cliai.com
- **Data Loss**: Create urgent GitHub issue
- **System Damage**: Seek professional help

### Community Support
- **GitHub Discussions**: [Ask questions](https://github.com/cliai-team/cliai/discussions)
- **GitHub Issues**: [Report bugs](https://github.com/cliai-team/cliai/issues)
- **Documentation**: [Read guides](https://cliai-team.github.io/cliai/)

### Response Times
- **Critical Security**: 24-48 hours
- **Bug Reports**: 3-7 days
- **Feature Requests**: 1-4 weeks
- **Community Questions**: 1-3 days

---

**Still having issues?** Don't hesitate to reach out to our community. We're here to help! 🤝