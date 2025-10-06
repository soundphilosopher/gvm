#!/usr/bin/env bash

set -e

# Colors for pretty output 🎨
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Default options
INSTALL_LATEST=false

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --install-latest)
            INSTALL_LATEST=true
            shift
            ;;
        --help)
            echo "🧪 Install Golang Version Manager for Linux"
            echo ""
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --install-latest    Install the latest version of Golang"
            echo "  --help              Show this help message"
            echo ""
            exit 0
            ;;
        *)
            echo -e "${RED}❌ Unknown option: $1${NC}"
            echo "Use --help for available options"
            exit 1
            ;;
    esac
done

echo -e "${CYAN}🧪 Install Golang Version Manager for Linux${NC}"
echo "=================================="

echo -e "\n${BLUE}📦 Check for system dependencies ...${NC}"
if ! command -v cargo 2>&1 > /dev/null; then
    echo -e "${RED}❌ Error: No installation for Rust found${NC}"
    exit 1
fi

echo -e "\n${BLUE}📦 Installing version manager ...${NC}"
if ! command -v gvm 2>&1 > /dev/null; then
  echo "Install GVM ..."
  cargo install --path .
  echo -e "${GREEN}✅ GVM installed${NC}"
else
  echo -e "${GREEN}✅ GVM already installed${NC}"
fi

echo -e "\n${BLUE}💡 Initialize GVM and update remote release cache${NC}"
echo "--------------------------------------"
gvm init
gvm update
echo -e "${GREEN}✅ GVM initialized${NC}"

if [[ "$INSTALL_LATEST" == "true" ]]; then
    echo -e "\n${BLUE}📦 Installinng latest stable go version ...${NC}"
    echo "--------------------------------------"

    latest_version="$(gvm ls-remote --stable | tail -n1 | awk '{print $1}')"
    gvm install $latest_version --use

    echo -e "${GREEN}✅ Go ${latest_version} installed${NC}"
fi

echo -e "\n${YELLOW}💡 Reload your shell profile to use GVM${NC}"
