use std::borrow::Cow;

use approx::relative_eq;
use chrono::NaiveDate;
use compact_str::CompactString;
use serde::{
    Deserialize,
    Deserializer,
    de::Error,
};
use unicode_normalization::UnicodeNormalization;
use wana_kana::{
    ConvertJapanese,
    IsJapaneseChar,
};

pub(crate) fn de_timestamp<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: From<Option<NaiveDate>> + serde::Deserialize<'de>,
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum DateOrTimestamp {
        Date(NaiveDate),
        Timestamp(i64),
    }

    let date_or_timestamp = Option::<DateOrTimestamp>::deserialize(deserializer)?;
    Ok(match date_or_timestamp {
        Some(DateOrTimestamp::Date(date)) => T::from(Some(date)),
        Some(DateOrTimestamp::Timestamp(timestamp)) => T::from(Some(
            chrono::DateTime::from_timestamp(timestamp, 0)
                .map(|t| t.date_naive())
                .ok_or(D::Error::custom("could not deserialize date"))?,
        )),
        None => T::from(None),
    })
}

pub(crate) fn de_from_string<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: std::str::FromStr + serde::Deserialize<'de>,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum TOrString<T> {
        T(T),
        String(CompactString),
    }

    let t_or_string = TOrString::<T>::deserialize(deserializer)?;
    Ok(match t_or_string {
        TOrString::T(t) => t,
        TOrString::String(string) => string.parse().map_err(Error::custom)?,
    })
}

pub(crate) fn de_bool_from_num<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum BoolOrNum {
        Bool(bool),
        Num(i64),
    }

    let bool_or_num = BoolOrNum::deserialize(deserializer)?;
    Ok(match bool_or_num {
        BoolOrNum::Bool(bool) => bool,
        BoolOrNum::Num(num) => num == 1,
    })
}

pub(crate) fn ge_tol(a: f32, b: f32) -> bool { a > b || relative_eq!(a, b) }

pub fn normalize_str(s: &str) -> CompactString {
    let needs_romaji = s.chars().any(|c| c.is_japanese());
    let source = if needs_romaji {
        Cow::Owned(s.to_romaji())
    } else {
        Cow::Borrowed(s)
    };
    let mut result = CompactString::with_capacity(source.len());
    let mut last_was_space = true;
    for c in source.nfd().filter(|c| c.is_ascii()) {
        if c.is_whitespace() {
            if !last_was_space && !result.is_empty() {
                result.push(' ');
                last_was_space = true;
            }
        } else {
            result.push(c.to_ascii_lowercase());
            last_was_space = false;
        }
    }

    if last_was_space && !result.is_empty() {
        result.pop();
    }

    result
}
