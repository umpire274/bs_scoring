//! Engine command types and their text-notation parser.
//!
//! These are the internal commands produced by parsing user scoring input
//! (pitches, contacts, defensive plays, …) and consumed by the engine reducer.
//! They are distinct from the user-facing CLI "commands" under `crate::cli`.
//!
//! # Pipeline
//!
//! 1. `grammar` — stateless syntactic parsing: line → `Vec<Segment>`.
//! 2. `validator` — semantic checks against the `GameState`:
//!    `Vec<Segment> + &GameState` → `Vec<EngineCommand>`.
//! 3. `parser` — thin facade composing the two, exposing
//!    [`parser::parse_engine_commands`] as the single entry point for the
//!    game loop.

pub mod errors;
pub mod grammar;
pub mod parser;
pub mod types;
pub mod validator;
