use regex::Regex;
use std::{mem, ops::Range};

#[derive(Debug)]
pub(crate) enum SearchPattern {
    Empty,
    Regex {
        regex: regex::Regex,
        original: String,
    },
    Invalid {
        original: String,
    },
}

impl SearchPattern {
    pub(crate) fn empty() -> SearchPattern {
        SearchPattern::Empty
    }

    pub(crate) fn from_string(pattern: String) -> SearchPattern {
        if pattern.is_empty() {
            return SearchPattern::Empty;
        }
        match Regex::new(&pattern) {
            Ok(regex) => SearchPattern::Regex {
                regex,
                original: pattern,
            },
            Err(_) => SearchPattern::Invalid { original: pattern },
        }
    }

    pub(crate) fn find(&self, s: &str) -> Vec<Range<usize>> {
        match self {
            SearchPattern::Empty => Vec::new(),
            SearchPattern::Regex { regex, .. } => regex.find_iter(s).map(|m| m.range()).collect(),
            SearchPattern::Invalid { .. } => Vec::new(),
        }
    }

    pub(crate) fn as_str(&self) -> &str {
        match self {
            SearchPattern::Empty => "",
            SearchPattern::Regex { regex, .. } => regex.as_str(),
            SearchPattern::Invalid { original } => original.as_str(),
        }
    }

    pub(crate) fn into_string(self) -> String {
        match self {
            SearchPattern::Empty => String::new(),
            SearchPattern::Regex { original, .. } => original,
            SearchPattern::Invalid { original } => original,
        }
    }

    pub(crate) fn modify(&mut self, f: impl FnOnce(&mut String)) {
        let mut tmp = SearchPattern::Empty;
        mem::swap(&mut tmp, self);
        let mut pattern: String = tmp.into_string();
        f(&mut pattern);
        *self = SearchPattern::from_string(pattern);
    }
}
