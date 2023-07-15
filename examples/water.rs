use recipemd::{Recipe, Result};

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

fn main() -> Result<()> {
    let recipe = Recipe::parse(MARKDOWN)?;
    println!("{recipe:#?}");
    Ok(())
}
