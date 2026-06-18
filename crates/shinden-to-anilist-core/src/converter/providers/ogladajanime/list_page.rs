use compact_str::{
    CompactString,
    ToCompactString,
};
use scraper::Html;

use super::{
    OgladajAnimeError,
    models::OgladajAnimeListItem,
};
use crate::converter::{
    common::AnimeId,
    database::AnimeType,
    exporter::WatchStatus,
    providers::scraping::{
        attr,
        element_text,
        selector,
    },
};

pub(super) fn parse_list_page(
    path: &str,
    html: &str,
) -> Result<(Vec<OgladajAnimeListItem>, usize), OgladajAnimeError> {
    let document = Html::parse_document(html);
    let row_selector = selector("table#my_anime_table tbody tr[id^=\"anime_list_item_\"]");
    let cell_selector = selector("td");
    let title_selector = selector("a[href^=\"/anime/\"]");

    let items = document
        .select(&row_selector)
        .map(|row| {
            let cells = row.select(&cell_selector).collect::<Vec<_>>();
            if cells.len() < 7 {
                return Err(OgladajAnimeError::Parse {
                    path: path.to_string(),
                    message: "anime list row has fewer than seven cells".to_string(),
                });
            }

            let row_id = attr(row, "id").ok_or_else(|| OgladajAnimeError::Parse {
                path: path.to_string(),
                message: "anime list row is missing id".to_string(),
            })?;
            let id = parse_row_id(&row_id).ok_or_else(|| OgladajAnimeError::Parse {
                path: path.to_string(),
                message: format!("anime list row id is not recognized: {row_id}"),
            })?;
            let title_link =
                cells[2]
                    .select(&title_selector)
                    .next()
                    .ok_or_else(|| OgladajAnimeError::Parse {
                        path: path.to_string(),
                        message: "anime list row is missing title link".to_string(),
                    })?;
            let href = attr(title_link, "href").ok_or_else(|| OgladajAnimeError::Parse {
                path: path.to_string(),
                message: "anime list title link is missing href".to_string(),
            })?;
            let slug = anime_slug(&href).ok_or_else(|| OgladajAnimeError::Parse {
                path: path.to_string(),
                message: format!("anime href is not an OgladajAnime anime path: {href}"),
            })?;
            let title = element_text(title_link);
            let status = element_text(cells[3]);
            let score = element_text(cells[4]);
            let progress = element_text(cells[5]);
            let anime_type = element_text(cells[6]);
            let (watched_episodes, total_episodes) = parse_progress(&progress);

            Ok(OgladajAnimeListItem {
                id,
                slug,
                title,
                anime_type: parse_type(&anime_type),
                watch_status: parse_watch_status(&status).ok_or_else(|| OgladajAnimeError::Parse {
                    path: path.to_string(),
                    message: format!("anime list status is not recognized: {status}"),
                })?,
                watched_episodes,
                total_episodes,
                score: parse_score(&score),
            })
        })
        .collect::<Result<Vec<_>, OgladajAnimeError>>()?;

    Ok((items, 1))
}

fn parse_row_id(value: &str) -> Option<AnimeId> {
    value
        .strip_prefix("anime_list_item_")
        .and_then(|id| id.parse().ok())
}

fn anime_slug(href: &str) -> Option<CompactString> {
    href.strip_prefix("/anime/")
        .or_else(|| href.split_once("/anime/").map(|(_, slug)| slug))
        .map(|slug| slug.trim_matches('/'))
        .filter(|slug| !slug.is_empty())
        .map(|slug| slug.to_compact_string())
}

fn parse_progress(value: &str) -> (i32, Option<i32>) {
    let Some((watched, total)) = value.trim().split_once('/') else {
        return (0, None);
    };

    (
        watched.trim().parse().unwrap_or_default(),
        total.trim().parse().ok(),
    )
}

fn parse_score(value: &str) -> Option<i32> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.parse().ok()).flatten()
}

pub(super) fn parse_type(value: &str) -> Option<AnimeType> {
    match value.trim().to_ascii_lowercase().as_str() {
        "tv" => Some(AnimeType::Tv),
        "movie" | "film" => Some(AnimeType::Movie),
        "ova" => Some(AnimeType::Ova),
        "ona" => Some(AnimeType::Ona),
        "special" | "specjal" | "speciale" => Some(AnimeType::Special),
        _ => None,
    }
}

pub(super) fn parse_watch_status(value: &str) -> Option<WatchStatus> {
    match value.trim() {
        "Oglądam" | "Ogladam" => Some(WatchStatus::Watching),
        "Obejrzane" => Some(WatchStatus::Completed),
        "Planuje" | "Planuję" => Some(WatchStatus::PlanToWatch),
        "Wstrzymane" => Some(WatchStatus::OnHold),
        "Porzucone" => Some(WatchStatus::Dropped),
        _ => None,
    }
}
