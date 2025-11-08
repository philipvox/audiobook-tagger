#!/bin/bash

# 🚀 Audiobook Tagger - Complete Git & GitHub Setup
# This script does EVERYTHING to set up your repository

set -e  # Exit on error

PROJECT_DIR=~/Desktop/Code\ Projects/audiobook-tagger-working
REPO_NAME="audiobook-tagger"

echo "╔════════════════════════════════════════════════════╗"
echo "║   Audiobook Tagger - Git Repository Setup   ║"
echo "╚════════════════════════════════════════════════════╝"
echo ""

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Navigate to project
echo -e "${BLUE}📂 Navigating to project directory...${NC}"
cd "$PROJECT_DIR"
echo -e "${GREEN}✅ In: $(pwd)${NC}"
echo ""

# Check if git exists
if ! command -v git &> /dev/null; then
    echo -e "${RED}❌ Git is not installed!${NC}"
    echo "Install it from: https://git-scm.com/"
    exit 1
fi

# Check if already a git repo
if [ -d .git ]; then
    echo -e "${YELLOW}⚠️  Git repository already exists${NC}"
    echo ""
    read -p "Do you want to continue? This will not overwrite your history. (y/N) " -n 1 -r
    echo ""
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo -e "${RED}❌ Aborted${NC}"
        exit 1
    fi
else
    # Initialize git
    echo -e "${BLUE}📦 Initializing git repository...${NC}"
    git init
    git branch -M main
    echo -e "${GREEN}✅ Git initialized${NC}"
fi

echo ""

# Create/Update .gitignore
echo -e "${BLUE}📝 Setting up .gitignore...${NC}"
cat > .gitignore << 'EOF'
# Rust / Cargo
target/
**/*.rs.bk
*.pdb

# Node / NPM
node_modules/
npm-debug.log*
yarn-debug.log*
dist/
dist-ssr/

# Tauri
src-tauri/target/
src-tauri/WixTools/

# Environment
.env
.env.local
.env.*.local
*.secret

# OS
.DS_Store
Thumbs.db
*.log

# IDE
.vscode/
!.vscode/settings.json
!.vscode/tasks.json
.idea/
*.code-workspace

# Backups
*.backup
*.backup.*
*.bak

# Caches
.cache/
.eslintcache
genre_cache.json
metadata_cache.json

# User Data
user_data/
libraries/
EOF
echo -e "${GREEN}✅ .gitignore ready${NC}"

# Create/Update README
echo -e "${BLUE}📝 Setting up README.md...${NC}"
if [ ! -f README.md ]; then
cat > README.md << 'EOF'
# 🎧 Audiobook Tagger

AI-powered audiobook metadata tagging tool optimized for AudiobookShelf.

## ✨ Features

- 🔍 Multi-source metadata (Google Books, Audible, GPT-5-nano)
- 📝 AudiobookShelf optimized tag writing
- 🛠️ Raw tag inspector for debugging
- ⚡ Parallel processing (M4 Mac optimized)
- 📚 Genre separation and narrator field correction

## 🚀 Quick Start

```bash
# Install dependencies
npm install

# Run development
npm run tauri dev

# Build
npm run tauri build
```

## 🎯 AudiobookShelf Integration

Writes tags exactly how AudiobookShelf expects:
- ✅ Multiple separate genre tags
- ✅ Narrator in Composer field
- ✅ Clean descriptions
- ✅ Series information

## 📄 License

MIT License - see LICENSE file
EOF
    echo -e "${GREEN}✅ README.md created${NC}"
else
    echo -e "${GREEN}✅ README.md exists${NC}"
fi

# Create LICENSE if missing
if [ ! -f LICENSE ]; then
    echo -e "${BLUE}📝 Creating LICENSE...${NC}"
    cat > LICENSE << 'EOF'
MIT License

Copyright (c) 2025 [Your Name]

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
EOF
    echo -e "${GREEN}✅ LICENSE created${NC}"
    echo -e "${YELLOW}⚠️  Don't forget to update [Your Name] in LICENSE${NC}"
else
    echo -e "${GREEN}✅ LICENSE exists${NC}"
fi

echo ""

# Stage files
echo -e "${BLUE}📦 Staging files...${NC}"
git add .

# Show what will be committed
echo ""
echo -e "${BLUE}Files to be committed:${NC}"
git status --short | head -20
echo ""

# Create commit if needed
if git diff --staged --quiet; then
    echo -e "${YELLOW}⚠️  No changes to commit${NC}"
else
    echo -e "${BLUE}💾 Creating commit...${NC}"
    git commit -m "Initial commit: Audiobook Tagger

- Rust/Tauri backend with lofty for tag manipulation
- React frontend with Vite and TailwindCSS
- Multi-source metadata fetching (Google Books, Audible, GPT-5-nano)
- AudiobookShelf optimized tag writing
- Tag inspector tool for debugging
- Parallel processing support"
    echo -e "${GREEN}✅ Commit created${NC}"
fi

echo ""

# Check for GitHub CLI
if command -v gh &> /dev/null; then
    echo -e "${GREEN}✅ GitHub CLI detected${NC}"
    echo ""
    echo "Would you like to create a GitHub repository now?"
    echo "1) Yes, create public repository"
    echo "2) Yes, create private repository"
    echo "3) No, I'll do it manually later"
    echo ""
    read -p "Choose (1/2/3): " -n 1 -r choice
    echo ""
    
    case $choice in
        1)
            echo -e "${BLUE}🌐 Creating public GitHub repository...${NC}"
            gh repo create "$REPO_NAME" --public --source=. --remote=origin --push
            echo -e "${GREEN}✅ Repository created and pushed!${NC}"
            echo -e "${GREEN}🎉 View at: https://github.com/$(gh api user -q .login)/$REPO_NAME${NC}"
            ;;
        2)
            echo -e "${BLUE}🔒 Creating private GitHub repository...${NC}"
            gh repo create "$REPO_NAME" --private --source=. --remote=origin --push
            echo -e "${GREEN}✅ Repository created and pushed!${NC}"
            echo -e "${GREEN}🎉 View at: https://github.com/$(gh api user -q .login)/$REPO_NAME${NC}"
            ;;
        3)
            echo -e "${YELLOW}⏭️  Skipping GitHub creation${NC}"
            ;;
        *)
            echo -e "${YELLOW}⏭️  Invalid choice, skipping${NC}"
            ;;
    esac
else
    echo -e "${YELLOW}⚠️  GitHub CLI not found${NC}"
    echo ""
    echo "To create GitHub repository manually:"
    echo "1. Go to https://github.com/new"
    echo "2. Create repository named: $REPO_NAME"
    echo "3. Don't initialize with README"
    echo "4. Then run:"
    echo ""
    echo -e "${BLUE}   git remote add origin https://github.com/YOURUSERNAME/$REPO_NAME.git${NC}"
    echo -e "${BLUE}   git push -u origin main${NC}"
    echo ""
    echo "Or install GitHub CLI:"
    echo -e "${BLUE}   brew install gh${NC}"
fi

echo ""
echo "╔════════════════════════════════════════════════════╗"
echo "║              Setup Complete! 🎉                    ║"
echo "╚════════════════════════════════════════════════════╝"
echo ""
echo -e "${GREEN}✅ Git repository initialized${NC}"
echo -e "${GREEN}✅ Essential files created${NC}"
echo -e "${GREEN}✅ Initial commit made${NC}"
echo ""
echo "Next steps:"
echo "  • Update LICENSE with your name"
echo "  • Customize README.md"
echo "  • Push to GitHub (if not done automatically)"
echo "  • Add repository description and topics on GitHub"
echo ""
echo "Happy coding! 🚀"
