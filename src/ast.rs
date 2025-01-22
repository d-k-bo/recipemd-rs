// Copyright (c) 2023 d-k-bo
// SPDX-License-Identifier: LGPL-3.0-or-later

//! Build a simplified abstract syntax tree from [`pulldown_cmark::Event`]s.

use std::ops::Range;

use pulldown_cmark::{CowStr, Event, HeadingLevel, Tag};

use crate::parser::RecipeParser;

/// Represents a node in the abstract syntax tree.
#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub(crate) struct Node<'s> {
    pub kind: NodeKind<'s>,
    pub span: Range<usize>,
}

impl Node<'_> {
    /// Recursively replaces all paragraphs its children
    pub(crate) fn flatten_paragraphs(&mut self) -> &mut Self {
        let children = match &mut self.kind {
            NodeKind::Heading { children, .. }
            | NodeKind::Paragraph(children)
            | NodeKind::Emphasis(children)
            | NodeKind::Strong(children)
            | NodeKind::List(children)
            | NodeKind::ListItem(children)
            | NodeKind::Link { children, .. } => children,
            _ => return self,
        };

        for mut child in std::mem::take(children) {
            match child.kind {
                NodeKind::Paragraph(mut p_children) => {
                    for p_child in p_children.iter_mut() {
                        p_child.flatten_paragraphs();
                    }
                    children.extend(p_children);
                }
                _ => {
                    child.flatten_paragraphs();
                    children.push(child);
                }
            }
        }

        self
    }
}

/// The simplified type of node if it is relevant for parsing recipe.
#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub(crate) enum NodeKind<'s> {
    Heading {
        level: HeadingLevel,
        children: Vec<Node<'s>>,
    },
    Paragraph(Vec<Node<'s>>),
    Emphasis(Vec<Node<'s>>),
    Strong(Vec<Node<'s>>),
    List(Vec<Node<'s>>),
    ListItem(Vec<Node<'s>>),
    HorizontalLine,
    Text(CowStr<'s>),
    Link {
        destination: CowStr<'s>,
        children: Vec<Node<'s>>,
    },
    Other,
}

pub(crate) trait NodeList {
    fn span(&self) -> Range<usize>;
}
impl NodeList for [Node<'_>] {
    /// Returns the calculated total span of multiple adjacent nodes.
    ///
    ///
    /// # Panics
    ///
    /// Panics if self is empty.
    fn span(&self) -> Range<usize> {
        self.first().expect("node list is empty").span.start
            ..self.last().expect("node list is empty").span.end
    }
}

impl<'s> RecipeParser<'s> {
    /// Consume events of the underlying parser until a complete [`Node`] can be returned.
    /// Returns `None` when the parser reaches its end.
    pub fn parse_node(&mut self) -> Option<Node<'s>> {
        if let Event::End(_) = self.parser.peek()?.0 {
            return None;
        }

        let node = match self.parser.next()? {
            (Event::Start(tag), Range { start, .. }) => match tag {
                Tag::Heading { level, .. } => {
                    let (children, end) = self.parse_child_nodes();

                    Node {
                        kind: NodeKind::Heading { level, children },
                        span: start..end,
                    }
                }
                Tag::Paragraph => {
                    let (children, end) = self.parse_child_nodes();
                    Node {
                        kind: NodeKind::Paragraph(children),
                        span: start..end,
                    }
                }
                Tag::Emphasis => {
                    let (children, end) = self.parse_child_nodes();
                    Node {
                        kind: NodeKind::Emphasis(children),
                        span: start..end,
                    }
                }
                Tag::Strong => {
                    let (children, end) = self.parse_child_nodes();
                    Node {
                        kind: NodeKind::Strong(children),
                        span: start..end,
                    }
                }
                Tag::List(_) => {
                    let (children, end) = self.parse_child_nodes();
                    Node {
                        kind: NodeKind::List(children),
                        span: start..end,
                    }
                }
                Tag::Item => {
                    let (children, end) = self.parse_child_nodes();
                    Node {
                        kind: NodeKind::ListItem(children),
                        span: start..end,
                    }
                }
                Tag::Link {
                    dest_url: destination,
                    ..
                } => {
                    let (children, end) = self.parse_child_nodes();

                    Node {
                        kind: NodeKind::Link {
                            destination,
                            children,
                        },
                        span: start..end,
                    }
                }
                _ => {
                    let (_, end) = self.parse_child_nodes();
                    Node {
                        kind: NodeKind::Other,
                        span: start..end,
                    }
                }
            },
            (Event::Rule, span) => Node {
                kind: NodeKind::HorizontalLine,
                span,
            },
            (Event::Text(text), span) => Node {
                kind: NodeKind::Text(text),
                span,
            },
            (_, span) => Node {
                kind: NodeKind::Other,
                span,
            },
        };

        self.pos = node.span.end;

        Some(node)
    }
    fn parse_child_nodes(&mut self) -> (Vec<Node<'s>>, usize) {
        let children = std::iter::from_fn(|| self.parse_node()).collect();

        match self.parser.next() {
            Some((Event::End(_), Range { end, .. })) => (children, end),
            _ => panic!("expected an Event::End(_)"),
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn parse_heading() {
        assert_eq!(
            RecipeParser::new("# A recipe title\n").parse_node(),
            Some(Node {
                kind: NodeKind::Heading {
                    level: HeadingLevel::H1,
                    children: vec![Node {
                        kind: NodeKind::Text("A recipe title".into()),
                        span: 2..16
                    }]
                },
                span: 0..17
            })
        );
    }

    #[test]
    fn parse_paragraph() {
        assert_eq!(
            RecipeParser::new("Hello World!").parse_node(),
            Some(Node {
                kind: NodeKind::Paragraph(vec![Node {
                    kind: NodeKind::Text("Hello World!".into()),
                    span: 0..12
                }]),
                span: 0..12
            })
        );
    }

    #[test]
    fn parse_emphasis_strong() {
        assert_eq!(
            RecipeParser::new("*emphasis*").parse_node(),
            Some(Node {
                kind: NodeKind::Paragraph(vec![Node {
                    kind: NodeKind::Emphasis(vec![Node {
                        kind: NodeKind::Text("emphasis".into()),
                        span: 1..9
                    }]),
                    span: 0..10
                }]),
                span: 0..10
            })
        );
        assert_eq!(
            RecipeParser::new("**strong**").parse_node(),
            Some(Node {
                kind: NodeKind::Paragraph(vec![Node {
                    kind: NodeKind::Strong(vec![Node {
                        kind: NodeKind::Text("strong".into()),
                        span: 2..8
                    }]),
                    span: 0..10
                }]),
                span: 0..10
            })
        );
    }

    #[test]
    fn parse_list() {
        assert_eq!(
            RecipeParser::new("- first\n- second\n- third\n").parse_node(),
            Some(Node {
                kind: NodeKind::List(vec![
                    Node {
                        kind: NodeKind::ListItem(vec![Node {
                            kind: NodeKind::Text("first".into()),
                            span: 2..7
                        }]),
                        span: 0..8
                    },
                    Node {
                        kind: NodeKind::ListItem(vec![Node {
                            kind: NodeKind::Text("second".into()),
                            span: 10..16
                        }]),
                        span: 8..17
                    },
                    Node {
                        kind: NodeKind::ListItem(vec![Node {
                            kind: NodeKind::Text("third".into()),
                            span: 19..24
                        }]),
                        span: 17..25
                    }
                ]),
                span: 0..25
            })
        );
    }

    #[test]
    fn parse_horizontal_line() {
        assert_eq!(
            RecipeParser::new("---").parse_node(),
            Some(Node {
                kind: NodeKind::HorizontalLine,
                span: 0..3
            })
        );
    }

    #[test]
    fn parse_link() {
        assert_eq!(
            RecipeParser::new("[Example](https://example.org)").parse_node(),
            Some(Node {
                kind: NodeKind::Paragraph(vec![Node {
                    kind: NodeKind::Link {
                        children: vec![Node {
                            kind: NodeKind::Text("Example".into()),
                            span: 1..8
                        }],
                        destination: "https://example.org".into(),
                    },
                    span: 0..30
                }]),
                span: 0..30
            })
        );
    }
}
