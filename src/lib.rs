#![allow(
    dead_code,
    irrefutable_let_patterns,
    unused_variables,
    unused_imports,
    unused_import_braces
)]
#![cfg_attr(feature = "ffi", deny(improper_ctypes_definitions))]
pub mod error;
pub mod fluid_pairings;
pub mod operations;
pub mod player;
pub mod player_registry;
pub mod round;
pub mod round_registry;
pub mod scoring;
pub mod settings;
pub mod standard_scoring;
pub mod swiss_pairings;
pub mod tournament;
pub mod tournament_manager;
#[cfg(feature = "ffi")]
pub mod ffi;
