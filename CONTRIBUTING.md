# Contributing to Ash

## Setup

```bash
git clone https://github.com/kenny1125nz/ash-lang.git
cd ash/ash
cargo build
cargo test
```

Requirements: Rust 1.70+

## Project Structure

```
ash/
├── .github/workflows/ci.yml   # CI/CD pipeline
├── ash/                       # Rust crate
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs            # CLI entry point
│       ├── lib.rs             # Library root
│       ├── token.rs           # Token types
│       ├── value.rs           # Runtime values
│       ├── ast.rs             # AST node types
│       ├── scope.rs           # Variable scoping
│       ├── compact.rs         # Compact config directives
│       ├── lexer.rs           # Lexer
│       ├── parser.rs          # Parser
│       ├── interpolation.rs   # String interpolation
│       ├── executor.rs        # Agent execution interface
│       ├── engine.rs          # Agent engine registry
│       ├── eval.rs            # AST evaluator
│       ├── tree.rs            # Directory-based orchestration
│       └── repl.rs            # Interactive REPL
├── tasks/                     # Task definitions for development
└── DISTRIBUTION.md            # Distribution and release plan
```

## Development

```bash
cd ash
cargo build          # Build the project
cargo test           # Run all tests
cargo run -- --help  # Run the CLI
```

## Pull Requests

- Ensure `cargo build` and `cargo test` pass locally
- Follow existing code style and conventions
- Keep PRs focused and describe the motivation

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
