---
sidebar_position: 1
---

# Welcome to CLIAI

**Your intelligent CLI assistant powered by AI** 🤖

CLIAI is a command-line AI assistant that helps you with terminal commands, system administration, and general questions. It is built for local-first AI (via Ollama), ensuring maximum privacy and offline capability.

## ✨ Key Features

- **🔒 Privacy-First**: Local AI processing with Ollama - your data never leaves your machine
- **🛡️ Safety-Focused**: Multi-level command validation and safety checks
- **⚡ Fast & Reliable**: Built-in performance monitoring and circuit breakers
- **🎯 Smart Command Generation**: Context-aware command suggestions with explanations
- **📝 Copy-Paste Safe**: Clean command output without formatting artifacts
- **🔄 Flexible Execution**: Auto-execute, confirmation prompts, or manual copy-paste
- **📊 Comprehensive Testing**: Built-in test suite with 50+ scenarios
- **🎨 Beautiful Interface**: Colored output with progress indicators

## 🚀 Quick Start

Get started with CLIAI in just a few minutes:

```bash
# Install with one command
curl -fsSL https://raw.githubusercontent.com/cliai-team/cliai/main/install.sh | bash

# Or use your package manager
yay -S cliai          # Arch Linux
brew install cliai    # macOS
choco install cliai   # Windows
```

## 💡 Example Usage

```bash
# Ask for help with commands
cliai "how do I list all files including hidden ones?"
# Output: ls -la

# Get system information
cliai "show me disk usage"
# Output: df -h

# File operations
cliai "compress this folder into a zip file"
# Output: zip -r folder.zip folder/

# Git operations
cliai "create a new branch and switch to it"
# Output: git checkout -b new-branch
```

## 🏗️ Architecture

CLIAI follows a modular, safety-first architecture:

- **Local-First**: Primary processing with Ollama
- **Circuit Breakers**: Automatic failover and recovery
- **Multi-Layer Validation**: Command safety checks
- **Performance Monitoring**: Built-in metrics and optimization
- **Cross-Platform**: Linux, macOS, and Windows support

## 📚 What's Next?

- [**Installation Guide**](./installation) - Detailed installation instructions for all platforms
- [**Configuration**](./configuration) - Customize CLIAI to your needs
- [**Usage Guide**](./usage) - Learn all the features and commands
- [**Safety & Security**](./safety) - Understand CLIAI's safety features
- [**Troubleshooting**](./troubleshooting) - Common issues and solutions

## 🤝 Community

- **GitHub**: [cliai-team/cliai](https://github.com/cliai-team/cliai)
- **Issues**: [Report bugs](https://github.com/cliai-team/cliai/issues)
- **Discussions**: [Ask questions](https://github.com/cliai-team/cliai/discussions)
- **Releases**: [Latest updates](https://github.com/cliai-team/cliai/releases)

---

Ready to supercharge your command line experience? Let's get started! 🚀