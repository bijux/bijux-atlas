#![allow(unused_crate_dependencies)]

use std::io::{self, Write};

fn main() -> io::Result<()> {
    let mut stdout = io::stdout().lock();
    stdout.write_all(
        format!(
            "{name} {version}\nClient SDK module lives at `bijux_atlas::client`.\nRuntime CLIs: `bijux-atlas`, `atlas-server`, `bijux-dev-atlas`.\n",
            name = "bijux-atlas::client",
            version = env!("CARGO_PKG_VERSION")
        )
        .as_bytes(),
    )
}
