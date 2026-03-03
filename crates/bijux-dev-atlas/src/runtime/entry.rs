// SPDX-License-Identifier: Apache-2.0
//! Shared runtime-entry helpers for registry-backed runnable selection.

use crate::model::{RunnableId, RunnableSelection, SuiteId, Tag};

pub fn build_runnable_selection(
    suite: Option<&str>,
    group: Option<&str>,
    tag: Option<&str>,
    id: Option<&str>,
) -> Result<RunnableSelection, String> {
    Ok(RunnableSelection {
        suite: suite.map(SuiteId::parse).transpose()?,
        group: group.map(str::to_string),
        tag: tag.map(Tag::parse).transpose()?,
        id: id.map(RunnableId::parse).transpose()?,
    })
}
