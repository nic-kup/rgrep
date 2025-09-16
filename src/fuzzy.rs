pub fn fuzzy_match(pattern: &str, text: &str) -> Option<(String, u32)> {
    if pattern.is_empty() {
        return Some((text.to_string(), 0));
    }

    if text.is_empty() {
        return None;
    }

    let positions: Vec<usize> = text
        .chars()
        .enumerate()
        .filter_map(|(i, c)| {
            if c == pattern.chars().next().expect("Already checked empty!") {
                Some(i)
            } else {
                None
            }
        })
        .collect();

    let pattern_chars: Vec<char> = pattern.chars().collect();

    let mut best_score = u32::MAX;
    let mut optimal_result = String::new();

    for pos in positions {
        let mut first_match: bool = false;

        let mut result = String::new();
        result.push_str(&text[..pos]);

        let mut score: u32 = 0;
        let mut pattern_index = 0;

        for text_char in text[pos..].chars() {
            if pattern_chars.len() > pattern_index
                && text_char
                    .to_lowercase()
                    .eq(pattern_chars[pattern_index].to_lowercase())
            {
                result.push_str("\x1b[1;31m");
                result.push(text_char);
                result.push_str("\x1b[0m");

                first_match = true;
                pattern_index += 1;
            } else {
                result.push(text_char);
                if first_match && pattern_chars.len() > pattern_index {
                    score += 1;
                }
            }
        }

        if score < best_score && pattern_index == pattern_chars.len() {
            best_score = score;
            optimal_result = result;
        }
    }

    if best_score == u32::MAX {
        return None;
    }

    Some((optimal_result, best_score))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimal_matching() {
        let result = fuzzy_match("abc", "aXbXXcXXabcXX");
        assert!(result.is_some());
        let (_highlighted, score) = result.unwrap();
        assert_eq!(score, 0);
    }
}
