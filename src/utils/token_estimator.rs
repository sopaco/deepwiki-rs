use serde::{Deserialize, Serialize};

/// Token estimator for estimating the number of tokens in text
pub struct TokenEstimator {
    /// Token calculation rules for different models
    model_rules: TokenCalculationRules,
}

/// Token calculation rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenCalculationRules {
    /// Average token ratio for English characters (characters/token)
    pub english_char_per_token: f64,
    /// Average token ratio for Chinese characters
    pub chinese_char_per_token: f64,
    /// Base token overhead (system prompt, etc.)
    pub base_token_overhead: usize,
}

impl Default for TokenCalculationRules {
    fn default() -> Self {
        Self {
            // Based on empirical values from GPT series models
            english_char_per_token: 4.0,
            chinese_char_per_token: 1.5,
            base_token_overhead: 50,
        }
    }
}

/// Token estimation result
#[derive(Debug, Clone)]
pub struct TokenEstimation {
    /// Estimated number of tokens
    pub estimated_tokens: usize,
    /// Number of characters in text
    #[allow(dead_code)]
    pub character_count: usize,
    /// Number of Chinese characters
    #[allow(dead_code)]
    pub chinese_char_count: usize,
    /// Number of English characters
    #[allow(dead_code)]
    pub english_char_count: usize,
}

impl TokenEstimator {
    pub fn new() -> Self {
        Self {
            model_rules: TokenCalculationRules::default(),
        }
    }

    /// Estimate the number of tokens in text
    pub fn estimate_tokens(&self, text: &str) -> TokenEstimation {
        let character_count = text.chars().count();
        let chinese_char_count = self.count_chinese_chars(text);
        let english_char_count = self.count_english_chars(text);
        let other_char_count = character_count - chinese_char_count - english_char_count;

        // Calculate token count for each part
        let chinese_tokens =
            (chinese_char_count as f64 / self.model_rules.chinese_char_per_token).ceil() as usize;
        let english_tokens =
            (english_char_count as f64 / self.model_rules.english_char_per_token).ceil() as usize;
        // Calculate other characters using English rules
        let other_tokens = if other_char_count > 0 {
            (other_char_count as f64 / self.model_rules.english_char_per_token).ceil() as usize
        } else {
            0
        };

        let estimated_tokens =
            chinese_tokens + english_tokens + other_tokens + self.model_rules.base_token_overhead;

        TokenEstimation {
            estimated_tokens,
            character_count,
            chinese_char_count,
            english_char_count,
        }
    }

    /// Count number of Chinese characters
    fn count_chinese_chars(&self, text: &str) -> usize {
        text.chars().filter(|c| self.is_chinese_char(*c)).count()
    }

    /// Count number of English characters
    fn count_english_chars(&self, text: &str) -> usize {
        text.chars()
            .filter(|c| {
                c.is_ascii_alphabetic()
                    || c.is_ascii_whitespace()
                    || c.is_ascii_digit()
                    || c.is_ascii_punctuation()
            })
            .count()
    }

    /// Check if a character is Chinese
    fn is_chinese_char(&self, c: char) -> bool {
        matches!(c as u32,
            0x4E00..=0x9FFF |  // CJK Unified Ideographs
            0x3400..=0x4DBF |  // CJK Extension A
            0x20000..=0x2A6DF | // CJK Extension B
            0x2A700..=0x2B73F | // CJK Extension C
            0x2B740..=0x2B81F | // CJK Extension D
            0x2B820..=0x2CEAF | // CJK Extension E
            0x2CEB0..=0x2EBEF | // CJK Extension F
            0x30000..=0x3134F   // CJK Extension G
        )
    }
}
