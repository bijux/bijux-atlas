// SPDX-License-Identifier: Apache-2.0

use super::super::Cli;
use clap::CommandFactory;
use clap_complete::{generate, Generator};

pub(crate) fn print_completion<G: Generator>(generator: G) {
    let mut command = Cli::command();
    let name = command.get_name().to_string();
    generate(generator, &mut command, name, &mut std::io::stdout());
}
