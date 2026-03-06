use std::io::{self, Write};

use bijux_atlas_client as _;
use reqwest as _;
use serde as _;
use serde_json as _;

fn main() -> io::Result<()> {
    let mut stdout = io::stdout().lock();
    stdout.write_all(
        format!(
            "{name} {version}\nLibrary support crate. Use `cargo add {name}` in Rust projects.\nRuntime CLIs: `bijux-atlas`, `atlas-server`, `bijux-dev-atlas`.\n",
            name = env!("CARGO_PKG_NAME"),
            version = env!("CARGO_PKG_VERSION")
        )
        .as_bytes(),
    )
}
