# Rust Frag Submodules

## Status
Accepted

## Context
Several backend Rust modules split hand-written code with `include!("frag_*.rs")`. This makes module boundaries implicit, weakens navigation, and conflicts with the repo guidance to use true Rust submodules with explicit internal APIs.

## Decision
Replace every hand-written `frag_*.rs` include under `backend/src/` with named Rust submodules. Keep behavior and external module paths stable by using explicit `mod` declarations plus selective `pub use` re-exports where needed.

Target areas:
- `backend/src/services/chat`
- `backend/src/services/chat/tools`
- `backend/src/services/search`
- `backend/src/db/videos`
- `backend/src/handlers/search`
- `backend/src/handlers/content`

## Consequences
Module responsibilities become named and navigable.
Refactors require explicit imports and re-exports, which may surface hidden coupling during compilation.
