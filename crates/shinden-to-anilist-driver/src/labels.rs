use shinden_to_anilist_core::{
    database::{
        AnimeStatus,
        AnimeType,
    },
    exporter::WatchStatus,
};

pub fn anime_status(value: AnimeStatus) -> &'static str {
    match value {
        AnimeStatus::Finished => "finished",
        AnimeStatus::Ongoing => "ongoing",
        AnimeStatus::Upcoming => "upcoming",
        AnimeStatus::Unknown => "unknown",
    }
}

pub fn anime_type(value: AnimeType) -> &'static str {
    match value {
        AnimeType::Tv => "tv",
        AnimeType::Movie => "movie",
        AnimeType::Ova => "ova",
        AnimeType::Ona => "ona",
        AnimeType::Special => "special",
        AnimeType::Unknown => "unknown",
    }
}

pub fn watch_status(value: WatchStatus) -> &'static str {
    match value {
        WatchStatus::Dropped => "dropped",
        WatchStatus::Completed => "completed",
        WatchStatus::Watching => "watching",
        WatchStatus::OnHold => "on_hold",
        WatchStatus::PlanToWatch => "plan_to_watch",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_watch_status_to_wire_label() {
        assert_eq!(watch_status(WatchStatus::PlanToWatch), "plan_to_watch");
        assert_eq!(watch_status(WatchStatus::OnHold), "on_hold");
    }
}
