#!/bin/bash
set -e

SOURCE_DIR=${1}
OBFUSCATED_DIR=${2}

GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'


echo -e "${BLUE}=================================${NC}"
echo -e "${BLUE}Python App Packaging Script${NC}"
echo -e "${BLUE}=================================${NC}"
echo


echo -e "${GREEN}Obfuscating code...${NC}"
echo "  Source: $SOURCE_DIR"
echo "  Output: $OBFUSCATED_DIR"

# Source the environment to get access to uv and pyarmor
. /home/user/.local/bin/env
. /home/user/the_venv/bin/activate

# Run pyarmor directly (it's already installed in the venv)
pyarmor gen -O "$OBFUSCATED_DIR" "$SOURCE_DIR"
