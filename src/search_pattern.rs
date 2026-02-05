use regex::Regex;

#[derive(Debug)]
pub(crate) enum SearchPattern {
    Empty,
    Regex { regex: regex::Regex },
    Invalid { regex: String },
}

impl SearchPattern {
    pub(crate) fn empty() -> SearchPattern {
        SearchPattern::from_string("")
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

    pub(crate) fn is_match(&self, s: &str) -> bool {
        match self {
            SearchPattern::Empty => true,
            SearchPattern::Regex { regex } => regex.is_match(s),
            SearchPattern::Invalid { .. } => false,
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
        *self = match regex::Regex::new(&regex) {
            Ok(regex) => SearchPattern::Regex { regex },
            Err(_) => SearchPattern::Invalid { regex },
        }
    }
}
