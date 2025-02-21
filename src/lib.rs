// Copyright (c) 2023 d-k-bo
// SPDX-License-Identifier: LGPL-3.0-or-later

//! A library for parsing recipes written in markdown that follows the
//! [RecipeMD](https://recipemd.org/) specification.
//!
//! # Example
//!
//! ```
//! # use recipemd::{Recipe, Result};
//! const MARKDOWN: &str = r#"# Water
//!
//! A refreshing drink that should be consumed several times a day.
//!
//! *drink, non-alcoholic, H2O*
//!
//! **1 glass**
//!
//! ---
//!
//! - *1* glass
//! - *1* faucet
//!
//! ---
//!
//! Turn on the faucet and fill the glass.
//! "#;
//!
//! # fn main() -> Result<()> {
//! let recipe = Recipe::parse(MARKDOWN)?;
//! println!("{recipe:#?}");
//! # Ok(()) }
//! ```
//! <details><summary>Result of the above program</summary>
//!
//!
//! ```ignore
#![doc = include_str!("../examples/water.txt")]
//! ```
//!
//! (If it doesn't show up, visit the [docs](https://docs.rs/recipemd#example) instead)
//!
//! </details>

#![cfg_attr(docsrs, feature(doc_auto_cfg))]

mod ast;
mod error;
mod models;
mod parser;
mod utils;

use std::str::FromStr;

#[doc(inline)]
pub use error::*;
#[doc(inline)]
pub use models::*;
use parser::RecipeParser;

impl Recipe {
    /// Parse a recipe from a markdown string.
    #[cfg(feature = "diagnostics")]
    pub fn parse(src: &str) -> Result<Self> {
        RecipeParser::new(src)
            .parse_recipe()
            .map_err(|e| e.with_src(src.to_owned()))
    }
    #[cfg(not(feature = "diagnostics"))]
    pub fn parse(src: &str) -> Result<Self> {
        RecipeParser::new(src).parse_recipe()
    }
}

impl FromStr for Recipe {
    type Err = Error;

    fn from_str(src: &str) -> Result<Self> {
        Recipe::parse(src)
    }
}
