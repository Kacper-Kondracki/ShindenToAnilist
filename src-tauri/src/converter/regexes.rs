use regex::Regex;
use std::sync::LazyLock;

const OPT_SEP_PREFIX: &str = r"(?:[:-]\s*)?";
const OPT_BRACKET_PREFIX: &str = r"(?:[(\[<]\s*)?";
const OPT_ARTICLE: &str = r"(?:\s*(?:\bthe\b|\ba\b)\s*)?";
const OPT_SEP_MIDDLE: &str = r"\s*[.,-:]?\s*";
const OPT_PUNCT: &str = r"[.!?]?";
const OPT_BRACKET_SUFFIX: &str = r"(?:\s*[)\]>])?";
const SEASON_WORD: &str = r"(?:\bseason\b|\bmovie\b|\bova\b|\bs(?:eries)?\b)";
const PART_WORD: &str = r"(?:\bpart\b)";

const NUMERAL_END: &str = r"\s*(?:st\b|nd\b|rd\b|th\b)?";
const DECIMAL_NUM: &str = r"(\d)";
const ROMAN_NUM: &str = r"(\bI{1,3}\b|\bIV\b|\bVI{0,3}\b|\bIX\b)";
const WORD_NUM: &str = r"(\bone\b|\btwo\b|\bthree\b|\bfour\b|\bfive\b|\bsix\b|\bseven\b|\beight\b|\bnine\b|\bfinal\b|\blast\b)";
const NUMERAL_NUM: &str = r"(\bfirst\b|\bsecond\b|\bthird\b|\bfourth\b|\bfifth\b|\bsixth\b|\bseventh\b|\beighth\b|\bninth\b|\blast\b|\bfinal\b)";
const ALL_NUM: &str = r"(\bone\b|\btwo\b|\bthree\b|\bfour\b|\bfive\b|\bsix\b|\bseven\b|\beight\b|\bnine\b|\bfirst\b|\bsecond\b|\bthird\b|\bfourth\b|\bfifth\b|\bsixth\b|\bseventh\b|\beighth\b|\bninth\b|\blast\b|\bfinal\b)";

pub static SEASON: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(&format!(r"(?i){}", SEASON_WORD)).unwrap());
pub static PART: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(&format!(r"(?i){}", PART_WORD)).unwrap());

pub static YEAR: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        r"{OPT_BRACKET_PREFIX}((?:19|20)\d{{2}}){OPT_BRACKET_SUFFIX}"
    ))
    .unwrap()
});
pub static SEASON_DECIMAL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
    r"(?i){OPT_SEP_PREFIX}{OPT_BRACKET_PREFIX}{OPT_ARTICLE}{SEASON_WORD}{OPT_SEP_MIDDLE}{OPT_BRACKET_PREFIX}{DECIMAL_NUM}{OPT_PUNCT}{OPT_BRACKET_SUFFIX}{OPT_BRACKET_SUFFIX}"
)).unwrap()
});
pub static SEASON_ROMAN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
    r"(?i){OPT_SEP_PREFIX}{OPT_BRACKET_PREFIX}{OPT_ARTICLE}{SEASON_WORD}{OPT_SEP_MIDDLE}{OPT_BRACKET_PREFIX}{ROMAN_NUM}{OPT_PUNCT}{OPT_BRACKET_SUFFIX}{OPT_BRACKET_SUFFIX}"
)).unwrap()
});
pub static SEASON_NUMERAL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
    r"(?i){OPT_SEP_PREFIX}{OPT_BRACKET_PREFIX}{OPT_ARTICLE}{SEASON_WORD}{OPT_SEP_MIDDLE}{OPT_BRACKET_PREFIX}{WORD_NUM}{OPT_PUNCT}{OPT_BRACKET_SUFFIX}{OPT_BRACKET_SUFFIX}"
)).unwrap()
});
pub static DECIMAL_SEASON: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
    r"(?i){OPT_SEP_PREFIX}{OPT_BRACKET_PREFIX}{OPT_ARTICLE}{OPT_BRACKET_PREFIX}{DECIMAL_NUM}{NUMERAL_END}{OPT_PUNCT}{OPT_BRACKET_SUFFIX}{OPT_SEP_MIDDLE}{SEASON_WORD}{OPT_BRACKET_SUFFIX}"
)).unwrap()
});
pub static ROMAN_SEASON: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
    r"(?i){OPT_SEP_PREFIX}{OPT_BRACKET_PREFIX}{OPT_ARTICLE}{OPT_BRACKET_PREFIX}{ROMAN_NUM}{OPT_PUNCT}{OPT_BRACKET_SUFFIX}{OPT_SEP_MIDDLE}{SEASON_WORD}{OPT_BRACKET_SUFFIX}"
)).unwrap()
});
pub static NUMERAL_SEASON: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
    r"(?i){OPT_SEP_PREFIX}{OPT_BRACKET_PREFIX}{OPT_ARTICLE}{OPT_BRACKET_PREFIX}{NUMERAL_NUM}{OPT_PUNCT}{OPT_BRACKET_SUFFIX}{OPT_SEP_MIDDLE}{SEASON_WORD}{OPT_BRACKET_SUFFIX}"
)).unwrap()
});
pub static SEASON_DECIMAL_END: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
    r"(?i){OPT_SEP_PREFIX}{OPT_BRACKET_PREFIX}{OPT_ARTICLE}\D{DECIMAL_NUM}{NUMERAL_END}{OPT_PUNCT}{OPT_BRACKET_SUFFIX}\s*$"
)).unwrap()
});
pub static SEASON_ROMAN_END: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
    r"(?i){OPT_SEP_PREFIX}{OPT_BRACKET_PREFIX}{OPT_ARTICLE}{ROMAN_NUM}{OPT_PUNCT}{OPT_BRACKET_SUFFIX}\s*$"
)).unwrap()
});
pub static SEASON_NUMERAL_END: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
    r"(?i){OPT_SEP_PREFIX}{OPT_BRACKET_PREFIX}{OPT_ARTICLE}{ALL_NUM}{OPT_PUNCT}{OPT_BRACKET_SUFFIX}\s*$"
)).unwrap()
});
pub static PART_DECIMAL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
    r"(?i){OPT_SEP_PREFIX}{OPT_BRACKET_PREFIX}{OPT_ARTICLE}{PART_WORD}\.?{OPT_SEP_MIDDLE}{OPT_BRACKET_PREFIX}{DECIMAL_NUM}{OPT_PUNCT}{OPT_BRACKET_SUFFIX}{OPT_BRACKET_SUFFIX}"
)).unwrap()
});
pub static PART_ROMAN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
    r"(?i){OPT_SEP_PREFIX}{OPT_BRACKET_PREFIX}{OPT_ARTICLE}{PART_WORD}\.?{OPT_SEP_MIDDLE}{OPT_BRACKET_PREFIX}{ROMAN_NUM}{OPT_PUNCT}{OPT_BRACKET_SUFFIX}{OPT_BRACKET_SUFFIX}"
)).unwrap()
});
pub static PART_NUMERAL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
    r"(?i){OPT_SEP_PREFIX}{OPT_BRACKET_PREFIX}{OPT_ARTICLE}{PART_WORD}\.?{OPT_SEP_MIDDLE}{OPT_BRACKET_PREFIX}{WORD_NUM}{OPT_PUNCT}{OPT_BRACKET_SUFFIX}{OPT_BRACKET_SUFFIX}"
)).unwrap()
});

pub static DECIMAL_PART: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
    r"(?i){OPT_SEP_PREFIX}{OPT_BRACKET_PREFIX}{OPT_ARTICLE}{OPT_BRACKET_PREFIX}{DECIMAL_NUM}{NUMERAL_END}{OPT_PUNCT}{OPT_BRACKET_SUFFIX}{OPT_SEP_MIDDLE}{PART_WORD}\.?{OPT_BRACKET_SUFFIX}"
)).unwrap()
});
pub static ROMAN_PART: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
    r"(?i){OPT_SEP_PREFIX}{OPT_BRACKET_PREFIX}{OPT_ARTICLE}{OPT_BRACKET_PREFIX}{ROMAN_NUM}{OPT_PUNCT}{OPT_BRACKET_SUFFIX}{OPT_SEP_MIDDLE}{PART_WORD}\.?{OPT_BRACKET_SUFFIX}"
)).unwrap()
});
pub static NUMERAL_PART: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
    r"(?i){OPT_SEP_PREFIX}{OPT_BRACKET_PREFIX}{OPT_ARTICLE}{OPT_BRACKET_PREFIX}{NUMERAL_NUM}{OPT_PUNCT}{OPT_BRACKET_SUFFIX}{OPT_SEP_MIDDLE}{PART_WORD}\.?{OPT_BRACKET_SUFFIX}"
)).unwrap()
});
pub static ANIME_TYPE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        r"(?i){OPT_BRACKET_PREFIX}(\btv|\bmovie|\bova|\bona|\bspecial|\bmusic){OPT_BRACKET_SUFFIX}"
    ))
    .unwrap()
});
