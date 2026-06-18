use scraper::{
    ElementRef,
    Html,
};

use super::{
    AnimeZoneError,
    list_page::parse_polish_status,
    models::{
        AnimeZoneDetail,
        clean_cell_text,
    },
};
use crate::converter::{
    database::AnimeType,
    providers::scraping::{
        attr,
        element_text,
        extract_mal_id,
        selector,
    },
};

pub(super) fn parse_detail_page(path: &str, html: &str) -> Result<AnimeZoneDetail, AnimeZoneError> {
    let document = Html::parse_document(html);
    let mal_selector = selector("a.mal-link[href*=\"myanimelist.net/anime/\"]");
    let table_row_selector = selector("table tr");
    let cell_selector = selector("td");
    let episode_row_selector = selector("table.episodes tbody tr");

    let mal_id = document
        .select(&mal_selector)
        .find_map(|element| attr(element, "href"))
        .and_then(|href| extract_mal_id(&href));

    let mut detail = AnimeZoneDetail {
        mal_id,
        episodes: count_episode_rows(&document, &episode_row_selector),
        ..AnimeZoneDetail::default()
    };

    for row in document.select(&table_row_selector) {
        let cells = row.select(&cell_selector).collect::<Vec<_>>();
        if cells.len() < 2 {
            continue;
        }

        let label = element_text(cells[0]);
        let value = element_text(cells[1]);
        merge_labeled_value(&mut detail, &label, &value);
    }

    if detail.mal_id.is_none() && detail.alternative_title.is_none() && detail.year.is_none() {
        return Err(AnimeZoneError::Parse {
            path: path.to_string(),
            message: "anime detail page did not contain expected metadata".to_string(),
        });
    }

    Ok(detail)
}

fn merge_labeled_value(detail: &mut AnimeZoneDetail, label: &str, value: &str) {
    match label.trim() {
        "Alternatywny tytuł" | "Alternatywny tytul" => {
            detail.alternative_title = detail
                .alternative_title
                .clone()
                .or_else(|| clean_cell_text(value));
        },
        "Rok produkcji" => {
            detail.year = detail.year.or_else(|| value.trim().parse().ok());
        },
        "Rodzaj" => {
            detail.anime_type = detail.anime_type.or_else(|| parse_type(value));
        },
        "Status" => {
            detail.anime_status = detail.anime_status.or_else(|| parse_polish_status(value));
        },
        _ => {},
    }
}

fn parse_type(value: &str) -> Option<AnimeType> {
    match value.trim().to_ascii_lowercase().as_str() {
        "tv" => Some(AnimeType::Tv),
        "movie" | "film" => Some(AnimeType::Movie),
        "ova" => Some(AnimeType::Ova),
        "ona" => Some(AnimeType::Ona),
        "special" | "specjal" | "speciale" => Some(AnimeType::Special),
        _ => None,
    }
}

fn count_episode_rows(document: &Html, selector: &scraper::Selector) -> Option<i32> {
    let count = document
        .select(selector)
        .filter(|row| has_episode_link(*row))
        .count();

    (count > 0).then(|| count as i32)
}

fn has_episode_link(row: ElementRef<'_>) -> bool {
    let selector = selector("a[href*=\"/odcinek/\"]");
    row.select(&selector).next().is_some()
}
