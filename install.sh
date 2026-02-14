#!/bin/bash
set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo -e "${CYAN}struct installer${NC}"
echo ""

# Check if Linux
OS="$(uname -s)"
if [ "$OS" != "Linux" ]; then
    echo -e "${RED}This installer only supports Linux${NC}"
    echo ""
    echo "For other platforms, build from source:"
    echo "  cargo build --release"
    echo "  sudo cp target/release/struct /usr/local/bin/"
    exit 1
fi

# Detect architecture
ARCH="$(uname -m)"
case "$ARCH" in
    x86_64|amd64)
        ARCH="x86_64"
        ;;
    aarch64|arm64)
        ARCH="aarch64"
        ;;
    armv7l)
        ARCH="armv7"
        ;;
    *)
        echo -e "${YELLOW}Architecture: $ARCH${NC}"
        echo -e "${YELLOW}This might work, but hasn't been tested${NC}"
        echo ""
        ;;
esac

echo -e "${CYAN}Detected: Linux ($ARCH)${NC}"
echo ""

# Check if running in the struct-cli repo
if [ ! -f "Cargo.toml" ] || ! grep -q "struct-cli" Cargo.toml; then
    echo -e "${RED}Not in struct-cli repo${NC}"
    echo ""
    echo "Clone the repo first:"
    echo "  git clone https://github.com/caffienerd/struct-cli"
    echo "  cd struct-cli"
    echo "  ./install.sh"
    exit 1
fi

# Check git branch
CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD 2>/dev/null)
if [ "$CURRENT_BRANCH" != "master" ] && [ "$CURRENT_BRANCH" != "main" ]; then
    echo -e "${YELLOW}You are not in the master/main branch (current: $CURRENT_BRANCH)${NC}"
    echo ""
    read -p "Are you sure you want to continue the installation? (y/n): " branch_confirm
    
    if [ "$branch_confirm" != "y" ] && [ "$branch_confirm" != "Y" ]; then
        echo -e "${YELLOW}Installation cancelled${NC}"
        exit 1
    fi
    echo ""
fi

echo -e "${GREEN}Building from source...${NC}"

# Detect Linux distribution and install dependencies
if [ -f /etc/os-release ]; then
    . /etc/os-release
    DISTRO=$ID
else
    DISTRO="unknown"
fi

echo -e "${CYAN}Detected distribution: $DISTRO${NC}"

# Check and install required build dependencies
case "$DISTRO" in
    ubuntu|debian|linuxmint)
        echo ""
        echo -e "${YELLOW}Installing build dependencies for $DISTRO...${NC}"
        sudo apt-get update
        sudo apt-get install -y build-essential libssl-dev pkg-config
        ;;
    fedora)
        echo ""
        echo -e "${YELLOW}Installing build dependencies for $DISTRO...${NC}"
        sudo dnf install -y gcc openssl-devel pkg-config
        ;;
    opensuse*|suse)
        echo ""
        echo -e "${YELLOW}Installing build dependencies for $DISTRO...${NC}"
        sudo zypper install -y gcc openssl-devel pkg-config
        ;;
    arch|manjaro)
        echo ""
        echo -e "${YELLOW}Installing build dependencies for $DISTRO...${NC}"
        sudo pacman -Syu --noconfirm gcc openssl pkg-config
        ;;
    *)
        echo -e "${YELLOW}Unknown distribution: $DISTRO${NC}"
        echo -e "${YELLOW}Please ensure you have build-essential/gcc and openssl development packages installed${NC}"
        ;;
esac

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}Rust is not installed.${NC}"
    echo ""
    read -p "Would you like to install Rust now? (y/n): " install_rust
    
    if [ "$install_rust" = "y" ] || [ "$install_rust" = "Y" ]; then
        echo ""
        echo -e "${CYAN}Installing Rust...${NC}"
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        
        # Source cargo env
        source "$HOME/.cargo/env"
        
        echo ""
        echo -e "${GREEN}Rust installed successfully!${NC}"
        echo ""
    else
        echo ""
        echo -e "${YELLOW}Install Rust manually from: https://rustup.rs${NC}"
        echo "Or use your distro's package manager:"
        echo "  Arch: sudo pacman -S rust"
        echo "  Fedora: sudo dnf install rust cargo"
        echo "  Ubuntu/Debian: sudo apt install cargo"
        exit 1
    fi
fi

# Build release
echo ""
cargo build --release

if [ $? -ne 0 ]; then
    echo ""
    echo -e "${RED}Build failed${NC}"
    exit 1
fi

BINARY="target/release/struct"

# Check if binary exists
if [ ! -f "$BINARY" ]; then
    echo -e "${RED}Binary not found at $BINARY${NC}"
    exit 1
fi

echo ""
echo -e "${GREEN}Build successful!${NC}"
echo ""
echo -e "${CYAN}Choose installation method:${NC}"
echo "  1) Install to /usr/local/bin (recommended, requires sudo)"
echo "  2) Install to ~/.local/bin (no sudo needed)"
echo "  3) Just show binary location (I'll install manually)"
echo ""
read -p "Enter choice (1-3): " choice

case $choice in
    1)
        echo ""
        echo -e "${GREEN}Installing to /usr/local/bin...${NC}"
        sudo cp "$BINARY" /usr/local/bin/struct
        sudo chmod +x /usr/local/bin/struct
        echo -e "${GREEN}Installed successfully!${NC}"
        echo ""
        echo -e "${CYAN}Try: struct 3 -z${NC}"
        ;;
    2)
        mkdir -p ~/.local/bin
        cp "$BINARY" ~/.local/bin/struct
        chmod +x ~/.local/bin/struct
        echo ""
        echo -e "${GREEN}Installed to ~/.local/bin/struct${NC}"
        echo ""
        
        # Check if ~/.local/bin is in PATH
        if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
            echo -e "${YELLOW}~/.local/bin is not in your PATH${NC}"
            echo ""
            echo "Add this to your shell config (~/.bashrc, ~/.zshrc, etc):"
            echo ""
            echo -e "${CYAN}  export PATH=\"\$HOME/.local/bin:\$PATH\"${NC}"
            echo ""
            echo "Then reload: source ~/.bashrc (or restart terminal)"
        else
            echo -e "${CYAN}Try: struct 3${NC}"
        fi
        ;;
    3)
        echo ""
        echo -e "${GREEN}Binary location:${NC}"
        echo -e "${CYAN}  $(pwd)/$BINARY${NC}"
        echo ""
        echo "To install manually:"
        echo -e "${CYAN}  sudo cp $(pwd)/$BINARY /usr/local/bin/struct${NC}"
        ;;
    *)
        echo ""
        echo -e "${RED}Invalid choice${NC}"
        exit 1
        ;;
esac

echo ""
echo -e "${GREEN}Done!${NC}"