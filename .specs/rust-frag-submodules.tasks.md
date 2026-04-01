# Tasks: Rust Frag Submodules

## Current State
All backend `frag_*.rs` includes under `backend/src/` were replaced with named Rust submodules. Backend `cargo check`, `cargo test`, and `cargo audit` passed; Rust export tests also rewrote generated frontend bindings under `frontend/src/lib/bindings/`.

## Steps
- [x] Create spec and task tracking files
- [x] Replace `services/chat` fragments with named submodules
- [x] Replace `services/chat/tools` fragments with named submodules
- [x] Replace remaining backend `frag_*.rs` includes with named submodules
- [x] Run formatting and backend verification
