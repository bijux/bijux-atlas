// SPDX-License-Identifier: Apache-2.0
//! Binary entry wiring for `bijux-dev-atlas`.
//!
//! The existing binary crate still owns the concrete module graph while the app layer is being
//! carved out. `app::run()` is the stable top-level entrypoint so `main.rs` stays a single-line
//! process wrapper.

pub(crate) fn run() -> i32 {
    crate::run()
}
