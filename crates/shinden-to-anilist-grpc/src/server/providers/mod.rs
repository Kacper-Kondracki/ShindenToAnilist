pub(super) mod animezone;
pub(super) mod direct;
pub(super) mod ogladajanime;
pub(super) mod scraped;
pub(super) mod shinden;

use shinden_to_anilist_core::{
    matcher::DefaultMatcher,
    searcher::Search,
};

use crate::{
    DatabaseState,
    pb::SourceMatchResult,
    source::SourceList,
};

pub(super) fn match_source_list(
    source: &SourceList,
    database: &DatabaseState,
    options: Search,
    matcher: &DefaultMatcher,
) -> Vec<SourceMatchResult> {
    match source {
        SourceList::Shinden(shinden) => shinden::match_source_list(shinden, database, options, matcher),
        SourceList::AnimeZone(animezone) => {
            animezone::match_source_list(animezone, database, options, matcher)
        },
        SourceList::OgladajAnime(ogladajanime) => {
            ogladajanime::match_source_list(ogladajanime, database, options, matcher)
        },
    }
}
