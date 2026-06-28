// SPDX-License-Identifier: Apache-2.0

use crate::emit_payload;
use crate::ops_commands::run_id_or_default;
use crate::resolve_repo_root;
use bijux_atlas_ops::diagnostics::commands::{
    diagnose_bundle_payload_for_run, diagnose_explain_payload_for_bundle,
    diagnose_redaction_payload_for_bundle, resolve_bundle_path,
};

pub(crate) fn run_ops_diagnose_bundle(
    args: &crate::cli::OpsDiagnoseBundleArgs,
) -> Result<(String, i32), String> {
    if !args.common.allow_write {
        return Err("diagnose bundle requires --allow-write".to_string());
    }
    let repo_root = resolve_repo_root(args.common.repo_root.clone())?;
    let run_id = run_id_or_default(args.common.run_id.clone())?;
    let payload =
        diagnose_bundle_payload_for_run(&repo_root, run_id.as_str(), args.scenario.as_deref())?;
    let rendered = emit_payload(args.common.format, args.common.out.clone(), &payload)?;
    Ok((rendered, 0))
}

pub(crate) fn run_ops_diagnose_explain(
    args: &crate::cli::OpsDiagnoseExplainArgs,
) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(args.common.repo_root.clone())?;
    let bundle_path = resolve_bundle_path(&repo_root, &args.bundle);
    let payload = diagnose_explain_payload_for_bundle(&repo_root, &bundle_path)?;
    let rendered = emit_payload(args.common.format, args.common.out.clone(), &payload)?;
    Ok((rendered, 0))
}

pub(crate) fn run_ops_diagnose_redact(
    args: &crate::cli::OpsDiagnoseRedactArgs,
) -> Result<(String, i32), String> {
    if !args.common.allow_write {
        return Err("diagnose redact requires --allow-write".to_string());
    }
    let repo_root = resolve_repo_root(args.common.repo_root.clone())?;
    let bundle_path = resolve_bundle_path(&repo_root, &args.bundle);
    let payload = diagnose_redaction_payload_for_bundle(&repo_root, &bundle_path)?;
    let rendered = emit_payload(args.common.format, args.common.out.clone(), &payload)?;
    Ok((rendered, 0))
}
