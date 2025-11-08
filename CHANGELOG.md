# Changelog

All notable changes to Audiobook Tagger will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial release of Audiobook Tagger
- Multi-source metadata fetching (Google Books, Audible, GPT-5-nano)
- AudiobookShelf optimized tag writing with proper field mapping
- Tag inspector tool for debugging raw file metadata
- Parallel processing support (M4 Mac optimized)
- Smart caching system for faster re-scans
- Batch file selection and group-based organization
- Real-time progress tracking with speed metrics
- Configuration management with persistent settings

### Fixed
- Genre tags now written as separate entries (AudiobookShelf compatible)
- Narrator properly written to Composer field instead of Comment
- Description cleaning to remove debug output from GPT responses
- GPT-5-nano integration with correct model parameters

### Technical
- Rust/Tauri backend with lofty for audio tag manipulation
- React 18 frontend with Vite and TailwindCSS
- OpenAI GPT-5-nano for AI-powered metadata enhancement
- Google Books API integration for metadata retrieval
- Audible web scraping support for additional metadata

## [0.1.0] - 2025-01-XX

Initial development release.

---

## Version History Notes

### Future Planned Features
- Support for additional audio formats (AAC, OGG, WAV)
- Cover art management and embedding
- Direct AudiobookShelf server upload
- Custom genre mapping configuration
- Batch export to CSV/JSON
- Cross-platform builds (macOS, Windows, Linux)
- Integration test suite
- CI/CD pipeline

### Known Issues
- Audible scraping may break if website structure changes
- Large libraries (1000+ files) may take significant time to scan
- GPT API rate limits may affect batch processing

---

[Unreleased]: https://github.com/yourusername/audiobook-tagger/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/yourusername/audiobook-tagger/releases/tag/v0.1.0
