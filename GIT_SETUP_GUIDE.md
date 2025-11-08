# 🚀 Git Repository Setup Guide

Complete guide to creating a Git repository for Audiobook Tagger and pushing it to GitHub.

---

## 📋 Prerequisites

- [x] Git installed (`git --version`)
- [ ] GitHub account created
- [ ] GitHub CLI installed (optional, but recommended: `gh --version`)

---

## 🎯 Quick Setup (Automated)

### Option 1: Using the Setup Script (Easiest)

```bash
cd ~/Desktop/Code\ Projects/audiobook-tagger-working

# Make script executable
chmod +x init_git.sh

# Run it!
./init_git.sh
```

This script will:
1. ✅ Initialize git repository
2. ✅ Create .gitignore
3. ✅ Create README.md
4. ✅ Stage all files
5. ✅ Create initial commit

---

## 🛠️ Manual Setup (Step-by-Step)

### Step 1: Navigate to Project

```bash
cd ~/Desktop/Code\ Projects/audiobook-tagger-working
```

### Step 2: Initialize Git

```bash
git init
```

### Step 3: Add Essential Files

Copy these files to your project root:

```bash
# From the output directory, copy:
cp /path/to/outputs/.gitignore .
cp /path/to/outputs/README.md .
cp /path/to/outputs/LICENSE .
cp /path/to/outputs/CHANGELOG.md .
cp /path/to/outputs/CONTRIBUTING.md .
```

Or create them manually using the provided templates.

### Step 4: Stage Files

```bash
# Add all files
git add .

# Check what will be committed
git status
```

### Step 5: Create Initial Commit

```bash
git commit -m "Initial commit: Audiobook Tagger

- Rust/Tauri backend with lofty for tag manipulation
- React frontend with Vite and TailwindCSS
- Multi-source metadata fetching (Google Books, Audible, GPT)
- AudiobookShelf optimized tag writing
- Tag inspector tool for debugging
- Parallel processing support"
```

---

## 🌐 Push to GitHub

### Option A: Using GitHub CLI (Recommended)

```bash
# Create GitHub repository and push in one command
gh repo create audiobook-tagger --public --source=. --remote=origin --push

# Or if you want it private:
gh repo create audiobook-tagger --private --source=. --remote=origin --push
```

Done! Your repo is now on GitHub at: `https://github.com/yourusername/audiobook-tagger`

### Option B: Using GitHub Website

1. **Go to GitHub** and create a new repository:
   - Name: `audiobook-tagger`
   - Description: "AI-powered audiobook metadata tagging tool"
   - Public or Private: Your choice
   - **DO NOT** initialize with README (we already have one)

2. **Connect your local repo to GitHub**:

```bash
# Add the remote (replace with your username)
git remote add origin https://github.com/YOURUSERNAME/audiobook-tagger.git

# Rename branch to main (if needed)
git branch -M main

# Push to GitHub
git push -u origin main
```

3. **Verify on GitHub**: Visit your repository URL!

---

## 📦 What Gets Committed?

### ✅ Included in Git:

- Source code (`.rs`, `.jsx`, `.js`)
- Configuration files (`Cargo.toml`, `package.json`, `tauri.conf.json`)
- Documentation (`README.md`, `CHANGELOG.md`, `CONTRIBUTING.md`)
- License (`LICENSE`)
- Assets and styles

### ❌ Excluded from Git (.gitignore):

- `node_modules/` - Node dependencies
- `target/` - Rust build artifacts
- `dist/` - Build outputs
- `.env` - Environment variables
- `*.backup` - Backup files
- `.DS_Store` - macOS files
- IDE folders (`.vscode/`, `.idea/`)
- Log files
- Cache files

---

## 🔍 Verify Your Setup

```bash
# Check git status
git status

# View commit history
git log --oneline

# Check remote connection
git remote -v

# View what's ignored
git status --ignored
```

---

## 📝 Next Steps After Setup

### 1. Add Repository Description

On GitHub:
- Go to your repository
- Click "About" gear icon
- Add description: "AI-powered audiobook metadata tagging tool for AudiobookShelf"
- Add topics: `audiobook`, `tauri`, `rust`, `react`, `metadata`, `audiobookshelf`

### 2. Enable GitHub Features

- **Issues**: Already enabled by default
- **Discussions**: Settings → Features → Enable Discussions
- **Wiki**: Optional, for extensive documentation
- **Projects**: For task tracking

### 3. Protect Main Branch

Settings → Branches → Add rule for `main`:
- ✅ Require pull request before merging
- ✅ Require approvals: 1
- ✅ Require status checks to pass (when CI is added)

### 4. Add GitHub Actions (Optional)

Create `.github/workflows/ci.yml` for:
- Automated testing
- Build verification
- Release automation

### 5. Set Up Branch Protection

```bash
# Create a develop branch for active development
git checkout -b develop
git push -u origin develop

# Work on features in feature branches
git checkout -b feat/new-feature
```

---

## 🎨 Customize Your Repository

### Update README.md

Replace placeholder URLs:
```markdown
<!-- Change this: -->
https://github.com/yourusername/audiobook-tagger

<!-- To your actual URL: -->
https://github.com/actualusername/audiobook-tagger
```

### Update LICENSE

Replace `[Your Name]` with your actual name or organization.

### Add a Repository Banner (Optional)

Create a banner image and add to README:
```markdown
![Banner](docs/images/banner.png)
```

---

## 🤝 Collaboration Setup

### Invite Collaborators

Settings → Collaborators → Add people

### Set Up Labels

Issues → Labels → Add:
- `bug` - Something isn't working
- `enhancement` - New feature request
- `documentation` - Documentation improvements
- `good first issue` - Good for newcomers
- `help wanted` - Extra attention needed

### Create Issue Templates

`.github/ISSUE_TEMPLATE/bug_report.md`
`.github/ISSUE_TEMPLATE/feature_request.md`

---

## 🔒 Security Best Practices

### 1. Never Commit Secrets

Already in `.gitignore`:
- `.env` files
- API keys
- Tokens

### 2. Check for Accidental Commits

```bash
# Search for potential secrets
git log --all --full-history --source --find-copies-harder -S "api_key"
```

### 3. Use Environment Variables

```bash
# Example .env file (never commit this!)
OPENAI_API_KEY=sk-...
AUDIOBOOKSHELF_TOKEN=...
```

### 4. If You Accidentally Commit a Secret

```bash
# Remove from history (CAREFUL!)
git filter-branch --force --index-filter \
  "git rm --cached --ignore-unmatch path/to/file" \
  --prune-empty --tag-name-filter cat -- --all

# Then rotate the secret immediately!
```

---

## 📊 Git Workflow Best Practices

### Daily Workflow

```bash
# Start your day
git pull origin main

# Create feature branch
git checkout -b feat/awesome-feature

# Make changes and commit
git add .
git commit -m "feat: add awesome feature"

# Push to your fork/branch
git push origin feat/awesome-feature

# Create PR on GitHub
```

### Commit Message Format

```
type(scope): subject

body (optional)

footer (optional)
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `style`: Formatting
- `refactor`: Code restructuring
- `test`: Adding tests
- `chore`: Maintenance

**Examples:**
```
feat(tags): add support for FLAC files
fix(scanner): correct series detection regex
docs(readme): update installation instructions
```

---

## 🆘 Troubleshooting

### "fatal: not a git repository"

```bash
# Make sure you're in the project directory
cd ~/Desktop/Code\ Projects/audiobook-tagger-working

# Initialize git
git init
```

### "remote origin already exists"

```bash
# Remove old remote
git remote remove origin

# Add new remote
git remote add origin https://github.com/yourusername/audiobook-tagger.git
```

### "failed to push some refs"

```bash
# Pull first, then push
git pull origin main --rebase
git push origin main
```

### Large Files Rejected

```bash
# If you accidentally added large audio files
git rm --cached path/to/large/file
git commit --amend
```

---

## ✅ Final Checklist

Before sharing your repository:

- [ ] `.gitignore` is comprehensive
- [ ] README.md is complete and accurate
- [ ] LICENSE file is present
- [ ] No secrets committed
- [ ] All URLs updated with your username
- [ ] Repository description added on GitHub
- [ ] Topics/tags added on GitHub
- [ ] Issues and Discussions enabled
- [ ] First release tagged (optional)

---

## 🎉 You're Done!

Your Audiobook Tagger is now on GitHub! Share it with:

```markdown
Check out my audiobook tagger: https://github.com/yourusername/audiobook-tagger
```

Happy coding! 🚀
