# Rust + Tokio Agent Rules

## Project Context
You are an expert Rust developer working with Tokio runtime.

## Code Style & Structure
### Rust Defaults
- Embrace ownership and borrowing. Prefer borrowing (`&T`, `&mut T`) over cloning unless necessary.
- Use `Result<T, E>` for fallible operations and `Option<T>` for optional values — propagate errors with `?` operator instead of `unwrap()`.
- Use `impl Trait` in function signatures for return types and `&dyn Trait` for dynamic dispatch — prefer generics over trait objects when possible.
- Use `?` operator for error propagation. Define custom error types with `thiserror` or implement `std::error::Error`.
- Prefer iterators and combinators (`.map()`, `.filter()`, `.collect()`) over manual loops.
- Use `clippy` lints and fix all warnings. Run `cargo fmt` before every commit.
- Use `#[derive(...)]` for common traits: `Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`.
- Prefer `&str` over `String` in function parameters; return `String` when ownership transfer is needed.

### Rust API Guidelines
- Use snake_case for functions, variables, and modules with descriptive names including auxiliary verbs (e.g., is_valid, has_error).
- Handle errors early using guard clauses, early returns, and the ? operator.
- Minimize allocations in hot paths; prefer zero-copy operations and static data where possible.
- Modularize code to avoid duplication, favoring iteration over repetition.
- Separate policy and metadata management from core storage for cleaner APIs.
- Prefer contiguous storage with index-based indirection over scattered pointers or dynamic structures.
- Design concurrency explicitly from the start (e.g., sharding or lock-free) rather than as an afterthought.
- Document all public items with `///` doc comments — include a `# Examples` section with a runnable `doctest` and `# Errors` / `# Panics` / `# Safety` sections where applicable.
- Implement structured logging with contextual fields for better observability.

## Linting & Formatting
### Rustfmt & Clippy
- Run `cargo fmt` before every commit. Configure in `rustfmt.toml` if needed.
- Run `cargo clippy` and fix all warnings — Clippy catches common mistakes and unidiomatic code.
- Run `cargo clippy -- -W clippy::all` for comprehensive linting and `cargo fmt` for formatting — add both to CI.
- Use `cargo clippy -- -D warnings` in CI to treat warnings as errors.
- Use `#[allow(clippy::lint_name)]` for intentional suppressions — always add a comment explaining why.
- Configure `rustfmt.toml` for team preferences: `max_width`, `use_field_init_shorthand`, `edition`.
- Run `cargo clippy --all-targets --all-features` to lint test code and feature-gated code too.

## Performance
### Performance Guidelines
- Profile before optimizing — measure, don't guess. Premature optimization wastes time and adds complexity.
- Optimize the critical path first. 90% of performance comes from 10% of the code.
- Cache expensive computations and database queries — use appropriate TTLs and invalidation strategies based on data freshness requirements.
- Cache expensive computations and API calls. Invalidate caches explicitly — stale data is a bug.
- Use lazy loading for non-critical resources and code paths.
- Debounce user-input-driven operations (search, resize, scroll).
- Prefer pagination or virtual scrolling for large data sets — never render 10,000 DOM nodes.

### Rust Performance
- Use `&str` and `&[T]` (borrowed slices) to avoid unnecessary cloning and allocation.
- Compile with `--release` for optimized builds. Debug builds are 10-100x slower.
- Use `cargo bench` with Criterion.rs for benchmarks — compare against baselines to detect regressions across commits.
- Use iterators and combinators instead of indexed loops — they often optimize to the same assembly.
- Use `Vec::with_capacity(n)` when the final size is known to avoid reallocation.
- Use `Cow<str>` for functions that sometimes need to allocate and sometimes can borrow.
- Use `rayon` for data parallelism: `.par_iter()` for parallel map/filter/reduce.

## Security
### Security Guidelines
- Validate and sanitize all user inputs from external sources.
- NEVER hardcode secrets (API keys, passwords) in the codebase. Use environment variables.
- Use parameterized queries for all database access — never concatenate user input into SQL, command strings, or template expressions.
- If you detect a hardcoded secret, stop immediately and prompt the user to remove it.
- Use parameterized queries or ORMs to prevent SQL injection.
- Ensure code handles edge cases and failures gracefully, not just the happy path.

## Testing
### Rust Testing
- Use `#[cfg(test)]` module in each source file for unit tests. Use `assert_eq!`, `assert_ne!`, `assert!` macros. Put integration tests in `tests/` directory — each file is a separate test binary.
- Use `#[should_panic(expected = "message")]` for testing error conditions. Use `Result<(), Box<dyn Error>>` as test return type for `?` operator in tests. Use `cargo test -- --nocapture` to see stdout. Organize test helpers in `tests/common/mod.rs`. Use `#[ignore]` for slow tests, run with `cargo test -- --ignored`.

### Cargo Test
- Run all tests with `cargo test`.
- Run specific tests with `cargo test -- test_name`.
- Place unit tests inline in source modules using `#[cfg(test)] mod tests { use super::*; #[test] fn test_name() { ... } }`.
- Place integration tests in the `tests/` directory.
- Unit tests next to code enable fast iteration and access to private items via `super::*`.
- Organize `tests/` into subdirectories like `integration/`, `models/`, `backend/` for categorization.
- Integration tests run as separate binaries, testing full workflows.
- Unit tests compile with the library for focused, efficient testing.

## Libraries & Tools
### Serde
- Derive `#[derive(Serialize, Deserialize)]` on structs — use `#[serde(rename_all = "camelCase")]` for JSON field naming conventions.
- Use `#[serde(skip_serializing_if = "Option::is_none")]` on `Option<T>` fields to omit null values from JSON output.
- Use `#[serde(default)]` on fields to use `Default::default()` when the field is missing during deserialization — makes APIs forward-compatible.
- Use `#[serde(skip_serializing_if = "Option::is_none")]` to omit null fields in output.
- Use `#[serde(default)]` for fields with sensible defaults during deserialization.
- Use `#[serde(deny_unknown_fields)]` on request types to reject unexpected input.
- Use `serde_json::from_str()` and `serde_json::to_string_pretty()` for JSON operations.

## Git & Workflow
### Conventional Commits
- Format commits strictly following Conventional Commits format: `type(scope): subject`.
- Keep the subject line under 50 characters, use the imperative mood, and do not end with a period.
- Adhere strictly to Conventional Commits format (e.g., `feat(auth): add google sign-in`, `fix(api): resolve memory leak`).
- Use appropriate semantic types: `feat`, `fix`, `chore`, `docs`, `style`, `refactor`, `test`, `perf`.
- Indicate breaking changes clearly by appending a `!` to the type/scope (e.g., `feat(api)!: drop v1 endpoints`) or in the footer as `BREAKING CHANGE:`.

### Branch Strategy
- Use descriptive branch names: `feature/`, `fix/`, `chore/`, `docs/` prefixes followed by a short slug.
- Keep branches short-lived — merge or rebase frequently to avoid drift from main.
- Use short-lived feature branches that merge within 1-3 days — long-lived branches cause merge conflicts and integration pain.
- Protect the main branch — require PR reviews and passing CI before merge.
- Delete branches after merging to keep the repository clean.
- Rebase feature branches on main before creating a PR to ensure a clean diff.
- Utilize the optional body section to explain the *what* and *why* of the change (not the *how*, which should be in code comments), wrapped at 72 characters.

## Agent Workflow
- Always plan before coding — break complex tasks into small, verifiable steps before writing code.
- After each step, verify the result works (run tests, check output) before moving to the next.
- State assumptions explicitly before starting — verify them with the user if uncertain.
- Write rules for yourself that prevent the same mistake from recurring.

## Context Management
### Memory Bank Pattern
- Maintain project context files that the agent reads at the start of every session.
- Keep an `activeContext.md` with current work focus, recent decisions, and open questions.
- Store project-specific decisions, architecture choices, and common patterns in a persistent memory file — update it when decisions change.
- Include `projectbrief.md` (goals, scope) and `techContext.md` (stack, architecture, conventions).
- Update context files after significant decisions or architectural changes — they are living documents.
- Store context files in a known location (`.cursor/rules/`, `.agent/`, project root) for easy discovery.
- Maintain lightweight `.ai_manifest.md` or `ARCHITECTURE.md` files that provide broad system context efficiently without requiring the agent to read the entire repository.

## Project Management
### Linear Integration
- Link every PR and commit to a Linear issue — use identifiers (e.g., `ENG-123`) in branch names.
- Update issue status as work progresses: In Progress → In Review → Done.
- Reference issue IDs in commit messages and PR titles (`Fixes AGE-123`) — Linear auto-links commits and transitions issue states.
- Create sub-issues for complex tasks — break epics into implementable chunks with clear acceptance criteria.
- Use labels consistently — categorize by type (bug, feature, chore) and area (frontend, backend, infra).
- Write issue descriptions with context, acceptance criteria, and technical approach when known.
- Order tasks by dependency — identify what must be done first and what can be parallelized.
- Each task should be completable in 1-4 hours — if a task takes longer, it needs further decomposition into smaller pieces.

## Documentation
### Minimal
- Write self-documenting code with clear names and small functions — code should explain WHAT and HOW without comments.
- Add comments only for non-obvious WHY decisions (business rules, workarounds, trade-offs) — delete comments that restate the code.
- Delete outdated comments immediately — a wrong comment is worse than no comment. Keep docs in sync with code or remove them.
- Document complex algorithms, business rules, and non-obvious performance trade-offs inline.
- Keep a brief README with setup instructions, architecture overview, and key decisions.
- Never comment out code — delete it. Version control is your history.
