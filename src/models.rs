// Copyright (c) 2023 d-k-bo
// SPDX-License-Identifier: LGPL-3.0-or-later

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A [Recipe](https://recipemd.org/specification.html#recipe) as defined by the RecipeMD specification.
///
/// See the [top-level documentation](crate) for details.
#[derive(Clone, Debug)]
#[cfg_attr(any(test, feature = "tests"), derive(PartialEq))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Recipe {
    pub title: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub yields: Vec<Amount>,
    pub ingredients: Vec<Ingredient>,
    pub ingredient_groups: Vec<IngredientGroup>,
    pub instructions: Option<String>,
}

/// An [IngredientGroup](https://recipemd.org/specification.html#ingredient-group).
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(any(test, feature = "tests"), derive(PartialEq))]
pub struct IngredientGroup {
    pub title: Option<String>,
    pub ingredients: Vec<Ingredient>,
    pub ingredient_groups: Vec<IngredientGroup>,
}

/// An [Ingredient](https://recipemd.org/specification.html#ingredient).
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(any(test, feature = "tests"), derive(PartialEq))]
pub struct Ingredient {
    pub amount: Option<Amount>,
    pub name: String,
    pub link: Option<String>,
}

/// An [Amount](https://recipemd.org/specification.html#amount) used for ingredients and yields.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(any(test, feature = "tests"), derive(PartialEq))]
pub struct Amount {
    pub factor: Factor,
    pub unit: Option<String>,
}

/// Represents the numerical part of an [`Amount`].
///
/// Integers are serialized as integers, fractions and floats are serialized as floats.
#[derive(Copy, Clone, Debug)]
pub enum Factor {
    Integer(u32),
    Fraction(u16, u16),
    Float(f32),
}

impl From<Factor> for f32 {
    fn from(value: Factor) -> Self {
        match value {
            Factor::Integer(v) => v as f32,
            Factor::Fraction(num, denom) => num as f32 / denom as f32,
            Factor::Float(v) => v,
        }
    }
}

#[cfg(feature = "serde")]
impl Serialize for Factor {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Factor::Integer(v) => v.to_string().serialize(serializer),
            Factor::Fraction(num, denom) => (*num as f32 / *denom as f32)
                .to_string()
                .serialize(serializer),
            Factor::Float(v) => v.to_string().serialize(serializer),
        }
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Factor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use std::str::FromStr;

        let s = <&str>::deserialize(deserializer)?;
        u32::from_str(s)
            .map(Factor::Integer)
            .or_else(|_| f32::from_str(s).map(Factor::Float))
            .map_err(serde::de::Error::custom)
    }
}

#[cfg(any(test, feature = "tests"))]
impl PartialEq for Factor {
    fn eq(&self, other: &Self) -> bool {
        f32::from(*self).eq(&f32::from(*other))
    }
}
