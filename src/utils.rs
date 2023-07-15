// Copyright (c) 2023 d-k-bo
// SPDX-License-Identifier: LGPL-3.0-or-later

use std::{fmt::Debug, str::FromStr};

pub(crate) trait FromStrParseExpect {
    fn parse_expect<T>(&self) -> T
    where
        <T as std::str::FromStr>::Err: Debug,
        T: FromStr;
}
impl FromStrParseExpect for str {
    fn parse_expect<T>(&self) -> T
    where
        <T as std::str::FromStr>::Err: Debug,
        T: FromStr,
    {
        self.parse().expect("failed to parse string")
    }
}

pub(crate) fn escape_url(s: impl AsRef<str>) -> String {
    let mut buf = String::new();
    pulldown_cmark::escape::escape_href(&mut buf, s.as_ref())
        .expect("writing to a string never fails");
    buf
}

pub(crate) trait TrimNewlines {
    fn trim_newlines(&self) -> &Self;
}
impl TrimNewlines for str {
    fn trim_newlines(&self) -> &Self {
        self.trim_matches(|c| c == '\n' || c == '\r')
    }
}

pub(crate) fn decode_unicode_fraction(s: &str) -> (u16, u16) {
    match s {
        "\u{00bc}" => (1, 4),
        "\u{00bd}" => (1, 2),
        "\u{00be}" => (3, 4),
        "\u{2150}" => (1, 7),
        "\u{2151}" => (1, 9),
        "\u{2152}" => (1, 10),
        "\u{2153}" => (1, 3),
        "\u{2154}" => (2, 3),
        "\u{2155}" => (1, 5),
        "\u{2156}" => (2, 5),
        "\u{2157}" => (3, 5),
        "\u{2158}" => (4, 5),
        "\u{2159}" => (1, 6),
        "\u{215a}" => (5, 6),
        "\u{215b}" => (1, 8),
        "\u{215c}" => (3, 8),
        "\u{215d}" => (5, 8),
        "\u{215e}" => (7, 8),
        "\u{2189}" => (0, 3),
        _ => panic!("invalid unicode fraction: {s}"),
    }
}
