use regex::Regex;
use std::ops::Range;

#[derive(Debug)]
pub(crate) enum SearchPattern {
    Empty,
    Regex { regex: regex::Regex },
    Invalid { regex: String },
}

impl SearchPattern {
    pub(crate) fn empty() -> SearchPattern {
        SearchPattern::Empty
    }

    pub(crate) fn from_string(regex: &str) -> SearchPattern {
        if regex.is_empty() {
            return SearchPattern::Empty;
        }
        match Regex::new(regex) {
            Ok(regex) => SearchPattern::Regex { regex },
            Err(_) => SearchPattern::Invalid {
                regex: regex.to_string(),
            },
        }
    }

    pub(crate) fn find(&self, s: &str) -> Option<Range<usize>> {
        match self {
            SearchPattern::Empty => None,
            SearchPattern::Regex { regex } => regex.find(s).map(|m| m.range()),
            SearchPattern::Invalid { .. } => None,
        }
    }

    pub(crate) fn as_str(&self) -> &str {
        match self {
            SearchPattern::Empty => "",
            SearchPattern::Regex { regex } => regex.as_str(),
            SearchPattern::Invalid { regex } => regex.as_str(),
        }
    }

    pub(crate) fn modify(&mut self, f: impl FnOnce(&mut String)) {
        let mut regex: String = self.as_str().to_string();
        f(&mut regex);
        *self = if regex.is_empty() {
            SearchPattern::Empty
        } else {
            match regex::Regex::new(&regex) {
                Ok(regex) => SearchPattern::Regex { regex },
                Err(_) => SearchPattern::Invalid { regex },
            }
        }
    }
}
