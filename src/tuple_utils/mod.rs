//! Tuple utilities for type-level and value-level manipulation of Rust tuples.
//!
//! This module provides traits, type aliases, and macros for generic operations on tuples,
//! including splitting, concatenation, extraction by type-level index, conversion to tuples of references,
//! and more. These utilities enable advanced, type-safe manipulation of tuples for use in strongly-typed
//! data structures and algorithms throughout the codebase.
//!
//! All public API is re-exported at the top level of this module.

pub mod core;
pub mod first_last;
pub mod prepend_append;
pub mod as_refs;
pub mod extract;
pub mod concat;
pub mod aliases;

pub use core::*;
pub use first_last::*;
pub use prepend_append::*;
pub use as_refs::*;
pub use extract::*;
pub use concat::*;
pub use aliases::*; 