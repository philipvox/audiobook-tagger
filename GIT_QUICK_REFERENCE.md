# ⚡ Git Setup - Quick Reference Card

## 🎯 The Fastest Way (2 Minutes)

```bash
cd ~/Desktop/Code\ Projects/audiobook-tagger-working
chmod +x setup_repo.sh
./setup_repo.sh
```

**Done!** Script does everything automatically. ✅

---

## 📝 Essential Commands

### Initialize
```bash
git init                 # Start git
git branch -M main       # Rename to main
```

### Commit
```bash
git add .               # Stage all files
git status              # Check what's staged
git commit -m "message" # Commit with message
```

### GitHub
```bash
# With GitHub CLI (easiest):
gh repo create audiobook-tagger --public --source=. --remote=origin --push

# Manual:
git remote add origin https://github.com/USER/audiobook-tagger.git
git push -u origin main
```

---

## 📦 Files Needed

| File | Get From |
|------|----------|
| .gitignore | `/mnt/user-data/outputs/.gitignore` |
| README.md | `/mnt/user-data/outputs/README.md` |
| LICENSE | `/mnt/user-data/outputs/LICENSE` |
| CHANGELOG.md | `/mnt/user-data/outputs/CHANGELOG.md` |
| CONTRIBUTING.md | `/mnt/user-data/outputs/CONTRIBUTING.md` |

---

## 🚫 What Gets Ignored

- `node_modules/` - 200MB+ of dependencies
- `target/` - 300MB+ of Rust builds
- `dist/` - Build outputs
- `.env` - Your secrets
- `*.backup` - Backup files
- `.DS_Store` - macOS junk

**Total saved**: ~500MB+ ✅

---

## ✅ Before Pushing Checklist

- [ ] `.gitignore` exists
- [ ] README.md updated
- [ ] No secrets committed (check `.env`)
- [ ] LICENSE has your name
- [ ] Initial commit made

---

## 🆘 Quick Fixes

**Already a git repo?**
```bash
git status  # Check current state
```

**Need to undo last commit?**
```bash
git reset --soft HEAD~1
```

**Accidentally committed secret?**
```bash
git rm --cached .env
git commit --amend
# Then rotate the secret!
```

**Wrong remote URL?**
```bash
git remote remove origin
git remote add origin NEW_URL
```

---

## 🎓 Git Workflow

```bash
# Daily workflow
git pull origin main              # Update
git checkout -b feat/new-feature  # New branch
# ... make changes ...
git add .                        # Stage
git commit -m "feat: add thing"  # Commit
git push origin feat/new-feature # Push
# ... create PR on GitHub ...
```

---

## 🔗 Useful Links

- **Setup Guide**: `GIT_SETUP_GUIDE.md`
- **Full Summary**: `GIT_PACKAGE_SUMMARY.md`
- **GitHub Docs**: https://docs.github.com
- **Git Docs**: https://git-scm.com/doc

---

## 💡 Pro Tips

1. **Commit often**: Small, frequent commits are better
2. **Write clear messages**: Use conventional commits format
3. **Branch for features**: Don't work directly on main
4. **Pull before push**: Avoid merge conflicts
5. **Review before commit**: Check `git status` and `git diff`

---

## ⚡ One-Liners

```bash
# Quick commit
git add . && git commit -m "Update" && git push

# Delete branch
git branch -d branch-name

# See changes
git diff

# See history
git log --oneline

# Undo changes
git restore file.js
```

---

**Remember**: The setup script does all of this for you! 🚀
