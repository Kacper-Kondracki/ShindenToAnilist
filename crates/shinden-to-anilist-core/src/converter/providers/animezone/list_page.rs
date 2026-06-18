use compact_str::{
    CompactString,
    ToCompactString,
};
use scraper::Html;

use super::{
    AnimeZoneError,
    models::{
        AnimeZoneListItem,
        AnimeZoneSection,
    },
};
use crate::converter::{
    database::AnimeStatus,
    providers::scraping::{
        attr,
        discover_page_count,
        element_text,
        select_text,
        selector,
    },
};

pub(super) fn parse_list_page(
    path: &str,
    section: AnimeZoneSection,
    html: &str,
) -> Result<(Vec<AnimeZoneListItem>, usize), AnimeZoneError> {
    let document = Html::parse_document(html);
    let card_selector = selector("div.user-activity div.categories");
    let title_selector = selector("div.title p.label a[href^=\"/anime/\"]");
    let info_selector = selector("p.info");

    let items = document
        .select(&card_selector)
        .map(|card| {
            let title_link = card
                .select(&title_selector)
                .next()
                .ok_or_else(|| AnimeZoneError::Parse {
                    path: path.to_string(),
                    message: "anime card is missing title link".to_string(),
                })?;

            let title = element_text(title_link);
            let href = attr(title_link, "href").ok_or_else(|| AnimeZoneError::Parse {
                path: path.to_string(),
                message: "anime card title link is missing href".to_string(),
            })?;
            let slug = anime_slug(&href).ok_or_else(|| AnimeZoneError::Parse {
                path: path.to_string(),
                message: format!("anime href is not an AnimeZone anime path: {href}"),
            })?;
            let info = select_text(card, &info_selector).unwrap_or_default();

            Ok(AnimeZoneListItem {
                section,
                slug,
                title,
                score: parse_score(&info),
                site_status: parse_status(&info),
            })
        })
        .collect::<Result<Vec<_>, AnimeZoneError>>()?;

    Ok((items, discover_page_count(&document)))
}

fn anime_slug(href: &str) -> Option<CompactString> {
    href.strip_prefix("/anime/")
        .or_else(|| href.split_once("/anime/").map(|(_, slug)| slug))
        .map(|slug| slug.trim_matches('/'))
        .filter(|slug| !slug.is_empty())
        .map(|slug| slug.to_compact_string())
}

fn parse_score(text: &str) -> Option<i32> {
    let (_, after) = text.split_once("Ocena ")?;
    let (score, _) = after.split_once("/10")?;
    score.trim().parse().ok()
}

fn parse_status(text: &str) -> Option<AnimeStatus> {
    let (_, after) = text.split_once("Status:")?;
    let status = after
        .split("Dodane")
        .next()
        .unwrap_or(after)
        .split("Ocena")
        .next()
        .unwrap_or(after)
        .trim();

    parse_polish_status(status)
}

pub(super) fn parse_polish_status(value: &str) -> Option<AnimeStatus> {
    match value.trim() {
        "Zakończone" | "Zakonczone" => Some(AnimeStatus::Finished),
        "Emitowane" => Some(AnimeStatus::Ongoing),
        "Nadchodzące" | "Nadchodzace" => Some(AnimeStatus::Upcoming),
        _ => None,
    }
}
