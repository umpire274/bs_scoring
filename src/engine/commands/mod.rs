//! Engine command types and their text-notation parser.
//!
//! These are the internal commands produced by parsing user scoring input
//! (pitches, contacts, defensive plays, …) and consumed by the engine reducer.
//! They are distinct from the user-facing CLI "commands" under `crate::cli`.
//!
//! # Pipeline (WIP for v0.11.0-alpha2)
//!
//! 1. `grammar` — stateless syntactic parsing: line → `Vec<Segment>`.
//! 2. `validator` (TODO) — semantic checks against the `GameState`:
//!    `Vec<Segment> + &GameState` → `Vec<EngineCommand>`.
//! 3. `parser` (current) — legacy single-pass parser, still the active entry
//!    point from the game loop while the new pipeline is being assembled.

pub mod errors;
pub mod grammar;
pub mod parser;
pub mod types;
