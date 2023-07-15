use std::path::PathBuf;

use miette::IntoDiagnostic;
use once_cell::sync::Lazy;
use recipemd::Recipe;

static TESTCASE_DIR: Lazy<PathBuf> = Lazy::new(|| {
    let testcase_dir = PathBuf::from("./recipemd/testcases");
    assert!(
        testcase_dir.exists(),
        "repo must be cloned with --recurse-submodules"
    );
    testcase_dir
});

mod valid {
    use super::*;

    use pretty_assertions::assert_eq;

    macro_rules! testcase {
        ( $name:ident $( , ignore $( = $reason:literal )? )? ) => {
            #[test]
            $(
                #[ignore $( = $reason )? ]
            )?
            fn $name() -> miette::Result<()> {
                valid(stringify!($name))
            }
        };
    }

    fn valid(name: &str) -> miette::Result<()> {
        let md =
            std::fs::read_to_string(TESTCASE_DIR.join(format!("{name}.md"))).into_diagnostic()?;
        let json =
            std::fs::read_to_string(TESTCASE_DIR.join(format!("{name}.json"))).into_diagnostic()?;

        let ours = Recipe::parse(&md)?;
        let reference: Recipe = serde_json::from_str(&json).into_diagnostic()?;

        assert_eq!(ours, reference, "recipes don't match (ours vs. reference)");

        Ok(())
    }

    testcase!(commonmark_fenced_code_blocks);
    testcase!(commonmark_reference_images);
    testcase!(commonmark_reference_links);
    testcase!(
        ingredients_groups,
        ignore = "not spec compliant: nested ingredient groups"
    );
    testcase!(
        ingredients_links,
        ignore = "unspecified: differences in parsing whitespace"
    );
    testcase!(
        ingredients_multiline,
        ignore = "unspecified: differences in parsing whitespace"
    );
    testcase!(ingredients_numbered);
    testcase!(ingredients_sublist);
    testcase!(ingredients);
    testcase!(
        instructions,
        ignore = "not spec compliant: empty ingredient group"
    );
    testcase!(
        recipe,
        ignore = "not spec compliant: nested ingredient groups"
    );
    testcase!(
        tags_no_partial,
        ignore = "not spec compliant: empty ingredient group"
    );
    testcase!(
        tags_splitting,
        ignore = "not spec compliant: empty ingredient group"
    );
    testcase!(
        tags_yields,
        ignore = "not spec compliant: empty ingredient group"
    );
    testcase!(tags, ignore = "not spec compliant: empty ingredient group");
    testcase!(
        title_setext,
        ignore = "not spec compliant: empty ingredient group"
    );
    testcase!(title, ignore = "not spec compliant: empty ingredient group");
    testcase!(
        yields_tags,
        ignore = "not spec compliant: empty ingredient group"
    );
    testcase!(
        yields,
        ignore = "not spec compliant: empty ingredient group"
    );
}

mod invalid {
    use miette::Context;
    use recipemd::ErrorKind;

    use super::*;

    macro_rules! testcase {
        ( $name:ident, $error_kind:expr $( , ignore $( = $reason:literal )? )? ) => {
            #[test]
            $(
                #[ignore $( = $reason )? ]
            )?
            fn $name() -> miette::Result<()> {
                invalid(stringify!($name), $error_kind)
            }
        };
    }

    fn invalid(name: &str, error_kind: ErrorKind) -> miette::Result<()> {
        let md = std::fs::read_to_string(TESTCASE_DIR.join(format!("{name}.invalid.md")))
            .into_diagnostic()?;

        match Recipe::parse(&md) {
            Ok(recipe) => Err(miette::miette!(
                "recipe is invalid but no error was generated\n{recipe:#?}"
            )),
            Err(e) => {
                if e.kind == error_kind {
                    Ok(())
                } else {
                    Err(e).wrap_err(format!(
                        "Expected error \"{error_kind}\" but got different error",
                    ))
                }
            }
        }
    }

    testcase!(empty, ErrorKind::ExpectedTitle);
    testcase!(ingredients_empty, ErrorKind::EmptyIngredient);
    testcase!(ingredients_no_divider, ErrorKind::ExpectedHorizontalLine);
    testcase!(ingredients_no_name, ErrorKind::EmptyIngredient);
    testcase!(instructions_no_divider, ErrorKind::ExpectedHorizontalLine);
    testcase!(tags_multiple, ErrorKind::MultipleTagsSections);
    testcase!(title_second_level_heading, ErrorKind::ExpectedTitle);
    testcase!(yields_multiple, ErrorKind::MultipleYieldsSections);
}
