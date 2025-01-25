// Copyright (c) 2023 d-k-bo
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::{iter::Peekable, ops::Range};

use lazy_regex::regex;
use pulldown_cmark::{CowStr, HeadingLevel, OffsetIter, Parser};

use crate::{
    ast::{Node, NodeKind, NodeList},
    utils::{decode_unicode_fraction, escape_url, FromStrParseExpect, TrimNewlines},
    Amount, Error, ErrorKind, Factor, Ingredient, IngredientGroup, Recipe, Result,
};

pub(crate) struct RecipeParser<'s> {
    pub(crate) parser: Peekable<OffsetIter<'s>>,
    pub(crate) src: &'s str,
    pub(crate) pos: usize,
}

impl<'s> RecipeParser<'s> {
    pub(crate) fn new(src: &'s str) -> Self {
        let parser = Parser::new(src).into_offset_iter().peekable();

        Self {
            parser,
            src,
            pos: 0,
        }
    }
}

impl RecipeParser<'_> {
    pub(crate) fn parse_recipe(&mut self) -> Result<Recipe> {
        let title = self.parse_title()?;
        let DescriptionTagsYields {
            description,
            tags,
            yields,
        } = self.parse_description_tags_yields()?;

        let (ingredients, ingredient_groups) = self.parse_all_ingredients()?;

        let instructions = (self.pos < self.src.len())
            .then(|| self.src[self.pos..].trim_newlines())
            .and_then(|s| match s.is_empty() {
                true => None,
                false => Some(s.to_owned()),
            });

        Ok(Recipe {
            title,
            description,
            tags,
            yields,
            ingredients,
            ingredient_groups,
            instructions,
        })
    }
}

struct DescriptionTagsYields {
    description: Option<String>,
    tags: Vec<String>,
    yields: Vec<Amount>,
}

impl RecipeParser<'_> {
    fn parse_title(&mut self) -> Result<String> {
        match self.parse_node() {
            Some(Node {
                kind:
                    NodeKind::Heading {
                        level: HeadingLevel::H1,
                        children,
                    },
                ..
            }) => Ok(self.src[children.span()].to_owned()),
            Some(Node { span, .. }) => Err(Error::new(ErrorKind::ExpectedTitle, span)),
            None => Err(Error::new(ErrorKind::ExpectedTitle, None)),
        }
    }

    fn parse_description_tags_yields(&mut self) -> Result<DescriptionTagsYields> {
        let description_start = self.pos;

        enum DescriptionState {
            None,
            Started { end: usize },
            Final { end: usize },
        }

        let mut description_state = DescriptionState::None;
        let mut tags = None;
        let mut yields = None;

        loop {
            let Some(node) = self.parse_node() else {
                return Err(Error::new(ErrorKind::ExpectedHorizontalLine, None));
            };

            match node.kind {
                NodeKind::HorizontalLine => {
                    if let DescriptionState::Started { .. } = description_state {
                        description_state = DescriptionState::Final {
                            end: node.span.start,
                        }
                    }
                    break;
                }
                NodeKind::Paragraph(children) => {
                    match &children[..] {
                        // does the paragraph contain tags?
                        [Node {
                            kind: NodeKind::Emphasis(children),
                            span,
                        }] if children.len() == 1 => {
                            if tags.is_some() {
                                return Err(Error::new(
                                    ErrorKind::MultipleTagsSections,
                                    span.clone(),
                                ));
                            }
                            if let DescriptionState::Started { end } = description_state {
                                description_state = DescriptionState::Final { end }
                            }
                            let src = &self.src[children.span()];

                            tags = Some(
                                // https://regex101.com/r/1MmcHz/1
                                regex!(r"(?:[^,\d]*(?:\d+(?:,\d+)*)*[^,\d]*)*")
                                    .find_iter(src)
                                    .map(|m| m.as_str().trim().to_owned())
                                    .collect(),
                            );
                        }
                        // does the paragraph contain yields?
                        [Node {
                            kind: NodeKind::Strong(children),
                            span,
                        }] if children.len() == 1 => {
                            if yields.is_some() {
                                return Err(Error::new(
                                    ErrorKind::MultipleYieldsSections,
                                    span.clone(),
                                ));
                            }
                            if let DescriptionState::Started { end } = description_state {
                                description_state = DescriptionState::Final { end }
                            }
                            let span = children.span();

                            yields = Some(
                                // https://regex101.com/r/1MmcHz/1
                                regex!(r"(?:[^,\d]*(?:\d+(?:,\d+)*)*[^,\d]*)*")
                                    .find_iter(&self.src[span.clone()])
                                    .map(|m| {
                                        parse_amount(
                                            self.src,
                                            span.start + m.start()..span.start + m.end(),
                                        )
                                    })
                                    .collect::<Result<Vec<Amount>>>()?,
                            );
                        }
                        // the paragraph is part of the description
                        children => {
                            let span = children.span();

                            match description_state {
                                DescriptionState::None | DescriptionState::Started { .. } => {
                                    description_state = DescriptionState::Started { end: span.end }
                                }
                                DescriptionState::Final { .. } => {
                                    return Err(Error::new(
                                        ErrorKind::MultipleDescriptionSections,
                                        span,
                                    ))
                                }
                            }
                        }
                    }
                }
                _ => {
                    let span = node.span;

                    match description_state {
                        DescriptionState::None | DescriptionState::Started { .. } => {
                            description_state = DescriptionState::Started { end: span.end }
                        }
                        DescriptionState::Final { .. } => {
                            return Err(Error::new(ErrorKind::MultipleDescriptionSections, span))
                        }
                    }
                }
            }
        }

        let description = match description_state {
            DescriptionState::None => None,
            DescriptionState::Started { .. } => Some(self.src[description_start..].trim_newlines()),
            DescriptionState::Final { end } => {
                Some(self.src[description_start..end].trim_newlines())
            }
        }
        .and_then(|d| (!d.is_empty()).then_some(d))
        .map(ToOwned::to_owned);

        Ok(DescriptionTagsYields {
            description,
            tags: tags.unwrap_or_default(),
            yields: yields.unwrap_or_default(),
        })
    }

    fn parse_all_ingredients(&mut self) -> Result<(Vec<Ingredient>, Vec<IngredientGroup>)> {
        let mut ingredients = Vec::new();
        let mut ingredient_groups = Vec::new();

        let src = self.src;
        let mut nodes = std::iter::from_fn(|| self.parse_node()).peekable();

        while let Some(node) = nodes.next() {
            match node.kind {
                NodeKind::Heading { children, level } => {
                    ingredient_groups.push(parse_ingredient_group(
                        src,
                        src[children.span()].to_owned(),
                        level,
                        &mut nodes,
                    )?);
                }
                NodeKind::List(items) => {
                    ingredients.reserve(items.len());
                    for item in items {
                        ingredients.push(parse_ingredient(src, &item.flatten_paragraphs())?);
                    }
                }
                NodeKind::HorizontalLine => break,
                _ => return Err(Error::new(ErrorKind::ExpectedHorizontalLine, node.span)),
            }
        }

        Ok((ingredients, ingredient_groups))
    }
}

fn parse_amount(src: &str, span: Range<usize>) -> Result<Amount> {
    let s = src[span.clone()].trim();

    // proper (1/2) or improper fraction (1 1/2)
    if let Some(m) = regex!(
        r"^(?:(?P<whole>\d+)\s+)?(?P<numerator>\d+)\s*/\s*(?P<denominator>\d+)\s*(?P<unit>.+)?$"
    )
    .captures(s)
    {
        let whole: u16 = m
            .name("whole")
            .map(|m| m.as_str().parse_expect())
            .unwrap_or(0);
        let numerator: u16 = m["numerator"].parse_expect();
        let denominator: u16 = m["denominator"].parse_expect();
        let unit = m.name("unit").map(|m| m.as_str().to_owned());

        return Ok(Amount {
            factor: Factor::Fraction(whole * denominator + numerator, denominator),
            unit,
        });
    }

    // proper (½) or improper fraction with unicode vulgar fraction (1 ½)
    if let Some(m) =
        regex!(r"^(?:(?P<whole>\d+)\s+)?(?P<symbol>[\u00BC-\u00BE\u2150-\u215E])\s*(?P<unit>.+)?$")
            .captures(s)
    {
        let whole: u16 = m
            .name("whole")
            .map(|m| m.as_str().parse_expect())
            .unwrap_or(0);
        let (numerator, denominator) = decode_unicode_fraction(&m["symbol"]);
        let unit = m.name("unit").map(|m| m.as_str().to_owned());

        return Ok(Amount {
            factor: Factor::Fraction(whole * denominator + numerator, denominator),
            unit,
        });
    }

    // decimal (1.5 or 1,5)
    if let Some(m) = regex!(r"^(\d*)[.,](\d+)\s*(?P<unit>.+)?$").captures(s) {
        let value: f32 = format!("{}.{}", &m[1], &m[2]).parse_expect();
        let unit = m.name("unit").map(|m| m.as_str().to_owned());

        return Ok(Amount {
            factor: Factor::Float(value),
            unit,
        });
    }

    // integer (2)
    if let Some(m) = regex!(r"^(\d+)\s*(?P<unit>.+)?$").captures(s) {
        let value: u32 = m[1].parse_expect();
        let unit = m.name("unit").map(|m| m.as_str().to_owned());

        return Ok(Amount {
            factor: Factor::Integer(value),
            unit,
        });
    }

    Err(Error::new(ErrorKind::AmountWithoutValue, span))
}

fn parse_ingredient(src: &str, node: &Node) -> Result<Ingredient> {
    let children = match &node.kind {
        NodeKind::ListItem(children) | NodeKind::Paragraph(children) => children,
        _ => panic!("ingredient must be a list item or paragraph"),
    };

    match &children[..] {
        []
        | [Node {
            kind: NodeKind::Emphasis(_),
            ..
        }] => Err(Error::new(ErrorKind::EmptyIngredient, node.span.clone())),
        [paragraph @ Node {
            kind: NodeKind::Paragraph(_),
            ..
        }] => parse_ingredient(src, paragraph),
        [Node {
            kind: NodeKind::Emphasis(amount_children),
            ..
        }, Node {
            kind:
                NodeKind::Link {
                    destination,
                    children: link_children,
                },
            span,
        }]
        | [Node {
            kind: NodeKind::Emphasis(amount_children),
            ..
        }, Node {
            kind: NodeKind::Text(CowStr::Borrowed(" ")),
            ..
        }, Node {
            kind:
                NodeKind::Link {
                    destination,
                    children: link_children,
                },
            span,
        }] => {
            if link_children.is_empty() {
                return Err(Error::new(ErrorKind::EmptyIngredient, span.clone()));
            }

            let amount = Some(parse_amount(src, amount_children.span())?);
            Ok(Ingredient {
                amount,
                name: src[link_children.span()].trim().to_owned(),
                link: Some(escape_url(destination)),
            })
        }
        [Node {
            kind: NodeKind::Emphasis(amount_children),
            ..
        }, children @ ..] => {
            let amount = Some(parse_amount(src, amount_children.span())?);
            Ok(Ingredient {
                amount,
                name: src[children.span()].trim().to_string(),
                link: None,
            })
        }
        [Node {
            kind:
                NodeKind::Link {
                    destination,
                    children,
                },
            ..
        }] => Ok(Ingredient {
            amount: None,
            name: src[children.span()].trim().to_owned(),
            link: Some(escape_url(destination)),
        }),
        children => Ok(Ingredient {
            amount: None,
            name: src[children.span()].trim().to_owned(),
            link: None,
        }),
    }
}
fn parse_ingredient_group<'s>(
    src: &str,
    title: String,
    level: HeadingLevel,
    nodes: &mut Peekable<impl Iterator<Item = Node<'s>>>,
) -> Result<IngredientGroup> {
    let mut ingredients = Vec::new();
    let mut ingredient_groups = Vec::new();

    while let Some(node) = nodes.peek() {
        match &node.kind {
            NodeKind::Heading {
                level: child_level,
                children,
            } if child_level > &level => {
                let title = src[children.span()].to_owned();
                let child_level = *child_level;
                let _ = nodes.next();
                ingredient_groups.push(parse_ingredient_group(src, title, child_level, nodes)?);
            }
            NodeKind::List(items) => {
                ingredients.reserve(items.len());
                for item in items {
                    ingredients.push(parse_ingredient(src, &item.flatten_paragraphs())?);
                }
                let _ = nodes.next();
            }
            _ => break,
        }
    }

    Ok(IngredientGroup {
        title,
        ingredients,
        ingredient_groups,
    })
}
