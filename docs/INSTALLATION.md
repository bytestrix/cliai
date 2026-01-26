# Installation Guide

This guide covers various ways to install CLIAI on different platforms.

## 🚀 Quick Install

### Prerequisites

- **Rust 1.70+** - [Install Rust](https://rustup.rs/)
- **Ollama** (recommended) - [Install Ollama](https://ollama.ai/)

### One-Line Install (Recommended)

```bash
curl -fsSL https://raw.githubusercontent.com/cliai-team/cliai/main/install.sh | sh
```

## 📦 Installation Methods

### 1. From Source (Recommended for Developers)

```bash
# Clone the repository
git clone https://github.com/cliai-team/cliai.git
cd cliai

# Build and install
cargo build --release
sudo cp target/release/cliai /usr/local/bin/
```

### 2. Package Managers

#### Arch Linux (AUR)
```bash
# Using yay
yay -S cliai
```

## 🔧 Post-Installation Setup

### 1. Install Ollama (For Local AI)

CLIAI works best with local AI models via Ollama. This ensures your data never leaves your machine.

#### Linux
```bash
curl -fsSL https://ollama.ai/install.sh | sh
```

### 2. Pull an AI Model

```bash
# Recommended models
ollama pull mistral          # Fast, good quality
ollama pull codellama       # Optimized for code
```

### 3. Login for Professional Features

To access advanced online models and sync your subscription status:

```bash
cliai login
```

This will open your browser to the CLIAI platform where you can sign in. Once logged in, your professional features will be activated automatically in the CLI.

## 🛡️ Privacy & Safety

CLIAI is designed to be local-first. We do not support user-provided OpenAI or Azure API keys to ensure:
1. **Privacy**: Your data is managed through our secure platform connection.
2. **Sustainability**: Support for the development of CLIAI.
3. **Consistency**: A seamless experience across all your devices.

## 🧪 Verification

After installation, verify everything works:

```bash
# Check version
cliai --version

# Check local AI status
cliai provider-status

# Test basic functionality
cliai "how do I list files?"
```

## 🔄 Updating

```bash
cliai-update           # If using one-line install
# OR
cd cliai && git pull && cargo build --release
```

Happy CLI-ing! 🚀