#[cfg(test)]
pub(crate) mod test {
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
        return result;
    }
}
