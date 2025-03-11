use itertools::Itertools;
use lazy_static::lazy_static;
use regex::{Captures, Regex};
use std::collections::HashMap;

lazy_static! {

    static ref REPLACEMENT_RULES: HashMap<&'static str, &'static str> = {
        let mut map = HashMap::new();
        map.insert("ļ", "[ļĻ]");
        map.insert("l", "[l˥ļĻΛ]");
        map.insert("ź", "[źŹ]");
        map.insert("z", "[zźŹ]");
        map.insert("ou", "(ou|ō|o)");
        map.insert("oo", "(oo|ō|o)");
        map.insert("oh", "(oh|ō|o)");
        map.insert("wo", "(wo|o)");
        map.insert("ō", "[Ōō]");
        map.insert("o", "([oōŌóòöôøӨΦο]|ou|oo|oh|wo)");
        map.insert("uu", "(uu|u|ū)");
        map.insert("ū", "[ūŪ]");
        map.insert("u", "([uūŪûúùüǖμ]|uu)");
        map.insert("aa", "(aa|a)");
        map.insert("ae", "(ae|æ)");
        map.insert("λ", "[λΛ]");
        map.insert("a", "([aäãά@âàáạåæā∀Λ]|aa)");
        map.insert("c", "[cςč℃Ↄ]");
        map.insert("é", "[éÉ]");
        map.insert("e", "[eəéÉêёëèæē]");
        map.insert("'", "['’ˈ]");
        map.insert("n", "[nñ]");
        map.insert("0", "[0Ө]");
        map.insert("2", "[2²₂]");
        map.insert("3", "[3³]");
        map.insert("5", "[5⁵]");
        map.insert("*", "[*✻＊✳︎]");
        map.insert(" ", "([^\\w]+|_+)");
        map.insert("i", "([iíίɪ]|ii)");
        map.insert("x", "[x×]");
        map.insert("b", "[bßβ]");
        map.insert("r", "[rЯ]");
        map.insert("s", "[sς]");
        map
    };

    // Build a single regex that matches all keys in REPLACEMENT_RULES
    static ref REPLACEMENT_REGEX: Regex = {
        let pattern = REPLACEMENT_RULES.keys()
            .map(|key| regex::escape(key))
            .collect::<Vec<String>>()
            .join("|"); // Join with `|` to create an "OR" regex
        Regex::new(&pattern).unwrap()
    };

    static ref ARTIST_REGEX: Regex = {
        Regex::new(&r"(\w+) \(CV(:|\.)\s*(?P<a>.*?)\)").unwrap()
    };
}

// simply unwraps possible (CV:artistname) before calling create_regex
pub fn create_artist_regex(input: Vec<&String>) -> String {
    input
        .iter()
        .map(|a| {
            let parsed_artist = ARTIST_REGEX.replace_all(a, "$a");
            create_regex(&parsed_artist)
        })
        .intersperse("|".to_string())
        .collect()
}

/// Replaces using a precompiled regex
pub fn create_regex(input: &str) -> String {
    format!(
        "^{}$",
        REPLACEMENT_REGEX.replace_all(input, |caps: &Captures| {
            let matched = caps.get(0).unwrap().as_str();
            // Directly return the replacement from the map or the original match
            REPLACEMENT_RULES.get(matched).map_or_else(
                || matched.to_string(),                 // No replacement, keep the match as is
                |&replacement| replacement.to_string(), // Replacement exists, return it
            )
        })
    )
}
