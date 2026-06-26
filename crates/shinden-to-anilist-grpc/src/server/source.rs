use std::cmp::Ordering;

use shinden_to_anilist_core::common::AnimeList;
use tap::Tap;
use tokio_util::sync::CancellationToken;
use tonic::Status;
use tracing::{
    info,
    instrument,
    warn,
};

use super::{
    ShindenToAnilist,
    providers,
};
use crate::{
    cloudflare::ShindenCloudflareClearance,
    error::shinden_list_not_loaded,
    pb::*,
    source::SourceList,
};

impl ShindenToAnilist {
    #[instrument(skip_all, name = "app.fetch_source_list")]
    pub async fn fetch_source_list_with_progress(
        &self,
        request: FetchSourceListRequest,
        cancellation_token: CancellationToken,
        mut emit_progress: impl FnMut(SourceFetchProgress) -> Result<(), Status>,
    ) -> Result<FetchSourceListResponse, Status> {
        let provider = request.provider();
        info!(?provider, user = %request.user, "fetching source list");

        match provider {
            SourceProvider::Shinden => {
                providers::shinden::fetch_source_list(self, request, emit_progress).await
            },
            SourceProvider::AnimeZone => {
                let (source, total_entries) = providers::animezone::fetch_source_list(
                    self,
                    &request.user,
                    cancellation_token,
                    &mut emit_progress,
                )
                .await?;
                self.store_source_list(source, total_entries, emit_progress)
            },
            SourceProvider::OgladajAnime => {
                let (source, total_entries) = providers::ogladajanime::fetch_source_list(
                    self,
                    &request.user,
                    cancellation_token,
                    &mut emit_progress,
                )
                .await?;
                self.store_source_list(source, total_entries, emit_progress)
            },
            SourceProvider::Unspecified => {
                Err(Status::invalid_argument("Wybrane źródło nie jest obsługiwane."))
            },
        }
    }

    pub(super) fn store_source_list(
        &self,
        source: SourceList,
        total_entries: u64,
        mut emit_progress: impl FnMut(SourceFetchProgress) -> Result<(), Status>,
    ) -> Result<FetchSourceListResponse, Status> {
        let provider = source.provider();
        emit_progress(source_progress(
            provider,
            SourceFetchPhase::Storing,
            total_entries,
            total_entries,
            "",
        ))?;

        let total_entries = source.len() as u64;
        let source_version = self.source_list.store(source);

        emit_progress(source_progress(
            provider,
            SourceFetchPhase::Done,
            total_entries,
            total_entries,
            "",
        ))?;

        info!(source_version, total_entries, "source list fetched");
        Ok(FetchSourceListResponse {
            source_version,
            progress: Some(source_progress(
                provider,
                SourceFetchPhase::Done,
                total_entries,
                total_entries,
                "",
            )),
            done: true,
        })
    }

    #[instrument(skip_all, name = "app.set_shinden_cloudflare_clearance")]
    pub fn set_shinden_cloudflare_clearance(
        &self,
        request: SetShindenCloudflareClearanceRequest,
    ) -> Result<SetShindenCloudflareClearanceResponse, Status> {
        let clearance = request
            .clearance
            .ok_or_else(|| Status::invalid_argument("Brak danych weryfikacji Cloudflare."))?;
        info!(
            user_agent_len = clearance.user_agent.len(),
            cookie_len = clearance.cf_clearance.len(),
            domain = %clearance.domain,
            path = %clearance.path,
            "applying Shinden Cloudflare clearance"
        );

        let applied = self
            .http_clients
            .apply_shinden_cloudflare_clearance(ShindenCloudflareClearance {
                user_agent: clearance.user_agent,
                cf_clearance: clearance.cf_clearance,
                domain: clearance.domain,
                path: clearance.path,
                expires_unix_seconds: clearance.expires_unix_seconds,
                captured_at_ms: clearance.captured_at_ms,
            })
            .map_err(|error| {
                warn!(error = %error, "rejected Shinden Cloudflare clearance");
                Status::invalid_argument(error.to_string())
            })?;

        Ok(SetShindenCloudflareClearanceResponse {
            accepted: applied.accepted,
        })
    }

    #[instrument(skip_all, name = "app.get_source_ids")]
    pub async fn get_source_ids(&self, request: GetSourceIdsRequest) -> Result<GetSourceIdsResponse, Status> {
        let sorted_by = request.sorted_by();
        let guard = self.source_list.load();
        let source = guard
            .get()
            .ok_or_else(|| shinden_list_not_loaded().into_status())?;

        let source_version = guard.version();
        let ids = if sorted_by == AnimeListSortedBy::Urgency {
            source.ids_by_urgency()
        } else {
            source.ids()
        };

        info!(
            source_version,
            provider = ?source.provider(),
            ids = ids.len(),
            "source ids loaded"
        );
        Ok(GetSourceIdsResponse { source_version, ids })
    }

    #[instrument(skip_all, name = "app.get_source_full")]
    pub async fn get_source_full(
        &self,
        _request: GetSourceFullRequest,
    ) -> Result<GetSourceFullResponse, Status> {
        let guard = self.source_list.load();
        let source = guard
            .get()
            .ok_or_else(|| shinden_list_not_loaded().into_status())?;

        let source_version = guard.version();
        let entries = source.entries();
        info!(
            source_version,
            provider = ?source.provider(),
            entries = entries.len(),
            "loading full source list"
        );

        Ok(GetSourceFullResponse {
            source_version,
            entries,
        })
    }

    #[instrument(skip_all, name = "app.fetch_shinden_list")]
    pub async fn fetch_shinden_list(
        &self,
        request: FetchShindenListRequest,
    ) -> Result<FetchShindenListResponse, Status> {
        info!(shinden_id = request.id, "fetching shinden list");
        providers::shinden::fetch_legacy_list(self, request).await
    }

    #[instrument(skip_all, name = "app.get_shinden_ids")]
    pub async fn get_shinden_ids(
        &self,
        request: GetShindenIdsRequest,
    ) -> Result<GetShindenIdsResponse, Status> {
        let sorted_by = request.sorted_by();
        info!(?sorted_by, "loading shinden ids");

        let guard = self.shinden_list.load();

        let shinden = guard
            .get()
            .ok_or_else(|| shinden_list_not_loaded().into_status())?;

        let shinden_version = guard.version();
        let ids: Vec<u64> = shinden
            .iter()
            .map(|(id, entry)| (id, entry.premiere_date()))
            .collect::<Vec<_>>()
            .tap_mut(|v| {
                if sorted_by == AnimeListSortedBy::Urgency {
                    v.sort_by(|(_, date_a), (_, date_b)| match (date_a, date_b) {
                        (None, None) => Ordering::Equal,
                        (None, Some(_)) => Ordering::Less,
                        (Some(_), None) => Ordering::Greater,
                        (Some(date_a), Some(date_b)) => date_b.cmp(date_a),
                    })
                }
            })
            .into_iter()
            .map(|(id, _)| id)
            .collect();
        info!(shinden_version, ids = ids.len(), "shinden ids loaded");

        Ok(GetShindenIdsResponse { shinden_version, ids })
    }

    #[instrument(skip_all, name = "app.get_shinden_entries")]
    pub async fn get_shinden_entries(
        &self,
        request: GetShindenEntriesRequest,
    ) -> Result<GetShindenEntriesResponse, Status> {
        let requested_ids = request.ids.len();
        info!(requested_ids, ?request.ids, "loading shinden entries");

        let guard = self.shinden_list.load();

        let shinden = guard
            .get()
            .ok_or_else(|| shinden_list_not_loaded().into_status())?;

        let shinden_version = guard.version();
        let entries: Vec<ShindenEntry> = request
            .ids
            .into_iter()
            .filter_map(|id| shinden.get(id).map(Into::into))
            .collect();

        Ok(GetShindenEntriesResponse {
            shinden_version,
            entries,
        })
    }

    #[instrument(skip_all, name = "app.get_shinden_full")]
    pub async fn get_shinden_full(
        &self,
        _request: GetShindenFullRequest,
    ) -> Result<GetShindenFullResponse, Status> {
        let guard = self.shinden_list.load();

        let shinden = guard
            .get()
            .ok_or_else(|| shinden_list_not_loaded().into_status())?;

        let shinden_version = guard.version();
        let entries: Vec<ShindenEntry> = shinden.values().map(ShindenEntry::from).collect();
        info!(
            shinden_version,
            entries = entries.len(),
            "loading full shinden list"
        );

        Ok(GetShindenFullResponse {
            shinden_version,
            entries,
        })
    }
}

pub(super) fn source_progress(
    provider: SourceProvider,
    phase: SourceFetchPhase,
    current: u64,
    total: u64,
    latest_title: impl Into<String>,
) -> SourceFetchProgress {
    SourceFetchProgress {
        provider: provider.into(),
        phase: phase.into(),
        current,
        total,
        latest_title: latest_title.into(),
    }
}
