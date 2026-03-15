// SPDX-License-Identifier: Apache-2.0

pub(crate) use crate::support::effects::sqlite::{
    apply_readonly_pragmas, open_readonly_no_mutex,
};

pub const SCHEMA_V4_SQL: &str = include_str!("../../sql/schema_v4.sql");
