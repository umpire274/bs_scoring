//! User-facing command-line interface.
//!
//! - `menu` defines the choices rendered in each interactive menu.
//! - `screens` contains the handler for each menu entry (new game, list games,
//!   play-ball, umpire supervisor, …).

pub mod menu;
pub mod screens;
