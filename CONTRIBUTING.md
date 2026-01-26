# Contributing to CLIAI

Thank you for your interest in contributing to CLIAI! We welcome contributions from the community and are excited to see what you'll bring to the project.

## 🚀 Getting Started

### Prerequisites

- Rust 1.70 or later
- Git
- Ollama (for testing AI functionality)

### Development Setup

1. **Fork and Clone**
   ```bash
   git clone https://github.com/yourusername/cliai.git
   cd cliai
   ```

2. **Install Dependencies**
   ```bash
   cargo build
   ```

3. **Set up Ollama** (for testing)
   ```bash
   # Install Ollama
   curl -fsSL https://ollama.ai/install.sh | sh
   
   # Pull a test model
   ollama pull mistral
   ```

4. **Run Tests**
   ```bash
   # Unit tests
   cargo test
   
   # Integration tests with AI
   cargo run -- test --quick
   ```

## 🎯 How to Contribute

### Reporting Issues

- Use the [GitHub Issues](https://github.com/yourusername/cliai/issues) page
- Search existing issues before creating a new one
- Include detailed reproduction steps
- Provide system information (OS, Rust version, etc.)

### Suggesting Features

- Open a [GitHub Discussion](https://github.com/yourusername/cliai/discussions) first
- Describe the use case and expected behavior
- Consider implementation complexity and maintenance burden

### Code Contributions

1. **Create a Feature Branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make Your Changes**
   - Follow the existing code style
   - Add tests for new functionality
   - Update documentation as needed

3. **Test Your Changes**
   ```bash
   # Run all tests
   cargo test
   
   # Test with real AI (requires Ollama)
   cargo run -- test --categories "your-test-category"
   
   # Check formatting
   cargo fmt --check
   
   # Run clippy
   cargo clippy -- -D warnings
   ```

4. **Commit Your Changes**
   ```bash
   git add .
   git commit -m "feat: add amazing new feature"
   ```

5. **Push and Create PR**
   ```bash
   git push origin feature/your-feature-name
   ```

## 📝 Code Style Guidelines

### Rust Code Style

- Use `cargo fmt` for consistent formatting
- Follow Rust naming conventions
- Add documentation comments for public APIs
- Use `clippy` suggestions to improve code quality

### Commit Messages

We follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

- `feat:` - New features
- `fix:` - Bug fixes
- `docs:` - Documentation changes
- `style:` - Code style changes (formatting, etc.)
- `refactor:` - Code refactoring
- `test:` - Adding or updating tests
- `chore:` - Maintenance tasks

Examples:
```
feat: add support for custom AI providers
fix: resolve command validation edge case
docs: update installation instructions
test: add integration tests for file operations
```

## 🏗️ Project Structure

Understanding the codebase:

```
src/
├── src/
│   ├── main.rs              # CLI interface and entry point
│   ├── lib.rs               # Library exports
│   ├── agents/              # AI orchestration
│   │   ├── mod.rs           # Main orchestrator
│   │   └── profiles.rs      # AI model profiles
│   ├── config.rs            # Configuration management
│   ├── context.rs           # System context gathering
│   ├── execution.rs         # Command execution
│   ├── validation.rs        # Command validation
│   ├── providers.rs         # AI provider implementations
│   ├── history.rs           # Chat history
│   ├── performance.rs       # Performance monitoring
│   ├── error_handling.rs    # Error handling
│   ├── logging.rs           # Privacy-preserving logging
│   └── test_suite.rs        # Testing framework
├── Cargo.toml               # Dependencies and metadata
├── README.md                # Project documentation
└── CONTRIBUTING.md          # This file
```

## 🧪 Testing Guidelines

### Unit Tests

- Write tests for all public functions
- Use descriptive test names
- Test both success and error cases
- Mock external dependencies when possible

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_command_validation_success() {
        // Test implementation
    }
    
    #[test]
    fn test_command_validation_failure() {
        // Test implementation
    }
}
```

### Integration Tests

- Test real AI interactions when possible
- Use the built-in test suite for comprehensive testing
- Add new test categories for new features

### Performance Tests

- Monitor performance impact of changes
- Use the built-in performance monitoring
- Add benchmarks for critical paths

## 🔒 Security Considerations

### Command Safety

- All command generation must go through validation
- New validation rules should be thoroughly tested
- Consider security implications of new features

### Privacy Protection

- Never log user commands or prompts in production
- Ensure debug mode requires explicit consent
- Review data handling in new features

### AI Provider Security

- Validate all AI responses
- Implement proper error handling for AI failures
- Consider rate limiting and abuse prevention

## 📚 Documentation

### Code Documentation

- Document all public APIs with rustdoc comments
- Include examples in documentation
- Explain complex algorithms and design decisions

### User Documentation

- Update README.md for user-facing changes
- Add examples for new features
- Update configuration documentation

## 🎉 Recognition

Contributors will be recognized in:

- GitHub contributors list
- Release notes for significant contributions
- README acknowledgments section

## ❓ Questions?

- Open a [GitHub Discussion](https://github.com/yourusername/cliai/discussions)
- Check existing issues and discussions
- Reach out to maintainers

## 📋 Pull Request Checklist

Before submitting your PR, ensure:

- [ ] Code follows project style guidelines
- [ ] All tests pass (`cargo test`)
- [ ] New functionality includes tests
- [ ] Documentation is updated
- [ ] Commit messages follow conventional format
- [ ] No sensitive information is included
- [ ] Performance impact is considered
- [ ] Security implications are reviewed

Thank you for contributing to CLIAI! 🚀