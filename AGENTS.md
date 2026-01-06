# AI Assistant Context

This file defines the context and operating rules for AI agents working in this repository.

**Purpose**: `ai-nexus` serves as the central hub for **best practice files** and **workflows**. It is responsible for syncing these standards to all other repositories in `/Users/thorben.woelk/repos/`. Additionally, it hosts a **Dashboard App** to inspect the status and health of those repositories.

## Ground Rules

### General
- **Commits**: Follow [Conventional Commits](file:///.agent/workflows/commit-conventions.md.template) (v1.0.0).
- **Deployment**: Use [Graphite CLI (gt)](file:///.agent/workflows/graphite-usage.md.template) for stacked PRs.
- **Documentation Sync**: `AGENTS.md`, `CLAUDE.md`, `GEMINI.md`, and `WARP.md` MUST be kept strictly in sync. Core guidelines and best practices MUST be identical across all files, while agent-specific instructions (e.g., for commands) should be handled in their respective files.
- **Project Structure**: Do NOT add new root-level uppercase `.md` files. Root-level uppercase `.md` files are strictly reserved for AI Assistant Context (AGENTS, CLAUDE, GEMINI, WARP).
- **Startup**: The entry point for starting the application/service MUST be named `start_app.sh` in the root directory.
- **Tracking**: `TODO.md` is the EXCLUSIVE source of truth for all project and task tracking. Do NOT create or use `task.md` or other tracking files in the repository root.

### Python
- **Dependency Management**: ALWAYS use `uv`.
- **Linting**: Use `ruff`.

### JavaScript / TypeScript
- **Dependency Management**: Prefer `bun` over `npm`.

### Flutter / Dart
- **Dependency Management**: Use `flutter pub`.
- **Linting**: Strictly follow `analysis_options.yaml`.

### Rust
- **Dependency Management**: ALWAYS use `cargo`.
- **Linting**: Use `cargo fmt` and `cargo clippy`.

### Commands
- **Execution**: When a user provides a keyword command (e.g., `deploy`), follow the corresponding command definition in `.agent/commands/`.

---

## AI Coding Best Practices

### General Principles
- **Readability**: Produce human-readable code.
    - Small Functions: Aim for functions <= 20 lines.
    - Intuitive naming.
- **Isolate Side-Effects**: Confine I/O to specific boundary functions.
- **Testing**: Mandatory [TDD Workflow](file:///.agent/workflows/tdd-workflow.md.template).
- **Parallel Development**: Use [Nested Worktrees](file:///.agent/workflows/parallel-development.md.template) in `.worktrees/`.
- **Architecture**: The Backend is the source of truth for all state mutations. Frontend refetches state.

### Python-Specific Guidelines
- **Standard**: Follow [PEP 8](https://peps.python.org/pep-0008/).
- **Naming**: 
  - Modules/Packages: `snake_case` (nouns).
  - Classes: `PascalCase` (singular nouns).
  - Functions: `snake_case` (verb + object).
- **Organization**:
  - Entry point: Minimal orchestration (`app.py`).
  - Handlers: Map data to services (`handlers/`).
  - Services: Business logic (`services/`).
  - Data Access: DB/API clients (`repositories/`).
  - Utilities: Shared helpers (`utils/`).

### TypeScript-Specific Guidelines
- **Naming**:
  - Files/Folders: `kebab-case`.
  - Classes/Types/Interfaces: `PascalCase`.
  - Functions/Variables: `camelCase`.
- **Organization**:
  - Components: React components in `components/`.
  - Hooks: Custom hooks in `hooks/`.
  - Types: Global types in `types/`.

### Flutter-Specific Guidelines
- **Architecture**:
  - Follow Clean Architecture: Presentation, Domain, and Data layers.
  - **State Management**: Use `flutter_bloc` or `riverpod` (consistent within project).
- **Widgets**:
  - Break down large widgets into smaller, focused `StatelessWidget`s.
  - ALWAYS use `const` constructors where possible to optimize rebuilds.
- **Performance**:
  - Avoid expensive operations in `build()` methods.
  - Use `ListView.builder` for long or infinite lists.
- **Testing**:
  - Write Unit tests for implementations (Blocs, Repositories).
  - Write Widget tests for common UI components.

### Rust-Specific Guidelines
- **Naming**:
  - Crates/Modules/Functions/Variables: `snake_case`.
  - Types/Traits/Enums: `PascalCase`.
  - Constants: `SCREAMING_SNAKE_CASE`.
- **Organization**:
  - Binary entry point: `src/main.rs` (minimal logic).
  - Core logic: `src/lib.rs` (testable, reusable).
  - CLI: Use `clap`.
- **Error Handling**: Use `anyhow` for apps, `thiserror` for libs. Avoid `unwrap()` in production.
