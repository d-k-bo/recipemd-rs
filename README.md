# recipemd-rs

[![Build Status](https://github.com/d-k-bo/recipemd-rs/workflows/CI/badge.svg)](https://github.com/d-k-bo/recipemd-rs/actions?query=workflow%3ACI)
[![Crates.io](https://img.shields.io/crates/v/recipemd)](https://lib.rs/crates/recipemd)
[![Documentation](https://img.shields.io/docsrs/recipemd)](https://docs.rs/recipemd)
[![License: LGPL-3.0-or-later](https://img.shields.io/crates/l/recipemd)](LICENSE)

<!-- cargo-rdme start -->

A library for parsing recipes written in markdown that follows the
[RecipeMD](https://recipemd.org/) specification.

## Example

```rust
const MARKDOWN: &str = r#"# Water

A refreshing drink that should be consumed several times a day.

*drink, non-alcoholic, H2O*

**1 glass**

---

- *1* glass
- *1* faucet

---

Turn on the faucet and fill the glass.
"#;

let recipe = Recipe::parse(MARKDOWN)?;
println!("{recipe:#?}");
```
<details><summary>Result of the above program</summary>


```rust
```

(If it doesn't show up, visit the [docs](https://docs.rs/recipemd#example) instead)

</details>

<!-- cargo-rdme end -->

## License

This project is licensed under the GNU Lesser General Public License version 3 or (at your option) any later version (LGPL-3.0-or-later).
