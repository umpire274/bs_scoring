//! Engine command types and their text-notation parser.
//!
//! These are the internal commands produced by parsing user scoring input
//! (pitches, contacts, defensive plays, …) and consumed by the engine reducer.
//! They are distinct from the user-facing CLI "commands" under `crate::cli`.

pub mod parser;
pub mod types;
