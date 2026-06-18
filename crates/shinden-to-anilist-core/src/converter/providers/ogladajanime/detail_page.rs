use chrono::NaiveDate;
use compact_str::{
    CompactString,
    ToCompactString,
};
use scraper::{
    ElementRef,
    Html,
};
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
        episodes: parse_icon_parent_i32(&document, &episodes_selector),
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
            detail.episodes = detail.episodes.or_else(|| parse_first_i32(value));
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

fn parse_icon_parent_i32(document: &Html, selector: &scraper::Selector) -> Option<i32> {
    document
        .select(selector)
        .filter_map(|icon| icon.parent().and_then(ElementRef::wrap))
        .map(element_text)
        .find_map(|value| parse_first_i32(&value))
}

fn parse_first_i32(value: &str) -> Option<i32> {
    let start = value.find(|char: char| char.is_ascii_digit())?;
    let digits = value[start..]
        .chars()
        .take_while(char::is_ascii_digit)
        .collect::<String>();

    digits.parse().ok()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::converter::{
        database::{
            AnimeStatus,
            AnimeType,
        },
        providers::ogladajanime::models::OgladajAnimeDetail,
    };

    #[test]
    fn parses_episode_count_from_labeled_detail_value() {
        let html = r#"
            <html>
                <body>
                    <div class="outlines"><span class="btn-outline-warning">ONA</span></div>
                    <p class="m-0">Odcinki: 238 odc.</p>
                    <p class="m-0">Status: Zakończone</p>
                    <p class="m-0">Start emisji: 2018-01-20</p>
                </body>
            </html>
        "#;

        let detail = parse_detail_page("/anime/douluo-dalu-2nd-season", html).unwrap();

        assert_eq!(
            detail,
            OgladajAnimeDetail {
                anime_type: Some(AnimeType::Ona),
                anime_status: Some(AnimeStatus::Finished),
                premiere_date: NaiveDate::from_ymd_opt(2018, 1, 20),
                episodes: Some(238),
                ..OgladajAnimeDetail::default()
            }
        );
    }

    #[test]
    fn parses_tooltip_episode_count_from_play_icon_parent() {
        let html = serde_json::json!({
            "data": r#"
                <div>
                    <span class="badge-warning">ONA</span>
                    <small class="text-trim">Douluo Dalu 2nd Season | 斗罗大陆 第二季</small>
                    <small><i class="fa fa-play"></i> 238 odc.</small>
                    <small>Status: Zakończone</small>
                    <small>Start emisji: 2018-01-20</small>
                </div>
            "#
        })
        .to_string();

        let detail = parse_tooltip_page("/manager.php?action=get_anime_tooltip", &html).unwrap();

        assert_eq!(detail.anime_type, Some(AnimeType::Ona));
        assert_eq!(detail.anime_status, Some(AnimeStatus::Finished));
        assert_eq!(detail.premiere_date, NaiveDate::from_ymd_opt(2018, 1, 20));
        assert_eq!(detail.episodes, Some(238));
        assert_eq!(detail.original_title.as_deref(), Some("斗罗大陆 第二季"));
    }
}
