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

    pub(crate) fn bold(s: &str) -> String {
        let mut result = String::new();
        for char in s.chars() {
            result.push(char);
            result.push('\u{333}');
        }
        return result;

        // OR:

        // let mut result = String::new();
        // for mut char in s.chars() {
        //     if 'A' <= char && char <= 'Z' {
        //         const BOLD_CAPITAL_A: u32 = 0x1D538;
        //         char = char::from_u32(BOLD_CAPITAL_A + char as u32 - 'A' as u32).unwrap();
        //     }
        //     if 'a' <= char && char <= 'z' {
        //         const BOLD_SMALL_A: u32 = 0x1D552;
        //         char = char::from_u32(BOLD_SMALL_A + char as u32 - 'a' as u32).unwrap();
        //     }
        //     result.push(char);
        //     result.push('\u{35f}');
        // }
        // return result;
    }
}
