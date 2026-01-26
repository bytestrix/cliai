# CLIAI Distribution Guide

This document explains how CLIAI is packaged and distributed across different platforms.

## 🎯 Supported Platforms

### ✅ Fully Supported
- **Linux x86_64** (Ubuntu, Debian, Arch, etc.)
- **Linux aarch64** (ARM64)
- **macOS x86_64** (Intel Macs)
- **macOS aarch64** (Apple Silicon)

### 🔄 Limited Support
- **Windows x86_64** (Windows 10/11)

## 📦 Distribution Channels

### 1. GitHub Releases (Primary)
- Pre-built binaries for all platforms
- Automatic builds via GitHub Actions
- Direct download links
- Checksums provided

### 2. Package Managers

#### Arch Linux (AUR)
- Package: `cliai`
- Maintainer: CLIAI Team
- Install: `yay -S cliai` or `paru -S cliai`

#### Ubuntu/Debian
- `.deb` packages available
- Install: `sudo dpkg -i cliai.deb`
- Future: PPA repository

#### macOS (Homebrew)
- Tap: `cliai-team/tap`
- Install: `brew install cliai`

#### Windows (Chocolatey)
- Package: `cliai`
- Install: `choco install cliai`

#### Cargo (crates.io)
- Package: `cliai`
- Install: `cargo install cliai`

### 3. Installation Script
- One-line installer for Linux/macOS/Windows
- Detects platform automatically
- Downloads and installs latest version
- Sets up PATH if needed

## 🔧 Build Process

### Automated Builds
GitHub Actions automatically builds:
- Linux x86_64 and aarch64
- macOS x86_64 and aarch64 (Apple Silicon)
- Windows x86_64

### Manual Build
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/cliai-team/cliai.git
cd cliai
cargo build --release

# Binary location: target/release/cliai (or cliai.exe on Windows)
```

### Cross-compilation
```bash
# Install target
rustup target add aarch64-unknown-linux-gnu

# Build for ARM64 Linux
cargo build --release --target aarch64-unknown-linux-gnu
```

## 📋 Release Checklist

### Pre-release
- [ ] Update version in `Cargo.toml`
- [ ] Update `CHANGELOG.md`
- [ ] Test on all supported platforms
- [ ] Run comprehensive test suite
- [ ] Update documentation

### Release Process
1. Create and push version tag: `git tag v0.1.0 && git push origin v0.1.0`
2. GitHub Actions automatically:
   - Builds binaries for all platforms
   - Creates GitHub release
   - Publishes to crates.io
   - Builds .deb packages
3. Manual steps:
   - Update AUR package
   - Update Homebrew formula
   - Update Chocolatey package
   - Announce release

### Post-release
- [ ] Verify all download links work
- [ ] Test installation on different platforms
- [ ] Update documentation sites
- [ ] Social media announcement

## 🛠️ Package Maintenance

### AUR (Arch Linux)
- Location: `PKGBUILD`
- Update: Version, checksums, dependencies
- Test: `makepkg -si`

### Homebrew
- Location: `packaging/homebrew/cliai.rb`
- Update: Version, URL, SHA256
- Test: `brew install --build-from-source cliai`

### Chocolatey
- Location: `packaging/chocolatey/`
- Update: Version, URLs, checksums
- Test: `choco install cliai --source .`

### Debian
- Generated automatically via `cargo-deb`
- Metadata in `Cargo.toml` under `[package.metadata.deb]`

## 🔍 Platform-Specific Notes

### Linux
- Primary target platform
- Full feature support
- Systemd integration
- Package manager detection

### macOS
- Homebrew integration
- Apple Silicon support
- Limited package manager support

### Windows
- Basic functionality
- PowerShell integration
- Chocolatey packaging
- Limited shell detection

## 📊 Distribution Statistics

Track installation methods:
- GitHub Releases downloads
- Package manager installs
- Cargo installs
- Script installs

## 🚨 Troubleshooting

### Common Issues

**Linux: Permission denied**
```bash
chmod +x cliai
sudo cp cliai /usr/local/bin/
```

**macOS: "cliai" cannot be opened**
```bash
xattr -d com.apple.quarantine cliai
```

**Windows: Not recognized as command**
- Add installation directory to PATH
- Restart terminal/PowerShell

**All platforms: Ollama not found**
```bash
# Install Ollama first
curl -fsSL https://ollama.ai/install.sh | sh
ollama pull mistral
```

## 🔮 Future Plans

### Short-term
- [ ] Windows Package Manager (winget)
- [ ] Snap packages
- [ ] Flatpak packages
- [ ] Docker images

### Long-term
- [ ] Official Ubuntu PPA
- [ ] Fedora/RHEL packages
- [ ] FreeBSD ports
- [ ] NixOS packages

## 📞 Support

For distribution-related issues:
- GitHub Issues: [Report problems](https://github.com/cliai-team/cliai/issues)
- Discussions: [Ask questions](https://github.com/cliai-team/cliai/discussions)
- Email: contact@cliai.com