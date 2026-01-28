# Contributing to Track Leader

Thank you for your interest in contributing to Track Leader! This document provides guidelines for contributing.

## Getting Started

### Prerequisites

- Node.js 20+
- Rust (latest stable)
- PostgreSQL 15+ with PostGIS
- Docker (optional, for database)

### Development Setup

1. **Clone the repository:**
   ```bash
   git clone https://github.com/your-org/track-leader.git
   cd track-leader
   ```

2. **Start the development environment:**
   ```bash
   ./scripts/start-dev.sh
   ```

   This creates a tmux session with:
   - PostgreSQL (docker-compose)
   - Rust backend (port 3001)
   - Next.js frontend (port 3000)

3. **Verify setup:**
   - Frontend: http://localhost:3000
   - Backend: http://localhost:3001/health

### Environment Variables

Create a `.env.local` file for the frontend:
```bash
NEXT_PUBLIC_API_URL=http://localhost:3001
```

Backend uses environment variables from the start script.

## Development Workflow

### Making Changes

1. Create a branch for your work:
   ```bash
   jj new trunk
   jj describe -m "Add feature X"
   ```

2. Make your changes

3. Run tests:
   ```bash
   # Backend tests
   cargo nextest run

   # Frontend linting
   npm run lint

   # E2E tests
   npx playwright test
   ```

4. Commit your changes:
   ```bash
   jj commit -m "Add feature X"
   ```

### Code Style

#### TypeScript/React
- Use TypeScript strict mode
- Follow existing component patterns
- Use shadcn/ui components where possible

#### Rust
- Run `cargo +nightly fmt` before committing
- Run `cargo clippy` and fix warnings
- Use inline format strings: `println!("Hello, {name}!")`

### Testing

**Backend tests:**
```bash
cargo nextest run
```

**Frontend E2E tests:**
```bash
npx playwright test
```

**Running specific tests:**
```bash
# Backend - specific test
cargo nextest run test_name

# E2E - specific file
npx playwright test e2e/auth.spec.ts
```

## Pull Request Process

1. **Create a PR:**
   ```bash
   jj git push -c @
   gh pr create --head <bookmark-name>
   ```

2. **PR Guidelines:**
   - Keep changes focused and atomic
   - Write clear, concise descriptions
   - Reference any related issues
   - Include tests for new functionality

3. **Review Process:**
   - PRs require at least one approval
   - CI must pass (tests, linting)
   - Address feedback promptly

### PR Description Template

```markdown
## Summary
Brief description of changes

## Changes
- Change 1
- Change 2

## Testing
How to test these changes

## Screenshots
If applicable
```

## Project Structure

```
track-leader/
├── src/                    # Next.js frontend
│   ├── app/               # App Router pages
│   ├── components/        # React components
│   └── lib/               # Utilities
├── crates/tracks/         # Rust backend
│   ├── src/               # Source code
│   └── migrations/        # Database migrations
├── e2e/                   # E2E tests
├── docs/                  # Documentation
└── load-tests/            # k6 performance tests
```

## Architecture Decisions

Major architectural decisions are documented in:
- `docs/architecture/overview.md` - System design
- `docs/architecture/*.md` - Component-specific docs

When proposing significant changes, please:
1. Discuss in an issue first
2. Document the decision rationale
3. Update relevant architecture docs

## Reporting Issues

### Bug Reports

Include:
- Steps to reproduce
- Expected behavior
- Actual behavior
- Browser/environment details
- Screenshots if applicable

### Feature Requests

Include:
- Use case description
- Proposed solution
- Alternative approaches considered

## Code of Conduct

- Be respectful and inclusive
- Focus on constructive feedback
- Help newcomers learn

## Questions?

- Open a GitHub Discussion
- Check existing documentation
- Review closed issues/PRs

Thank you for contributing!
