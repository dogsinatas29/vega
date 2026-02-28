!/bin/bash

# Definition of colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}🌌 Project VEGA Environment Initializer${NC}"

# 1. Check for Rust Toolchain
echo -e "\n${YELLOW}[1/2] Checking Rust Toolchain...${NC}"

if ! command -v cargo &> /dev/null; then
    echo -e "${RED}❌ 'cargo' not found.${NC}"
    echo -e "VEGA requires the Rust toolchain to run."
    echo -e "Please install it by running the following command:\n"
    echo -e "    ${GREEN}curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh${NC}\n"
    echo -e "After installation, restart your terminal and run this script again."
    exit 1
fi

echo -e "${GREEN}✅ Rust is installed.$(cargo --version | awk '{print $2}')${NC}"

# 2. Check for Release Binary
echo -e "\n${YELLOW}[2/2] Checking VEGA Binary...${NC}"

TARGET_BIN="target/release/vega"

if [ ! -f "$TARGET_BIN" ]; then
    echo -e "${YELLOW}⚠️  Binary not found at $TARGET_BIN${NC}"
    echo -e "🚀 Building VEGA in release mode... (This may take a minute)"
    
    if cargo build --release; then
        echo -e "${GREEN}✅ Build successful!${NC}"
    else
        echo -e "${RED}❌ Build failed.${NC}"
        exit 1
    fi
else
    echo -e "${GREEN}✅ VEGA binary already exists.${NC}"
fi

# 3. Completion
echo -e "\n${GREEN}🎉 VEGA is ready to serve.${NC}"
echo -e "Run: ${YELLOW}./$TARGET_BIN <query>${NC}"
echo -e "Example: ${YELLOW}./$TARGET_BIN check disk space${NC}"
