// SPDX-License-Identifier: Apache-2.0

use bijux_atlas_core::ResultExt;

#[test]
fn result_ext_attaches_static_context_without_allocation() {
    let r: Result<(), &str> = Err("bad");
    let err = r.with_context("parse manifest").expect_err("must error");
    assert_eq!(err.context, "parse manifest");
    assert_eq!(err.source, "bad");
}
