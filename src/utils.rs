use ratatui::{style::Style, text::Span};
use std::ops::Range;

pub(crate) fn style_spans(
    mut spans: Vec<Span>,
    ranges: impl Iterator<Item = Range<usize>>,
    style: Style,
) -> Vec<Span> {
    for range in ranges {
        spans = style_spans_single(spans, range.clone(), style);
    }
    spans
}

fn style_spans_single(spans: Vec<Span>, mut range: Range<usize>, style: Style) -> Vec<Span> {
    let mut result = Vec::new();
    for span in spans {
        let len = span.content.len();
        let (a, b, c) = split_span(&span, &range);
        for snippet in [a, b.patch_style(style), c] {
            if !snippet.content.is_empty() {
                result.push(snippet.clone());
            }
        }
        range.start = range.start.saturating_sub(len);
        range.end = range.end.saturating_sub(len);
    }
    result
}

fn split_span(s: &Span, r: &Range<usize>) -> (Span<'static>, Span<'static>, Span<'static>) {
    let start = r.start.min(s.content.len());
    let end = r.end.min(s.content.len());
    (
        Span::styled(s.content[..start].to_string(), s.style),
        Span::styled(s.content[start..end].to_string(), s.style),
        Span::styled(s.content[end..].to_string(), s.style),
    )
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use ratatui::style::Stylize;

    #[test]
    fn style_spans_works() {
        let cases = vec![
            (vec!["foo"], (0..3), vec!["foo".bold()]),
            (
                vec!["foo"],
                (1..2),
                vec!["f".into(), "o".bold(), "o".into()],
            ),
            (vec!["foo"], (3..7), vec!["foo".into()]),
            (vec!["foo"], (0..42), vec!["foo".bold()]),
            (vec!["foo"], (0..0), vec!["foo".into()]),
            (vec![], (1..2), vec![]),
            (vec!["foo", "bar"], (3..6), vec!["foo".into(), "bar".bold()]),
            (
                vec!["foo", "bar"],
                (1..5),
                vec!["f".into(), "oo".bold(), "ba".bold(), "r".into()],
            ),
            (
                vec!["foo", "bar", "baz"],
                (2..7),
                vec![
                    "fo".into(),
                    "o".bold(),
                    "bar".bold(),
                    "b".bold(),
                    "az".into(),
                ],
            ),
            (vec!["foo"], (5..7), vec!["foo".into()]),
        ];
        for (spans, range, expected) in cases {
            let mut spans = spans.into_iter().map(Span::from).collect();
            spans = style_spans_single(spans, range, Style::default().bold());
            assert_eq!(spans, expected);
        }
    }

    #[test]
    fn style_spans_maintains_existing_styles() {
        let mut spans = vec![Span::from("foo").underlined()];
        spans = style_spans_single(spans, 0..3, Style::default().bold());
        assert_eq!(spans, vec![Span::from("foo").underlined().bold()]);
    }
}

#[cfg(test)]
pub(crate) mod test_utils {
    pub(crate) fn render_number(n: usize) -> &'static str {
        match n {
            1 => "one",
            2 => "two",
            3 => "three",
            4 => "four",
            5 => "five",
            6 => "six",
            7 => "seven",
            _ => "some-process",
        }
    }

    pub(crate) fn underline(s: &str) -> String {
        let mut result = String::new();
        for char in s.chars() {
            result.push(char);
            result.push('\u{35f}');
        }
        result
    }
}
