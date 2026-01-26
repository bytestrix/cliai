#!/bin/bash

# CLIAI Build Script
# Builds CLIAI for different targets and configurations

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
BINARY_NAME="cliai"
BUILD_DIR="target"
DIST_DIR="dist"

# Default values
BUILD_TYPE="release"
TARGET=""
FEATURES=""
VERBOSE=false

# Print usage
usage() {
    echo "CLIAI Build Script"
    echo ""
    echo "Usage: $0 [options]"
    echo ""
    echo "Options:"
    echo "  -t, --target TARGET     Build for specific target (e.g., x86_64-unknown-linux-gnu)"
    echo "  -d, --debug            Build in debug mode (default: release)"
    echo "  -f, --features FEATURES Comma-separated list of features to enable"
    echo "  -v, --verbose          Verbose output"
    echo "  -c, --clean            Clean before building"
    echo "  -a, --all-targets      Build for all supported targets"
    echo "  -h, --help             Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0                                    # Build for current platform (release)"
    echo "  $0 --debug                           # Build in debug mode"
    echo "  $0 --target x86_64-unknown-linux-gnu # Build for specific target"
    echo "  $0 --all-targets                     # Build for all targets"
    echo "  $0 --features local-ai,cloud-ai      # Build with specific features"
    echo ""
}

# Parse command line arguments
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            -t|--target)
                TARGET="$2"
                shift 2
                ;;
            -d|--debug)
                BUILD_TYPE="debug"
                shift
                ;;
            -f|--features)
                FEATURES="$2"
                shift 2
                ;;
            -v|--verbose)
                VERBOSE=true
                shift
                ;;
            -c|--clean)
                CLEAN=true
                shift
                ;;
            -a|--all-targets)
                ALL_TARGETS=true
                shift
                ;;
            -h|--help)
                usage
                exit 0
                ;;
            *)
                echo -e "${RED}Unknown option: $1${NC}"
                usage
                exit 1
                ;;
        esac
    done
}

# Clean build directory
clean_build() {
    if [[ "$CLEAN" == "true" ]]; then
        echo -e "${BLUE}Cleaning build directory...${NC}"
        cargo clean
        rm -rf "$DIST_DIR"
    fi
}

# Check if target is installed
check_target() {
    local target=$1
    if ! rustup target list --installed | grep -q "$target"; then
        echo -e "${YELLOW}Installing target: $target${NC}"
        rustup target add "$target"
    fi
}

# Build for single target
build_target() {
    local target=$1
    local build_args=()
    
    echo -e "${BLUE}Building for target: $target${NC}"
    
    # Check if target is installed
    if [[ -n "$target" ]]; then
        check_target "$target"
        build_args+=("--target" "$target")
    fi
    
    # Add build type
    if [[ "$BUILD_TYPE" == "release" ]]; then
        build_args+=("--release")
    fi
    
    # Add features
    if [[ -n "$FEATURES" ]]; then
        build_args+=("--features" "$FEATURES")
    fi
    
    # Add verbose flag
    if [[ "$VERBOSE" == "true" ]]; then
        build_args+=("--verbose")
    fi
    
    # Build
    echo -e "${BLUE}Running: cargo build ${build_args[*]}${NC}"
    cargo build "${build_args[@]}"
    
    # Determine binary path
    local binary_path
    if [[ -n "$target" ]]; then
        binary_path="$BUILD_DIR/$target/$BUILD_TYPE/$BINARY_NAME"
    else
        binary_path="$BUILD_DIR/$BUILD_TYPE/$BINARY_NAME"
    fi
    
    # Add .exe extension for Windows
    if [[ "$target" == *"windows"* ]]; then
        binary_path="${binary_path}.exe"
    fi
    
    if [[ -f "$binary_path" ]]; then
        echo -e "${GREEN}✓ Build successful: $binary_path${NC}"
        
        # Copy to dist directory
        mkdir -p "$DIST_DIR"
        local dist_name="$BINARY_NAME"
        if [[ -n "$target" ]]; then
            dist_name="${BINARY_NAME}-${target}"
        fi
        if [[ "$target" == *"windows"* ]]; then
            dist_name="${dist_name}.exe"
        fi
        
        cp "$binary_path" "$DIST_DIR/$dist_name"
        echo -e "${GREEN}✓ Copied to: $DIST_DIR/$dist_name${NC}"
    else
        echo -e "${RED}✗ Build failed: $binary_path not found${NC}"
        exit 1
    fi
}

# Build for all supported targets
build_all_targets() {
    local targets=(
        "x86_64-unknown-linux-gnu"
        "aarch64-unknown-linux-gnu"
        "x86_64-apple-darwin"
        "aarch64-apple-darwin"
        "x86_64-pc-windows-msvc"
    )
    
    echo -e "${BLUE}Building for all supported targets...${NC}"
    
    for target in "${targets[@]}"; do
        echo ""
        build_target "$target"
    done
}

# Create release archives
create_archives() {
    if [[ ! -d "$DIST_DIR" ]]; then
        echo -e "${YELLOW}No dist directory found, skipping archive creation${NC}"
        return
    fi
    
    echo -e "${BLUE}Creating release archives...${NC}"
    
    cd "$DIST_DIR"
    
    for binary in *; do
        if [[ -f "$binary" ]]; then
            local archive_name
            if [[ "$binary" == *".exe" ]]; then
                archive_name="${binary%.exe}.zip"
                zip "$archive_name" "$binary"
            else
                archive_name="${binary}.tar.gz"
                tar -czf "$archive_name" "$binary"
            fi
            echo -e "${GREEN}✓ Created: $archive_name${NC}"
        fi
    done
    
    cd ..
}

# Show build summary
show_summary() {
    echo ""
    echo -e "${GREEN}🎉 Build Summary${NC}"
    echo -e "${BLUE}Build type: $BUILD_TYPE${NC}"
    
    if [[ -n "$FEATURES" ]]; then
        echo -e "${BLUE}Features: $FEATURES${NC}"
    fi
    
    if [[ -d "$DIST_DIR" ]]; then
        echo -e "${BLUE}Artifacts in $DIST_DIR:${NC}"
        ls -la "$DIST_DIR"
    fi
    
    echo ""
    echo -e "${BLUE}Next steps:${NC}"
    echo -e "• Test the binary: ${GREEN}./$DIST_DIR/$BINARY_NAME --version${NC}"
    echo -e "• Run tests: ${GREEN}cargo test${NC}"
    echo -e "• Install locally: ${GREEN}cargo install --path .${NC}"
}

# Main function
main() {
    echo -e "${GREEN}🔨 CLIAI Build Script${NC}"
    echo ""
    
    parse_args "$@"
    clean_build
    
    if [[ "$ALL_TARGETS" == "true" ]]; then
        build_all_targets
        create_archives
    elif [[ -n "$TARGET" ]]; then
        build_target "$TARGET"
    else
        build_target ""
    fi
    
    show_summary
}

# Run main function
main "$@"