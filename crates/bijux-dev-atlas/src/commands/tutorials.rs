// SPDX-License-Identifier: Apache-2.0

use crate::cli::{TutorialsCommand, TutorialsCommandArgs};
use crate::{emit_payload, resolve_repo_root};
use std::io::{self, Write};

pub(crate) fn run_tutorials_command(quiet: bool, command: TutorialsCommand) -> i32 {
    let run = match command {
        TutorialsCommand::List(args) => run_tutorials_action("list", &args),
        TutorialsCommand::Explain(args) => run_tutorials_action("explain", &args),
        TutorialsCommand::Verify(args) => run_tutorials_action("verify", &args),
    };
    match run {
        Ok((rendered, code)) => {
            if !quiet && !rendered.is_empty() {
                let _ = writeln!(io::stdout(), "{rendered}");
            }
            code
        }
        Err(err) => {
            let _ = writeln!(io::stderr(), "bijux-dev-atlas tutorials failed: {err}");
            1
        }
    }
}

fn run_tutorials_action(
    action: &'static str,
    args: &TutorialsCommandArgs,
) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(args.repo_root.clone())?;
    let payload = serde_json::json!({
        "schema_version": 1,
        "action": action,
        "domain": "tutorials",
        "text": format!("tutorials {action} surface registered"),
        "repo_root": repo_root.display().to_string()
    });
    let rendered = emit_payload(args.format, args.out.clone(), &payload)?;
    Ok((rendered, 0))
}
