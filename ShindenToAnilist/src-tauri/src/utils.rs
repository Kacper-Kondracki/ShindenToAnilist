use chrono::{Days, NaiveDate};
use serde::de::Error;
use serde::{Deserialize, Deserializer};

pub fn de_timestamp<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
where
    D: Deserializer<'de>,
{
    let timestamp = i64::deserialize(deserializer)?;
    chrono::DateTime::from_timestamp(timestamp, 0)
        .map(|t| t.date_naive() + Days::new(1))
        .ok_or(D::Error::custom("could not deserialize date"))
}

pub fn de_opt_timestamp<'de, D>(deserializer: D) -> Result<Option<NaiveDate>, D::Error>
where
    D: Deserializer<'de>,
{
    let timestamp: Option<i64> = Option::deserialize(deserializer)?;
    if let Some(timestamp) = timestamp {
        chrono::DateTime::from_timestamp(timestamp, 0)
            .map(|t| Some(t.date_naive() + Days::new(1)))
            .ok_or(D::Error::custom("could not deserialize date"))
    } else {
        Ok(None)
    }
}

pub fn de_from_string<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: std::str::FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    let s = String::deserialize(deserializer)?;
    s.parse().map_err(Error::custom)
}

#[derive(Deserialize)]
#[serde(untagged)]
enum BoolOrNum {
    Bool(bool),
    Num(i64),
}

pub fn de_bool_from_num<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let bool_or_num = BoolOrNum::deserialize(deserializer)?;
    Ok(match bool_or_num {
        BoolOrNum::Bool(bool) => bool,
        BoolOrNum::Num(num) => num == 1,
    })
}
