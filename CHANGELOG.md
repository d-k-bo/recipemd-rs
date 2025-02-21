# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- Adapt to changes in the [RecipeMD specification version 2.4.0](https://recipemd.org/specification.html#version-2-4-0-2024-02-14). `recipemd-rs` now follows the reference implementation more closely
- **BREAKING**: Change `Recipe` fields:
  - `tags` and `yield`: change type from `Option<Vec<_>>` to `Vec<_>`
  - `ingredients`: split into `ingredients` for top-level ingredients and `ingredient_groups`
- **BREAKING**: Change `IngredientGroup` fields:
  - `title`: change type from `Option<String>` to `String`
  - `ingredient_groups`: add nested ingredient groups
- **BREAKING**: Change `Amount` fields:
  - `factor`: change type from `Option<Factor>` to `Factor`
- **BREAKING**: Change `ErrorKind` variants:
  - Remove `ErrorKind::EmptyIngredientGroup`
  - Add `ErrorKind::AmountWithoutValue`

## [0.1.0] - 2023-07-31

[Unreleased]: https://github.com/d-k-bo/recipemd-rs/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/d-k-bo/recipemd-rs/releases/tag/v0.1.0
