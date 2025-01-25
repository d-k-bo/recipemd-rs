# recipemd-rs

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
