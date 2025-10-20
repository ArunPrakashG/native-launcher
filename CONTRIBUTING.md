# Contributing to Native Launcher

Thank you for your interest in contributing to Native Launcher! This document provides guidelines for contributing to the project.

## Code of Conduct

Be respectful, inclusive, and professional in all interactions.

## How to Contribute

### Reporting Bugs

1. **Search existing issues** to avoid duplicates
2. **Use the bug report template** when creating a new issue
3. **Include details:**
   - Operating system and version
   - Compositor (Sway, Hyprland, etc.)
   - GTK version
   - Steps to reproduce
   - Expected vs actual behavior
   - Logs (run with `RUST_LOG=debug`)

### Suggesting Features

1. **Check the roadmap** in `plans.md` to see if it's already planned
2. **Create a feature request issue** with:
   - Clear description of the feature
   - Use cases and benefits
   - Potential implementation approach

### Pull Requests

1. **Fork the repository** and create a feature branch
2. **Follow the coding standards** (see below)
3. **Write tests** for new functionality
4. **Update documentation** as needed
5. **Run tests and lints** before submitting
6. **Write clear commit messages**

#### PR Checklist

- [ ] Code follows Rust style guidelines (`cargo fmt`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] Tests pass (`cargo test`)
- [ ] Documentation updated
- [ ] Changelog updated (for significant changes)

## Development Setup

```bash
# Clone your fork
git clone https://github.com/your-username/native-launcher
cd native-launcher

# Create a feature branch
git checkout -b feature/my-feature

# Make changes and commit
git add .
git commit -m "Add my feature"

# Push to your fork
git push origin feature/my-feature
```

## Coding Standards

### Rust Style

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` for formatting
- Address all `cargo clippy` warnings
- Write idiomatic Rust code

### Documentation

- Add doc comments for public APIs
- Include examples in doc comments
- Update README.md for user-facing changes

### Testing

- Write unit tests for business logic
- Add integration tests for workflows
- Benchmark performance-critical code

### Commit Messages

```
type(scope): brief description

Longer explanation if needed.

Fixes #123
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

## Project Structure

See `plans.md` for detailed architecture and module organization.

## Areas for Contribution

### Good First Issues

- Documentation improvements
- Icon theme compatibility
- Additional compositor configurations
- Bug fixes

### Advanced Contributions

- Plugin development
- Performance optimizations
- New search algorithms
- UI/UX improvements

## Getting Help

- **Questions**: Open a discussion on GitHub
- **Issues**: Check existing issues or create a new one
- **Real-time chat**: _TBD_

## Recognition

Contributors will be acknowledged in:

- The AUTHORS file
- Release notes
- README contributors section

Thank you for contributing! 🎉
