# 🐳 Docker Guide for Audiobook Tagger

Complete guide to using Docker with this Tauri desktop application.

---

## ⚠️ Important: Desktop Apps and Docker

### What Tauri Is

Audiobook Tagger is a **desktop application** built with Tauri (Rust + WebView). It creates native macOS/Windows/Linux apps with a GUI.

### Docker Limitations

🚫 **You CANNOT run the GUI app inside Docker** (easily)
- Docker containers don't have displays
- GUI requires X11/Wayland forwarding (complex)
- Not practical for daily development

### What Docker IS Good For ✅

✅ **Consistent build environment** - Same tools everywhere
✅ **CI/CD pipelines** - Automated building and testing
✅ **Team consistency** - Everyone uses same versions
✅ **Clean builds** - Isolated from your system
✅ **Testing** - Run Rust tests in container

---

## 🎯 Use Cases

### Use Case 1: Consistent Development Environment

**Problem**: "Works on my machine" syndrome
**Solution**: Docker provides identical environment for all developers

### Use Case 2: CI/CD Builds

**Problem**: Need to build releases automatically
**Solution**: Docker container builds the app in GitHub Actions

### Use Case 3: Clean Builds

**Problem**: Local dependency conflicts
**Solution**: Build in fresh Docker container

### Use Case 4: Onboarding

**Problem**: New developer needs to install Rust, Node, Tauri deps
**Solution**: Just install Docker, run one command

---

## 📦 Files Created

### 1. Dockerfile.dev
Development/build environment with:
- Rust 1.75+
- Node.js 20.x
- All Tauri dependencies
- Pre-cached dependencies

### 2. docker-compose.yml
Services for:
- **dev**: Interactive development shell
- **build**: Automated builds

---

## 🚀 Quick Start

### Setup (One Time)

```bash
# 1. Make sure Docker is installed
docker --version

# 2. Navigate to project
cd ~/Desktop/Code\ Projects/audiobook-tagger

# 3. Build the Docker image
docker-compose build dev
```

### Daily Workflow

```bash
# Pull latest code from Git
git pull origin main

# Enter development container
docker-compose run --rm dev

# Inside container:
# - Run tests
cargo test

# - Build the app
npm run tauri build

# - Check dependencies
cargo check
npm audit

# Exit container
exit

# The built app is in: src-tauri/target/release/bundle/
# Run it on your HOST machine (not in Docker!)
```

---

## 💻 Development Workflow Options

### Option 1: Hybrid (Recommended)

**Develop on host, build in Docker**

```bash
# On your Mac/Linux:
git pull origin main           # Get latest
code .                        # Edit code in VS Code
npm run dev                   # Preview changes

# When ready to build:
docker-compose run --rm build # Build in Docker
```

**Pros:**
✅ Fast development with hot reload
✅ Use your favorite IDE
✅ Clean builds in Docker

**Cons:**
❌ Need Node/Rust on host

---

### Option 2: Full Docker Development

**Everything in Docker (except GUI)**

```bash
# Enter container
docker-compose run --rm dev bash

# Inside container:
cd /workspace

# Install new dependencies
npm install package-name
cargo add crate-name

# Run tests
cargo test
npm test

# Build
npm run tauri build

# Exit
exit

# Run built app on HOST
./src-tauri/target/release/bundle/macos/audiobook-tagger.app
```

**Pros:**
✅ Don't need tools on host
✅ Consistent environment
✅ Great for CI/CD

**Cons:**
❌ Can't run GUI in container
❌ Slower than native

---

### Option 3: Build-Only Docker

**Develop normally, use Docker for releases**

```bash
# Normal development on host
git pull
npm run tauri dev
# ... work normally ...
git commit && git push

# When releasing:
docker-compose run --rm build

# Creates release builds in:
# src-tauri/target/release/bundle/
```

**Pros:**
✅ Natural development flow
✅ Clean release builds

**Cons:**
❌ Still need tools on host

---

## 🔧 Common Docker Commands

### Build & Run

```bash
# Build image
docker-compose build dev

# Run container (interactive)
docker-compose run --rm dev

# Run specific command
docker-compose run --rm dev cargo test

# Build the app
docker-compose run --rm build
```

### Maintenance

```bash
# Remove old containers
docker-compose down

# Remove volumes (clean slate)
docker-compose down -v

# Rebuild from scratch
docker-compose build --no-cache dev

# See running containers
docker ps

# See images
docker images
```

### Debugging

```bash
# Enter running container
docker exec -it audiobook-tagger-dev bash

# View logs
docker-compose logs dev

# Check disk usage
docker system df

# Clean up everything
docker system prune -a
```

---

## 📊 Volume Mounting Explained

```yaml
volumes:
  - .:/workspace                    # Your code
  - cargo-cache:/usr/local/cargo    # Rust dependencies (fast)
  - node_modules:/workspace/node_modules  # Node deps (fast)
```

**What this means:**

1. **`.:/workspace`**: Your code is **shared** between host and container
   - Edit on host with VS Code
   - Changes instantly visible in container
   - Build outputs available on host

2. **`cargo-cache`**: Rust dependencies **cached** in Docker volume
   - Speeds up builds (downloads once)
   - Survives container restarts

3. **`node_modules`**: Node modules in Docker volume
   - Faster than mounting from host (especially macOS)
   - Isolated from host node_modules

---

## 🔄 Git + Docker Workflow

### Perfect Combined Workflow

```bash
# Morning: Get latest code
git pull origin main

# Work in hybrid mode:
# - Edit code on host in VS Code
# - Test changes: npm run tauri dev (on host)
# - Run tests in Docker occasionally:
docker-compose run --rm dev cargo test

# Commit your changes
git add .
git commit -m "feat: add new feature"

# Build clean release in Docker
docker-compose run --rm build

# Push to GitHub
git push origin main

# The built app is in src-tauri/target/release/bundle/
# Run it on your Mac!
```

### Multiple Machines Workflow

**On MacBook:**
```bash
git pull origin main
# Edit code
docker-compose run --rm dev cargo test
git add . && git commit -m "..." && git push
```

**On Desktop:**
```bash
git pull origin main              # Gets your MacBook work!
docker-compose run --rm dev bash  # Same environment as MacBook!
# Continue working
git add . && git commit -m "..." && git push
```

**Key Point**: Docker ensures **identical environment** on both machines!

---

## 🎯 CI/CD with Docker

### GitHub Actions Example

Create `.github/workflows/build.yml`:

```yaml
name: Build
on: [push]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Build in Docker
        run: |
          docker-compose build dev
          docker-compose run --rm build
      
      - name: Upload artifacts
        uses: actions/upload-artifact@v3
        with:
          name: audiobook-tagger
          path: src-tauri/target/release/bundle/
```

Now every push automatically builds your app in Docker! 🎉

---

## 🐛 Troubleshooting

### "Cannot connect to Docker daemon"

```bash
# Start Docker Desktop
open -a Docker

# Or check if running
docker ps
```

### "Permission denied"

```bash
# Fix Docker socket permissions
sudo chmod 666 /var/run/docker.sock
```

### "Disk space issues"

```bash
# Clean up Docker
docker system prune -a -f
docker volume prune -f
```

### "Build is slow"

```bash
# Use BuildKit for faster builds
export DOCKER_BUILDKIT=1
docker-compose build dev
```

### "Changes not reflected"

```bash
# Rebuild image
docker-compose build --no-cache dev

# Or just restart container
docker-compose restart dev
```

---

## 💡 Pro Tips

### Speed Up Builds

1. **Use BuildKit**:
   ```bash
   export DOCKER_BUILDKIT=1
   ```

2. **Pre-download dependencies**:
   ```bash
   docker-compose run --rm dev cargo fetch
   ```

3. **Mount cargo home**:
   Already done in docker-compose.yml!

### Save Disk Space

```bash
# After building, clean cargo target
docker-compose run --rm dev cargo clean

# Or exclude target from Docker:
echo "src-tauri/target" >> .dockerignore
```

### Debug Build Issues

```bash
# Build with verbose output
docker-compose run --rm dev cargo build -vv

# Or enter container and investigate
docker-compose run --rm dev bash
```

---

## 📋 Dockerfile Explanation

### Why Two Dockerfiles?

**Dockerfile** (if present): Production/release builds
**Dockerfile.dev**: Development environment

### What's Installed?

```dockerfile
# System dependencies
- build-essential      # Compilers
- libgtk-3-dev        # GTK for Tauri
- libwebkit2gtk-4.1   # WebView
- pkg-config          # Build tool

# Languages
- Rust 1.75+          # Backend
- Node.js 20.x        # Frontend

# Tools
- curl, wget, git     # Utilities
```

---

## 🎓 Learn More

### Docker Basics

```bash
# List containers
docker ps -a

# List images
docker images

# Remove container
docker rm container-name

# Remove image
docker rmi image-name

# View logs
docker logs container-name
```

### Docker Compose

```bash
# Start all services
docker-compose up

# Start in background
docker-compose up -d

# Stop all services
docker-compose down

# View status
docker-compose ps
```

---

## ✅ When to Use Docker

### Use Docker For:
✅ Consistent build environment
✅ CI/CD pipelines
✅ Team collaboration
✅ Testing
✅ Clean builds
✅ Isolating dependencies

### Don't Use Docker For:
❌ Running the GUI app daily
❌ Hot reload development (use host)
❌ Debugging GUI issues
❌ Performance-critical dev work

---

## 🎉 Summary

**Best Practice Workflow:**

1. **Development**: Work on host with hot reload
   ```bash
   npm run tauri dev
   ```

2. **Testing**: Run tests in Docker
   ```bash
   docker-compose run --rm dev cargo test
   ```

3. **Building**: Create releases in Docker
   ```bash
   docker-compose run --rm build
   ```

4. **Version Control**: Git for code management
   ```bash
   git pull → work → git push
   ```

**Result**: 
- ✅ Fast development on host
- ✅ Consistent builds in Docker
- ✅ Never re-download project (Git)
- ✅ Works on any machine

---

## 🚀 Quick Reference

```bash
# First time setup
docker-compose build dev

# Daily workflow
git pull                          # Get latest
# ... edit code on host ...
docker-compose run --rm dev bash  # Test in container
git add . && git commit && git push

# Build release
docker-compose run --rm build

# Clean up
docker-compose down -v
```

**You're all set!** 🎊
