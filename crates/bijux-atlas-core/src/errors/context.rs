#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ErrorContext<E> {
    pub context: &'static str,
    pub source: E,
}

impl<E> ErrorContext<E> {
    #[must_use]
    pub const fn new(context: &'static str, source: E) -> Self {
        Self { context, source }
    }
}

pub trait ResultExt<T, E> {
    fn with_context(self, context: &'static str) -> Result<T, ErrorContext<E>>;
}

impl<T, E> ResultExt<T, E> for Result<T, E> {
    fn with_context(self, context: &'static str) -> Result<T, ErrorContext<E>> {
        self.map_err(|source| ErrorContext::new(context, source))
    }
}
