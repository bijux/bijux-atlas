// SPDX-License-Identifier: Apache-2.0

use crate::executor::ExecError;
use crate::parser::ParseError;
use crate::planner::PlanError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum QueryErrorCode {
    Validation,
    Cursor,
    Sql,
    Policy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryError {
    pub code: QueryErrorCode,
    pub message: String,
}

impl QueryError {
    #[must_use]
    pub fn new(code: QueryErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for QueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.code, self.message)
    }
}
impl std::error::Error for QueryError {}

impl From<ParseError> for QueryError {
    fn from(value: ParseError) -> Self {
        Self::new(QueryErrorCode::Validation, value.to_string())
    }
}

impl From<PlanError> for QueryError {
    fn from(value: PlanError) -> Self {
        Self::new(QueryErrorCode::Validation, value.to_string())
    }
}

impl From<ExecError> for QueryError {
    fn from(value: ExecError) -> Self {
        match value {
            ExecError::Cursor(msg) => Self::new(QueryErrorCode::Cursor, msg),
            ExecError::Sql(msg) => Self::new(QueryErrorCode::Sql, msg),
            ExecError::Policy(msg) => Self::new(QueryErrorCode::Policy, msg),
            ExecError::Validation(msg) => Self::new(QueryErrorCode::Validation, msg),
        }
    }
}
