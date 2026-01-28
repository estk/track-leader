# CLAUDE.md

Essential instructions for Claude Code. For detailed documentation, see [docs/ai/](./docs/ai/).

## Quick Start

```bash
# Start dev environment (foreground with logs)
./scripts/dev.sh

# Start detached
./scripts/dev.sh -d

# Check status
./scripts/dev.sh status

# View logs
./scripts/dev.sh logs [backend|frontend|postgres]

# Stop
./scripts/dev.sh stop
```

Supports random ports for parallel workspaces. See [docs/ai/development.md](./docs/ai/development.md) for details.

## Key Rules

- **Version Control**: Use `jj` (Jujutsu), never `git`
- **Formatting**: `cargo +nightly fmt` for Rust
- **Testing**: `cargo nextest run` for Rust tests
- **Imports**: Import unused traits as `use MyTrait as _`
- **Format strings**: Use inline variables: `println!("Hello, {name}!")`

## Token-Efficient Build Commands

When running builds, filter verbose output to save context tokens:

```bash
# Cargo - filter out compile/download progress, show only errors/warnings
cargo build 2>&1 | grep -v -E "^\s*(Compiling|Downloading|Downloaded|Fresh|Blocking|Updating)"

# Cargo test - show only test results and failures
cargo nextest run 2>&1 | grep -v -E "^\s*(Compiling|Fresh|Blocking)"

# npm/Next.js - filter module counts and progress
npm run build 2>&1 | grep -v -E "^\s*(○|✓|▲).*modules"
```

For build errors, the unfiltered output is often needed - run without filters when debugging.

## Project Structure

```
track-leader/
├── src/                    # Next.js frontend
├── crates/tracks/          # Rust backend
├── scripts/                # Dev environment scripts
└── docs/ai/                # AI reference documentation
```

## Additional Documentation

- [docs/ai/index.md](./docs/ai/index.md) - Project overview and tech stack
- [docs/ai/development.md](./docs/ai/development.md) - Development environment details
- [docs/ai/context.md](./docs/ai/context.md) - Gotchas and patterns
