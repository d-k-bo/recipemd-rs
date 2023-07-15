// Copyright (c) 2023 d-k-bo
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::ops::Range;

/// Type alias for `Result<T, recipemd::Error>`.
pub type Result<T> = core::result::Result<T, Error>;

/// The exact reason why parsing of the recipe failed.
#[derive(Debug, PartialEq, thiserror::Error)]
pub enum ErrorKind {
    #[error("expected a first level heading as title")]
    ExpectedTitle,
    #[error("expected a horizontal line")]
    ExpectedHorizontalLine,
    #[error("found multiple tags sections")]
    MultipleTagsSections,
    #[error("found multiple yields sections")]
    MultipleYieldsSections,
    #[error("found description sections that are split by tags or yields section(s)")]
    MultipleDescriptionSections,
    #[error("ingredient is missing a name")]
    EmptyIngredient,
    #[error("ingredient group is empty")]
    EmptyIngredientGroup,
}

/// Returned if a parsing a recipe was not successful.
///
/// The exact byte range where the error occured can be retrieved from it's `span` field.
/// It is set to `None` if the error occured at the end of the file.
#[derive(Debug, thiserror::Error)]
#[error("failed to parse recipe")]
pub struct Error {
    #[source]
    pub kind: ErrorKind,
    pub span: Option<Range<usize>>,
    #[cfg(feature = "diagnostics")]
    src: Option<String>,
}

impl Error {
    /// Returns the markdown string of the recipe that could not be parsed.
    #[cfg(feature = "diagnostics")]
    pub fn src(&self) -> &str {
        self.src
            .as_deref()
            .expect("recipe source was not attached to the error")
    }
}

impl Error {
    pub(crate) fn new(kind: ErrorKind, span: impl Into<Option<Range<usize>>>) -> Self {
        Self {
            kind,
            span: span.into(),
            #[cfg(feature = "diagnostics")]
            src: None,
        }
    }
    #[cfg(feature = "diagnostics")]
    pub(crate) fn with_src(mut self, src: String) -> Self {
        self.src = Some(src);
        self
    }
}

#[cfg(feature = "miette")]
impl miette::Diagnostic for Error {
    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        self.src.as_ref().map(|x| x as _)
    }
    fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + '_>> {
        match &self.src {
            Some(src) => {
                let label = match &self.span {
                    Some(span) => miette::LabeledSpan::at(span.clone(), self.kind.to_string()),
                    None => miette::LabeledSpan::at_offset(src.len(), self.kind.to_string()),
                };
                Some(Box::new([label].into_iter()))
            }
            None => None,
        }
    }
}
