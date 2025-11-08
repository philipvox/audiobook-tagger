#!/bin/bash

# Audiobook Tagger - Git Repository Setup Script
# This script initializes a git repository and makes the first commit

set -e  # Exit on error

PROJECT_DIR=~/Desktop/Code\ Projects/audiobook-tagger-working

echo "🎯 Setting up Git repository for Audiobook Tagger..."
echo ""

# Navigate to project directory
cd "$PROJECT_DIR"

# Check if git is already initialized
if [ -d .git ]; then
    echo "⚠️  Git repository already exists!"
    echo ""
    read -p "Do you want to reinitialize? This will keep your history. (y/N) " -n 1 -r
    echo ""
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "❌ Aborted."
        exit 1
    fi
else
    echo "✅ No existing git repository found."
fi

# Initialize git repository
echo ""
echo "📦 Initializing git repository..."
git init
echo "✅ Git initialized"

# Create .gitignore if it doesn't exist
if [ ! -f .gitignore ]; then
    echo ""
    echo "📝 Creating .gitignore..."
    cat > .gitignore << 'GITIGNORE'
# Rust / Cargo
target/
**/*.rs.bk
*.pdb

# Node / NPM
node_modules/
npm-debug.log*
yarn-debug.log*
pnpm-debug.log*
dist/
dist-ssr/

# Tauri
src-tauri/target/
src-tauri/WixTools/

# Environment & Config
.env
.env.local
.env.*.local
*.secret
**/config.json
!**/config.example.json

# OS Files
.DS_Store
Thumbs.db
*.log

# IDE
.vscode/
.idea/
*.code-workspace

# Backup Files
*.backup
*.backup.*
*.bak
*.old

# Caches
.cache/
.eslintcache
genre_cache.json
metadata_cache.json

# User Data
user_data/
libraries/
GITIGNORE
    echo "✅ .gitignore created"
else
    echo "✅ .gitignore already exists"
fi

# Create README if it doesn't exist
if [ ! -f README.md ]; then
    echo ""
    echo "📝 Creating README.md..."
    cat > README.md << 'README'
# 🎧 Audiobook Tagger

A powerful desktop application for automatically tagging and organizing audiobook files using AI-powered metadata extraction.

## Features

- Multi-source metadata fetching (Google Books, Audible, GPT-5-nano)
- AudiobookShelf optimized tag writing
- Intelligent genre separation
- Correct narrator field placement
- Raw tag inspector for debugging
- Batch processing support

## Quick Start

```bash
# Install dependencies
npm install

# Run in development
npm run tauri dev

# Build for production
npm run tauri build
```

See full documentation for more details.
README
    echo "✅ README.md created"
else
    echo "✅ README.md already exists"
fi

# Add all files
echo ""
echo "📦 Adding files to git..."
git add .
echo "✅ Files staged"

# Create initial commit
echo ""
echo "💾 Creating initial commit..."
git commit -m "Initial commit: Audiobook Tagger

- Rust/Tauri backend with lofty for tag manipulation
- React frontend with Vite and TailwindCSS
- Multi-source metadata fetching (Google Books, Audible, GPT-5-nano)
- AudiobookShelf optimized tag writing
- Tag inspector tool for debugging
- Parallel processing support
- Smart caching system"

echo ""
echo "✅ Initial commit created!"

# Show status
echo ""
echo "📊 Current git status:"
git status

echo ""
echo "🎉 Git repository setup complete!"
echo ""
echo "Next steps:"
echo "  1. Create a GitHub repository"
echo "  2. Add remote: git remote add origin https://github.com/yourusername/audiobook-tagger.git"
echo "  3. Push code: git push -u origin main"
echo ""
echo "Or use GitHub CLI:"
echo "  gh repo create audiobook-tagger --public --source=. --remote=origin"
echo "  git push -u origin main"
