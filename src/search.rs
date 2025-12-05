//! Fuzzy search with Sublime Text-style scoring.

/// Result of a fuzzy match.
#[derive(Debug, Clone)]
pub struct FuzzyMatch {
    /// Match score (higher is better).
    pub score: i32,
    /// Indices of matched characters in the target string.
    pub indices: Vec<usize>,
}

/// Performs fuzzy matching with Sublime Text-style scoring.
///
/// Returns `None` if the pattern doesn't match, or `Some(FuzzyMatch)` with
/// the score and matched character indices.
///
/// # Scoring
/// - Word boundary bonus: +10 (after _, -, space, or camelCase transition)
/// - Consecutive match bonus: +5
/// - Start of string bonus: +8
/// - Gap penalty: -1 per skipped character
pub fn fuzzy_match(pattern: &str, target: &str) -> Option<FuzzyMatch> {
    if pattern.is_empty() {
        return Some(FuzzyMatch {
            score: 0,
            indices: vec![],
        });
    }

    let pattern_lower: Vec<char> = pattern.to_lowercase().chars().collect();
    let target_chars: Vec<char> = target.chars().collect();
    let target_lower: Vec<char> = target.to_lowercase().chars().collect();

    let mut indices = Vec::with_capacity(pattern_lower.len());
    let mut score: i32 = 0;
    let mut pattern_idx = 0;
    let mut last_match_idx: Option<usize> = None;

    for (target_idx, &target_char) in target_lower.iter().enumerate() {
        if pattern_idx >= pattern_lower.len() {
            break;
        }

        if target_char == pattern_lower[pattern_idx] {
            indices.push(target_idx);

            // Start of string bonus
            if target_idx == 0 {
                score += 8;
            }

            // Word boundary bonus
            if is_word_boundary(&target_chars, target_idx) {
                score += 10;
            }

            // Consecutive match bonus
            if let Some(last_idx) = last_match_idx {
                if target_idx == last_idx + 1 {
                    score += 5;
                } else {
                    // Gap penalty
                    let gap = (target_idx - last_idx - 1) as i32;
                    score -= gap;
                }
            }

            last_match_idx = Some(target_idx);
            pattern_idx += 1;
        }
    }

    // All pattern characters must match
    if pattern_idx == pattern_lower.len() {
        // Base score for matching
        score += 10;
        Some(FuzzyMatch { score, indices })
    } else {
        None
    }
}

/// Checks if a position is a word boundary.
fn is_word_boundary(chars: &[char], idx: usize) -> bool {
    if idx == 0 {
        return true;
    }

    let prev = chars[idx - 1];
    let curr = chars[idx];

    // After separator characters
    if matches!(prev, '_' | '-' | ' ' | '/' | '\\' | '.') {
        return true;
    }

    // camelCase transition (lowercase followed by uppercase)
    if prev.is_lowercase() && curr.is_uppercase() {
        return true;
    }

    false
}

/// Filters and sorts commands by fuzzy match score.
///
/// Returns indices of matching commands sorted by score (best first).
pub fn filter_commands<Message>(
    query: &str,
    commands: &[crate::Command<Message>],
) -> Vec<(usize, FuzzyMatch)> {
    if query.is_empty() {
        // No query: return all commands in original order
        return commands
            .iter()
            .enumerate()
            .map(|(i, _)| {
                (
                    i,
                    FuzzyMatch {
                        score: 0,
                        indices: vec![],
                    },
                )
            })
            .collect();
    }

    let mut matches: Vec<(usize, FuzzyMatch)> = commands
        .iter()
        .enumerate()
        .filter_map(|(idx, cmd)| {
            // Match against name
            let name_match = fuzzy_match(query, &cmd.name);

            // Match against description
            let desc_match = cmd
                .description
                .as_ref()
                .and_then(|d| fuzzy_match(query, d));

            // Match against keywords
            let keyword_match = cmd
                .keywords
                .iter()
                .filter_map(|k| fuzzy_match(query, k))
                .max_by_key(|m| m.score);

            // Take best match
            let best = [name_match, desc_match, keyword_match]
                .into_iter()
                .flatten()
                .max_by_key(|m| m.score);

            best.map(|m| (idx, m))
        })
        .collect();

    // Sort by score (highest first)
    matches.sort_by(|a, b| b.1.score.cmp(&a.1.score));

    matches
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match() {
        let result = fuzzy_match("save", "Save File").unwrap();
        assert!(result.score > 0);
        assert_eq!(result.indices, vec![0, 1, 2, 3]);
    }

    #[test]
    fn test_fuzzy_match() {
        let result = fuzzy_match("sf", "Save File").unwrap();
        assert!(result.score > 0);
        assert_eq!(result.indices, vec![0, 5]); // S and F
    }

    #[test]
    fn test_no_match() {
        assert!(fuzzy_match("xyz", "Save File").is_none());
    }

    #[test]
    fn test_camel_case_boundary() {
        let result = fuzzy_match("gw", "getCurrentWindow").unwrap();
        // Should match 'g' at start and 'W' at word boundary
        assert!(result.indices.contains(&0)); // g
        assert!(result.indices.contains(&10)); // W
    }

    #[test]
    fn test_snake_case_boundary() {
        let result = fuzzy_match("sf", "save_file").unwrap();
        assert!(result.score > 0);
    }

    #[test]
    fn test_empty_pattern() {
        let result = fuzzy_match("", "anything").unwrap();
        assert_eq!(result.score, 0);
        assert!(result.indices.is_empty());
    }

    #[test]
    fn test_consecutive_bonus() {
        let consecutive = fuzzy_match("sav", "save").unwrap();
        let scattered = fuzzy_match("sae", "save file").unwrap();
        // Consecutive matches should score higher
        assert!(consecutive.score > scattered.score);
    }
}
