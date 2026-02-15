# CLIAI 🤖

[![CI](https://github.com/cliai-team/cliai/actions/workflows/ci.yml/badge.svg)](https://github.com/cliai-team/cliai/actions/workflows/ci.yml)
[![Documentation](https://github.com/cliai-team/cliai/actions/workflows/docs.yml/badge.svg)](https://cliai-team.github.io/cliai/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Crates.io](https://img.shields.io/crates/v/cliai.svg)](https://crates.io/crates/cliai)

**Your intelligent CLI assistant powered by AI**

CLIAI is a completely free and open-source command-line AI assistant that helps you with terminal commands, system administration, and general questions. Use your own API keys (OpenAI, Anthropic, etc.) or run locally with Ollama for maximum privacy.

## ✨ Features

- **🔒 Privacy-First**: Local AI processing with Ollama - your data never leaves your machine
- **🔑 Bring Your Own Key**: Use your own OpenAI, Anthropic, or other LLM API keys
- **🆓 Completely Free**: No subscriptions, no hidden costs - 100% open source
- **🛡️ Safety-Focused**: Multi-level command validation and safety checks
- **⚡ Fast & Reliable**: Built-in performance monitoring and circuit breakers
- **🎯 Smart Command Generation**: Context-aware command suggestions with explanations
- **📝 Copy-Paste Safe**: Clean command output without formatting artifacts
- **🔄 Flexible Execution**: Auto-execute, confirmation prompts, or manual copy-paste
- **📊 Comprehensive Testing**: Built-in test suite with 50+ scenarios
- **🎨 Beautiful Interface**: Colored output with progress indicators

## 🚀 Quick Start

### Prerequisites

- **Ollama** (recommended for local AI) - [Install Ollama](https://ollama.ai/)

### Installation

#### 🔥 One-Line Install (Linux/macOS/Windows)
```bash
curl -fsSL https://raw.githubusercontent.com/cliai-team/cliai/main/install.sh | bash
```

#### 📦 Package Managers

**Arch Linux (AUR)**
```bash
yay -S cliai
# or
paru -S cliai
```

**Ubuntu/Debian**
```bash
# Download .deb from releases
wget https://github.com/cliai-team/cliai/releases/latest/download/cliai.deb
sudo dpkg -i cliai.deb
```

**macOS (Homebrew)**
```bash
brew tap cliai-team/tap
brew install cliai
```

**Windows (Chocolatey)**
```powershell
choco install cliai
```

**Windows (Winget)**
```powershell
winget install cliai-team.cliai
```

**Cargo (All Platforms)**
```bash
cargo install cliai
```

#### 📥 Manual Download

Download pre-built binaries from [GitHub Releases](https://github.com/cliai-team/cliai/releases):

- **Linux**: `cliai-linux-x86_64.tar.gz` or `cliai-linux-aarch64.tar.gz`
- **macOS**: `cliai-macos-x86_64.tar.gz` or `cliai-macos-aarch64.tar.gz`  
- **Windows**: `cliai-windows-x86_64.zip`

#### 🔨 From Source
```bash
git clone https://github.com/cliai-team/cliai.git
cd cliai
cargo build --release
# Linux/macOS
sudo cp target/release/cliai /usr/local/bin/
# Windows: Copy target/release/cliai.exe to a directory in your PATH
```

### Setup

1. **Choose your AI provider**:

   **Option A: Local AI (Ollama - Free & Private)**
   ```bash
   # Install Ollama
   curl -fsSL https://ollama.ai/install.sh | sh
   
   # Pull a model (recommended: mistral or llama2)
   ollama pull mistral
   ```

   **Option B: Cloud AI (Your API Keys)**
   ```bash
   # Set your OpenAI API key
   cliai set-key openai sk-your-openai-key-here
   
   # Or use Anthropic Claude
   cliai set-key anthropic your-anthropic-key-here
   
   # Or other supported providers
   cliai list-providers  # See all supported providers
   ```

2. **Configure CLIAI**:
```bash
# Set your preferred model
cliai select mistral              # For Ollama
# or
cliai select gpt-4               # For OpenAI
# or  
cliai select claude-3-sonnet     # For Anthropic

# Optional: Set a custom prefix
cliai set-prefix ai
```

3. **Start using CLIAI**:
```bash
cliai "how do I list all files including hidden ones?"
# Output: ls -la
# 💡 Lists all files including hidden ones with detailed information

cliai "find all Python files in this directory"
# Output: find . -name "*.py"
# 💡 Recursively searches for Python files in current directory and subdirectories
```

## 📖 Usage

### Basic Commands

```bash
# Ask questions
cliai "how do I check disk usage?"
cliai "what's my IP address?"
cliai "compress this folder"

# Configuration
cliai config                    # Show current settings
cliai list-models              # List available models
cliai list-providers           # List supported AI providers
cliai select <model>           # Switch models
cliai set-key <provider> <key> # Set API key for provider
cliai clear                    # Clear chat history

# API Key Management
cliai set-key openai sk-...    # Set OpenAI API key
cliai set-key anthropic ...    # Set Anthropic API key
cliai remove-key <provider>    # Remove API key
cliai test-key <provider>      # Test API key connection

# Safety & Execution
cliai auto-execute --enable    # Enable auto-execution for safe commands
cliai dry-run --enable         # Preview commands without executing
cliai safety-level high       # Set safety level (low/medium/high)

# Monitoring
cliai provider-status          # Check AI provider status
cliai performance-status       # View performance metrics
cliai test                     # Run comprehensive test suite
```

### Custom Prefix

Set a custom command prefix for easier access:

```bash
cliai set-prefix jarvis
# Now you can use: jarvis "list running processes"
```

### Safety Levels

- **Low**: Minimal safety checks, allows most commands
- **Medium**: Balanced safety with confirmation for risky commands (default)
- **High**: Maximum safety, blocks dangerous operations

## 🏗️ Architecture

CLIAI follows a modular architecture designed for reliability and extensibility:

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   CLI Interface │────│   Orchestrator   │────│  AI Providers   │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                              │                          │
                    ┌─────────┼─────────┐               │
                    │         │         │               │
            ┌───────▼───┐ ┌───▼────┐ ┌──▼─────┐        │
            │ Intent    │ │Context │ │Command │        │
            │Classifier │ │Gatherer│ │Validator│       │
            └───────────┘ └────────┘ └────────┘        │
                                                        │
                              ┌─────────────────────────┘
                              │
                    ┌─────────▼─────────┐
                    │                   │
            ┌───────▼────┐    ┌─────────▼──────┐
            │Local Ollama│    │Cloud Providers │
            │(Privacy)   │    │(Fallback)      │
            └────────────┘    └────────────────┘
```

### Core Components

- **Orchestrator**: Central coordinator managing AI providers and request routing
- **Intent Classifier**: Determines the type of request (command, question, etc.)
- **Context Gatherer**: Collects system information for better responses
- **Command Validator**: Multi-layer validation with security checks
- **Execution Engine**: Safe command execution with multiple modes
- **Performance Monitor**: Tracks metrics and system health
- **Circuit Breakers**: Automatic failover between providers

### AI Provider System

CLIAI supports multiple AI providers for maximum flexibility:

1. **Local Ollama** (Free & Private): Complete privacy, offline capable
2. **OpenAI** (Your API Key): GPT-3.5, GPT-4, and other OpenAI models
3. **Anthropic** (Your API Key): Claude models for advanced reasoning
4. **Other Providers**: Additional providers can be easily added

**Supported Providers:**
- Ollama (local)
- OpenAI (gpt-3.5-turbo, gpt-4, gpt-4-turbo)
- Anthropic (claude-3-sonnet, claude-3-haiku, claude-3-opus)
- More providers coming soon!

## 🛡️ Security & Privacy

### Privacy Protection
- **Local-First**: Primary processing happens on your machine
- **No Data Collection**: Commands and prompts are never logged in production
- **Minimal Logging**: Only errors and performance metrics are recorded
- **Debug Mode**: Explicit consent required for detailed logging

### Safety Features
- **Command Validation**: Multi-layer validation prevents dangerous commands
- **Placeholder Detection**: Catches AI hallucinations and incomplete commands
- **Syntax Checking**: Validates command syntax before execution
- **Risk Assessment**: Categorizes commands by potential impact
- **Confirmation Prompts**: User confirmation for sensitive operations

### Safety Levels
```bash
# High Safety (Recommended for beginners)
cliai safety-level high

# Medium Safety (Balanced - default)
cliai safety-level medium

# Low Safety (Experienced users)
cliai safety-level low
```

## 🧪 Testing

CLIAI includes a comprehensive test suite covering 50+ real-world scenarios:

```bash
# Run full test suite
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

## ⚙️ Configuration

CLIAI stores configuration in `~/.config/cliai/config.toml`:

```toml
model = "mistral"
provider = "ollama"  # or "openai", "anthropic"
auto_execute = false
dry_run = false
safety_level = "Medium"
context_timeout = 5000
ollama_url = "http://localhost:11434"
prefix = "cliai"

# API Keys (stored securely)
[api_keys]
# Keys are encrypted and stored separately for security
```

### API Key Management

CLIAI securely stores your API keys using your system's keyring:

```bash
# Set API keys
cliai set-key openai sk-your-key-here
cliai set-key anthropic your-anthropic-key

# Test connections
cliai test-key openai
cliai test-key anthropic

# Remove keys
cliai remove-key openai
```

## 🔧 Development

### Building from Source

```bash
git clone https://github.com/cliai-team/cliai.git
cd cliai
cargo build --release
```

### Running Tests

```bash
# Unit tests
cargo test

# Integration tests
cargo test --test integration

# Live AI tests (requires Ollama)
cliai test --quick
```

### Project Structure

```
src/
├── main.rs              # CLI interface and main entry point
├── lib.rs               # Library exports
├── agents/              # AI orchestration and provider management
│   ├── mod.rs
│   └── profiles.rs
├── config.rs            # Configuration management
├── context.rs           # System context gathering
├── execution.rs         # Command execution engine
├── validation.rs        # Command validation and safety
├── providers.rs         # AI provider implementations
├── history.rs           # Chat history management
├── performance.rs       # Performance monitoring
├── error_handling.rs    # Enhanced error reporting
├── logging.rs           # Privacy-preserving logging
└── test_suite.rs        # Comprehensive testing framework
```

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Setup

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Make your changes
4. Run tests: `cargo test && cliai test --quick`
5. Commit changes: `git commit -m 'Add amazing feature'`
6. Push to branch: `git push origin feature/amazing-feature`
7. Open a Pull Request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details. This means you can use, modify, and distribute the software freely, including for commercial purposes.

## 🙏 Acknowledgments

- [Ollama](https://ollama.ai/) for making local AI accessible
- [Clap](https://github.com/clap-rs/clap) for excellent CLI parsing
- [Tokio](https://tokio.rs/) for async runtime
- The Rust community for amazing tools and libraries

## 📞 Support

- **Documentation**: [https://cliai-team.github.io/cliai/](https://cliai-team.github.io/cliai/)
- **Issues**: [GitHub Issues](https://github.com/cliai-team/cliai/issues)
- **Discussions**: [GitHub Discussions](https://github.com/cliai-team/cliai/discussions)
- **Wiki**: [GitHub Wiki](https://github.com/cliai-team/cliai/wiki)

---

**Made with ❤️ by the CLIAI Team**

## 🌟 Star History

[![Star History Chart](https://api.star-history.com/svg?repos=cliai-team/cliai&type=Date)](https://star-history.com/#cliai-team/cliai&Date)