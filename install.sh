#!/bin/bash
# CLIAI Installation Script
# Supports Linux, macOS, and Windows (via Git Bash/WSL)

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
REPO="cliai/cliai"
BINARY_NAME="cliai"
INSTALL_DIR="$HOME/.local/bin"

# Detect OS and architecture
detect_platform() {
    local os=$(uname -s | tr '[:upper:]' '[:lower:]')
    local arch=$(uname -m)
    
    case $os in
        linux*)
            OS="linux"
            ;;
        darwin*)
            OS="macos"
            ;;
        mingw*|msys*|cygwin*)
            OS="windows"
            ;;
        *)
            echo -e "${RED}Unsupported operating system: $os${NC}"
            exit 1
            ;;
    esac
    
    case $arch in
        x86_64|amd64)
            ARCH="x86_64"
            ;;
        aarch64|arm64)
            ARCH="aarch64"
            ;;
        *)
            echo -e "${RED}Unsupported architecture: $arch${NC}"
            exit 1
            ;;
    esac
    
    if [ "$OS" = "windows" ]; then
        BINARY_NAME="cliai.exe"
        ARCHIVE_EXT="zip"
    else
        ARCHIVE_EXT="tar.gz"
    fi
    
    PLATFORM="${OS}-${ARCH}"
    echo -e "${BLUE}Detected platform: $PLATFORM${NC}"
}

# Get latest release version
get_latest_version() {
    echo -e "${BLUE}Fetching latest release information...${NC}"
    
    if command -v curl >/dev/null 2>&1; then
        VERSION=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
    elif command -v wget >/dev/null 2>&1; then
        VERSION=$(wget -qO- "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
    else
        echo -e "${RED}Error: curl or wget is required${NC}"
        exit 1
    fi
    
    if [ -z "$VERSION" ]; then
        echo -e "${RED}Error: Could not fetch latest version${NC}"
        exit 1
    fi
    
    echo -e "${GREEN}Latest version: $VERSION${NC}"
}

# Download and install
install_cliai() {
    local archive_name="${BINARY_NAME}-${PLATFORM}.${ARCHIVE_EXT}"
    local download_url="https://github.com/$REPO/releases/download/$VERSION/$archive_name"
    local temp_dir=$(mktemp -d)
    
    echo -e "${BLUE}Downloading $archive_name...${NC}"
    
    if command -v curl >/dev/null 2>&1; then
        curl -L "$download_url" -o "$temp_dir/$archive_name"
    elif command -v wget >/dev/null 2>&1; then
        wget "$download_url" -O "$temp_dir/$archive_name"
    else
        echo -e "${RED}Error: curl or wget is required${NC}"
        exit 1
    fi
    
    echo -e "${BLUE}Extracting archive...${NC}"
    cd "$temp_dir"
    
    if [ "$ARCHIVE_EXT" = "zip" ]; then
        if command -v unzip >/dev/null 2>&1; then
            unzip -q "$archive_name"
        else
            echo -e "${RED}Error: unzip is required for Windows installation${NC}"
            exit 1
        fi
    else
        tar -xzf "$archive_name"
    fi
    
    echo -e "${BLUE}Installing to $INSTALL_DIR...${NC}"
    mkdir -p "$INSTALL_DIR"
    
    if [ -f "$BINARY_NAME" ]; then
        cp "$BINARY_NAME" "$INSTALL_DIR/"
        chmod +x "$INSTALL_DIR/$BINARY_NAME"
    else
        echo -e "${RED}Error: Binary not found in archive${NC}"
        exit 1
    fi
    
    # Cleanup
    rm -rf "$temp_dir"
    
    echo -e "${GREEN}✅ CLIAI installed successfully!${NC}"
}

# Check if binary is in PATH
check_path() {
    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        echo -e "${YELLOW}⚠️  $INSTALL_DIR is not in your PATH${NC}"
        echo -e "${BLUE}Add this line to your shell profile (~/.bashrc, ~/.zshrc, etc.):${NC}"
        echo -e "${GREEN}export PATH=\"\$PATH:$INSTALL_DIR\"${NC}"
        echo ""
        echo -e "${BLUE}Or run this command now:${NC}"
        echo -e "${GREEN}echo 'export PATH=\"\$PATH:$INSTALL_DIR\"' >> ~/.bashrc && source ~/.bashrc${NC}"
    else
        echo -e "${GREEN}✅ $INSTALL_DIR is already in your PATH${NC}"
    fi
}

# Install Ollama if not present
suggest_ollama() {
    if ! command -v ollama >/dev/null 2>&1; then
        echo -e "${YELLOW}⚠️  Ollama not found. CLIAI works best with Ollama for local AI.${NC}"
        echo -e "${BLUE}Install Ollama:${NC}"
        
        case $OS in
            linux|macos)
                echo -e "${GREEN}curl -fsSL https://ollama.ai/install.sh | sh${NC}"
                ;;
            windows)
                echo -e "${GREEN}Download from: https://ollama.ai/download${NC}"
                ;;
        esac
        
        echo -e "${BLUE}After installing Ollama, run:${NC}"
        echo -e "${GREEN}ollama pull mistral${NC}"
    else
        echo -e "${GREEN}✅ Ollama is already installed${NC}"
    fi
}

# Main installation flow
main() {
    echo -e "${BLUE}🤖 CLIAI Installation Script${NC}"
    echo ""
    
    detect_platform
    get_latest_version
    install_cliai
    check_path
    suggest_ollama
    
    echo ""
    echo -e "${GREEN}🎉 Installation complete!${NC}"
    echo -e "${BLUE}Try running: ${GREEN}cliai \"hello world\"${NC}"
    echo -e "${BLUE}For help: ${GREEN}cliai --help${NC}"
    echo ""
    echo -e "${BLUE}Documentation: https://github.com/$REPO${NC}"
}

# Run main function
main "$@"