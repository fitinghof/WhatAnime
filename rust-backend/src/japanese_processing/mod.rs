use fuzzywuzzy::fuzz;
use kakasi::{self, IsJapanese};
use regex::Regex;
use std::collections::HashSet;

fn jaccard_similarity(str1: &str, str2: &str) -> f64 {
    let set1: HashSet<char> = str1.chars().collect();
    let set2: HashSet<char> = str2.chars().collect();
    let intersection = set1.intersection(&set2).count() as f64;
    let union = set1.union(&set2).count() as f64;
    if union == 0.0 {
        0.0
    } else {
        intersection / union
    }
}

pub fn process_possible_japanese(japanese: &str) -> String {
    if kakasi::is_japanese(japanese) == IsJapanese::False {
        japanese.to_string()
    } else {
        kakasi::convert(japanese).romaji
    }
}

fn normalize_text(text: &str) -> String {
    text.to_lowercase()
        .replace(|c: char| !c.is_alphanumeric() && c != ' ', "")
}

fn remove_vowels(word: &str) -> String {
    word.chars()
        .filter(|&c| !"aeiouAEIOU".contains(c))
        .collect()
}

fn remove_consonants(word: &str) -> String {
    word.chars().filter(|&c| "aeiouAEIOU".contains(c)).collect()
}

pub fn process_similarity(japanese_text: &str, romaji_text: &str) -> f32 {
    let japanese_regex = Regex::new(r"[\p{Hiragana}\p{Katakana}\p{Han}]").unwrap();
    if japanese_regex.is_match(japanese_text) {
        let romanized_japanese = process_possible_japanese(japanese_text);
        let normalized_japanese = normalize_text(&romanized_japanese);
        let normalized_romaji = normalize_text(romaji_text);
        let fuzz_value_full = fuzz::ratio(&normalized_japanese, &normalized_romaji);

        let normalized_japanese_consonants = remove_vowels(&normalized_japanese)
            .replace("r", "l")
            .replace("b", "v");
        let normalized_romaji_consonants = remove_vowels(&normalized_romaji)
            .replace("r", "l")
            .replace("b", "v");

        let fuzz_value_consonants = fuzz::ratio(
            &normalized_japanese_consonants,
            &normalized_romaji_consonants,
        );
        let consonant_weight = 0.9;
        let full_weight = 1.0 - consonant_weight;

        (fuzz_value_consonants as f32 * consonant_weight + fuzz_value_full as f32 * full_weight) as  f32
    } else {
        fuzz::ratio(&normalize_text(japanese_text), &normalize_text(romaji_text)) as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const TESTING_LIST: &[(&str, &str)] = &[
        ("デート ・ ア ・ ライブ", "Date A Live"),
        ("モンスター ハンター", "Monster Hunter"),
        ("ファイナル ファンタジー", "Final Fantasy"),
        ("オンライン ゲーム", "Online Game"),
        ("レジェンド オブ ゼルダ", "Legend of Zelda"),
        ("ポケット モンスター", "Pocket Monster"),
        ("ドラゴン クエスト", "Dragon Quest"),
        ("キングダム ハーツ", "Kingdom Hearts"),
        ("ストリート ファイター", "Street Fighter"),
        ("スーパーマリオ", "Super Mario"),
    ];

    const TEST_LIST_FAIL: &[(&str, &str)] = &[
        ("又三郎", "Shayou"),
        ("こんにちは", "Hello"),
        ("ありがとう", "Thank You"),
        ("バナナ", "Bandana"),
        ("コーヒー", "Cough"),
        ("ホテル", "Hostel"),
        ("スピーカー", "Spiker"),
        ("マイク", "Mice"),
        ("バイク", "Back"),
        ("チェック", "Chick"),
    ];

    fn test_similarity(test: (&str, &str), success_function: impl Fn(f32) -> bool) -> f32 {
        let score: f32 = process_similarity(test.0, test.1);
        if !success_function(score) {
            println!("Failed Test: {:?}, Score: {}", test, score);
        }
        println!("{}", score);
        score
    }

    fn test_all(tests: &[(&str, &str)], success_function: impl Fn(f32) -> bool) -> f64 {
        let mut total_score = 0.0;
        for test in tests {
            total_score += test_similarity(*test, &success_function);
        }
        total_score as f64 / tests.len() as f64
    }

    #[test]
    fn test_deltas() {
        let fail_limit = 60.0;

        println!("--------------- Doing match tests ---------------");
        let average_success_score = test_all(&TESTING_LIST, |a| a > fail_limit);

        println!("--------------- Doing False Match tests ---------------");
        let average_fail_score = test_all(&TEST_LIST_FAIL, |a| a < fail_limit);

        println!("Average Success Score: {}", average_success_score);
        println!("Average Fail Score: {}", average_fail_score);
        let delta = average_success_score - average_fail_score;
        println!("Delta: {}", delta);
        assert!(delta > 10.0);
    }
}
