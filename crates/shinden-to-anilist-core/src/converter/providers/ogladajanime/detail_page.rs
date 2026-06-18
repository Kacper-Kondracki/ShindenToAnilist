use chrono::NaiveDate;
use compact_str::{
    CompactString,
    ToCompactString,
};
use scraper::Html;
use serde::Deserialize;

use super::{
    OgladajAnimeError,
    list_page::parse_type,
    models::{
        OgladajAnimeDetail,
        clean_cell_text,
    },
};
use crate::converter::{
    database::AnimeStatus,
    providers::scraping::{
        attr,
        element_text,
        extract_mal_id,
        selector,
    },
};

pub(super) fn parse_detail_page(path: &str, html: &str) -> Result<OgladajAnimeDetail, OgladajAnimeError> {
    let document = Html::parse_document(html);
    let mal_selector = selector("a[href*=\"myanimelist.net/anime/\"]");
    let labeled_value_selector = selector("p.m-0");
    let outline_type_selector = selector("div.outlines span.btn-outline-warning");

    let mal_id = document
        .select(&mal_selector)
        .find_map(|element| attr(element, "href"))
        .and_then(|href| extract_mal_id(&href));

    let mut detail = OgladajAnimeDetail {
        mal_id,
        anime_type: document
            .select(&outline_type_selector)
            .next()
            .map(element_text)
            .and_then(|value| parse_type(&value)),
        ..OgladajAnimeDetail::default()
    };

    for element in document.select(&labeled_value_selector) {
        let text = element_text(element);
        let Some((label, value)) = text.split_once(':') else {
            continue;
        };
        merge_labeled_value(&mut detail, label, value);
    }

    if detail.mal_id.is_none()
        && detail.original_title.is_none()
        && detail.premiere_date.is_none()
        && detail.episodes.is_none()
    {
        return Err(OgladajAnimeError::Parse {
            path: path.to_string(),
            message: "anime detail page did not contain expected metadata".to_string(),
        });
    }

    Ok(detail)
}

pub(super) fn parse_tooltip_page(path: &str, html: &str) -> Result<OgladajAnimeDetail, OgladajAnimeError> {
    let response =
        serde_json::from_str::<TooltipResponse>(html).map_err(|error| OgladajAnimeError::Parse {
            path: path.to_string(),
            message: format!("anime tooltip response is not valid JSON: {error}"),
        })?;
    let document = Html::parse_fragment(&response.data);
    let labeled_value_selector = selector("small");
    let outline_type_selector = selector(".badge-warning");
    let episodes_selector = selector("i.fa-play");
    let original_title_selector = selector("small.text-trim");

    let mut detail = OgladajAnimeDetail {
        anime_type: document
            .select(&outline_type_selector)
            .next()
            .map(element_text)
            .and_then(|value| parse_type(&value)),
        episodes: document
            .select(&episodes_selector)
            .next()
            .map(element_text)
            .and_then(|value| parse_leading_i32(&value)),
        original_title: document
            .select(&original_title_selector)
            .next()
            .map(element_text)
            .and_then(|value| japanese_alias(&value)),
        ..OgladajAnimeDetail::default()
    };

    for element in document.select(&labeled_value_selector) {
        let text = element_text(element);
        let Some((label, value)) = text.split_once(':') else {
            continue;
        };
        merge_labeled_value(&mut detail, label, value);
    }

    if detail.original_title.is_none()
        && detail.anime_status.is_none()
        && detail.premiere_date.is_none()
        && detail.episodes.is_none()
    {
        return Err(OgladajAnimeError::Parse {
            path: path.to_string(),
            message: "anime tooltip did not contain expected metadata".to_string(),
        });
    }

    Ok(detail)
}

fn merge_labeled_value(detail: &mut OgladajAnimeDetail, label: &str, value: &str) {
    match label.trim() {
        "Japoński" | "Japonski" => {
            detail.original_title = detail.original_title.clone().or_else(|| clean_cell_text(value));
        },
        "Odcinki" => {
            detail.episodes = detail.episodes.or_else(|| value.trim().parse().ok());
        },
        "Status" => {
            detail.anime_status = detail.anime_status.or_else(|| parse_polish_status(value));
        },
        "Start emisji" => {
            detail.premiere_date = detail.premiere_date.or_else(|| parse_date(value));
        },
        "Koniec emisji" => {
            detail.finish_date = detail.finish_date.or_else(|| parse_date(value));
        },
        _ => {},
    }
}

fn parse_polish_status(value: &str) -> Option<AnimeStatus> {
    match value.trim() {
        "Zakończone" | "Zakonczone" => Some(AnimeStatus::Finished),
        "Emitowane" => Some(AnimeStatus::Ongoing),
        "Nadchodzące" | "Nadchodzace" => Some(AnimeStatus::Upcoming),
        _ => None,
    }
}

fn parse_date(value: &str) -> Option<NaiveDate> { NaiveDate::parse_from_str(value.trim(), "%Y-%m-%d").ok() }

fn parse_leading_i32(value: &str) -> Option<i32> {
    let value = value.trim_start();
    let digits = value.chars().take_while(char::is_ascii_digit).collect::<String>();

    (!digits.is_empty()).then(|| digits.parse().ok()).flatten()
}

fn japanese_alias(value: &str) -> Option<CompactString> {
    value
        .split('|')
        .map(str::trim)
        .find(|part| part.chars().any(is_japanese_char))
        .map(|part| part.to_compact_string())
}

fn is_japanese_char(value: char) -> bool {
    matches!(
        value,
        '\u{3040}'..='\u{30ff}' | '\u{3400}'..='\u{9fff}' | '\u{f900}'..='\u{faff}'
    )
}

#[derive(Deserialize)]
struct TooltipResponse {
    data: String,
}
