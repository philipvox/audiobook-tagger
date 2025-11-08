# Contributing to Audiobook Tagger

First off, thank you for considering contributing to Audiobook Tagger! It's people like you that make this tool better for everyone.

## Code of Conduct

This project and everyone participating in it is governed by respect and professionalism. Please be kind and constructive.

## How Can I Contribute?

### Reporting Bugs

Before creating bug reports, please check the existing issues to avoid duplicates. When you create a bug report, include as many details as possible:

**Bug Report Template:**
```markdown
**Describe the bug**
A clear and concise description of what the bug is.

**To Reproduce**
Steps to reproduce the behavior:
1. Go to '...'
2. Click on '....'
3. See error

**Expected behavior**
What you expected to happen.

**Screenshots**
If applicable, add screenshots.

**Environment:**
 - OS: [e.g., macOS 14.2]
 - App Version: [e.g., 0.1.0]
 - Audio Format: [e.g., M4B]

**Additional context**
Any other context about the problem.
```

### Suggesting Features

Feature requests are welcome! Please:

1. Check if the feature has already been requested
2. Explain the use case clearly
3. Describe the expected behavior
4. Consider if it fits the project's scope

### Pull Requests

1. **Fork the repo** and create your branch from `main`
2. **Make your changes** with clear, descriptive commits
3. **Test your changes** thoroughly
4. **Update documentation** if needed
5. **Submit a PR** with a clear description

## Development Setup

### Prerequisites

- Node.js 18+
- Rust 1.70+
- Tauri CLI

### Getting Started

```bash
# Clone your fork
git clone https://github.com/yourusername/audiobook-tagger.git
cd audiobook-tagger

# Install dependencies
npm install

# Run in development mode
npm run tauri dev
```

### Project Structure

```
audiobook-tagger/
├── src/                 # React frontend
│   ├── App.jsx         # Main application
│   └── components/     # React components
├── src-tauri/          # Rust backend
│   └── src/            # Rust source files
├── tests/              # Test files
└── docs/               # Documentation
```

## Coding Standards

### Rust Code

- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` before committing
- Run `cargo clippy` and fix warnings
- Write tests for new functionality
- Document public APIs

```rust
// Good: Clear, documented function
/// Extracts metadata from an audio file
///
/// # Arguments
/// * `path` - Path to the audio file
///
/// # Returns
/// * `Result<Metadata>` - Extracted metadata or error
pub fn extract_metadata(path: &Path) -> Result<Metadata> {
    // Implementation
}
```

### JavaScript/React Code

- Use ES6+ features
- Follow React best practices
- Use functional components with hooks
- Keep components small and focused
- Use meaningful variable names

```jsx
// Good: Clear, focused component
function TagInspector({ file, onClose }) {
  const [tags, setTags] = useState(null);
  
  useEffect(() => {
    loadTags(file);
  }, [file]);
  
  return (
    <div className="inspector">
      {/* Component JSX */}
    </div>
  );
}
```

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add support for AAC files
fix: correct narrator field mapping
docs: update installation instructions
refactor: simplify tag writing logic
test: add genre separation tests
chore: update dependencies
```

### Git Workflow

1. **Create a branch** for your feature:
   ```bash
   git checkout -b feat/amazing-feature
   ```

2. **Make commits** with clear messages:
   ```bash
   git commit -m "feat: add cover art extraction"
   ```

3. **Keep your fork updated**:
   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

4. **Push to your fork**:
   ```bash
   git push origin feat/amazing-feature
   ```

5. **Open a Pull Request** on GitHub

## Testing

### Running Tests

```bash
# Rust tests
cd src-tauri
cargo test

# JavaScript tests (if added)
npm test
```

### Writing Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_genre_separation() {
        let genres = vec!["Mystery", "Thriller"];
        let result = separate_genre_tags(&genres);
        assert_eq!(result.len(), 2);
    }
}
```

## Documentation

- Update README.md for user-facing changes
- Update inline documentation for code changes
- Add examples for new features
- Update CHANGELOG.md

## Areas That Need Help

### High Priority
- [ ] Unit tests for tag writing
- [ ] Integration tests for metadata fetching
- [ ] Windows/Linux build testing
- [ ] Performance optimization for large libraries

### Medium Priority
- [ ] Additional audio format support
- [ ] Cover art management
- [ ] Custom genre mappings
- [ ] Batch export features

### Documentation
- [ ] Video tutorials
- [ ] API documentation
- [ ] Architecture diagrams
- [ ] Troubleshooting guide

## Questions?

Feel free to:
- Open a [Discussion](https://github.com/yourusername/audiobook-tagger/discussions)
- Ask in an [Issue](https://github.com/yourusername/audiobook-tagger/issues)
- Reach out to maintainers

## Recognition

Contributors will be acknowledged in:
- README.md Contributors section
- CHANGELOG.md for significant contributions
- GitHub's contributor graph

Thank you for contributing! 🎉
