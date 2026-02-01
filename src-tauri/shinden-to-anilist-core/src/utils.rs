use chrono::{Days, NaiveDate};
use itertools::Itertools;
use serde::{Deserialize, Deserializer, Serializer, de::Error};
use unicode_normalization::UnicodeNormalization;
use wana_kana::ConvertJapanese;

pub fn de_timestamp<'de, T, D>(deserializer: D) -> Result<T, D::Error>
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
                .map(|t| t.date_naive() + Days::new(1))
                .ok_or(D::Error::custom("could not deserialize date"))?,
        )),
        None => T::from(None),
    })
}

pub fn de_from_string<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: std::str::FromStr + serde::Deserialize<'de>,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum TOrString<T> {
        T(T),
        String(String),
    }
    let t_or_string = TOrString::<T>::deserialize(deserializer)?;
    Ok(match t_or_string {
        TOrString::T(t) => t,
        TOrString::String(string) => string.parse().map_err(Error::custom)?,
    })
}

pub fn de_bool_from_num<'de, D>(deserializer: D) -> Result<bool, D::Error>
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

pub fn ser_mal_date<S>(date: &Option<NaiveDate>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let date = if let Some(date) = date {
        date.to_string()
    } else {
        "0000-00-00".to_string()
    };

    serializer.serialize_str(&date)
}

pub trait NormalizeStr {
    fn normalize(&self) -> String;
}
impl<T: AsRef<str>> NormalizeStr for T {
    fn normalize(&self) -> String {
        self.as_ref()
            .to_romaji()
            .nfc()
            .collect::<String>()
            .chars()
            .filter(|x| x.is_ascii())
            .collect::<String>()
            .split_whitespace()
            .join(" ")
            .to_lowercase()
    }
}
