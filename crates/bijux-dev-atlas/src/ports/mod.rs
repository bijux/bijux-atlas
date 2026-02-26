//! `ports` defines IO boundaries consumed by `core`.
//!
//! Boundary: `core` depends on `ports`; `adapters` implement these interfaces.
//!
//! During crate convergence, the canonical trait definitions still live in
//! `crate::core::ports`. This module re-exports that surface so call sites can
//! migrate to `crate::ports::*` incrementally before the definitions are moved.

pub use crate::core::ports::*;
