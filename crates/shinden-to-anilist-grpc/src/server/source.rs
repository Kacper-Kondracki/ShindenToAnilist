use std::cmp::Ordering;

use shinden_to_anilist_core::common::AnimeList;
use tap::Tap;
use tokio_util::sync::CancellationToken;
use tracing::{
    info,
    instrument,
};

use super::{
    ShindenToAnilist,
    providers,
};
use crate::{
    cloudflare::ShindenCloudflareClearance,
    error::{
        ServiceError,
        shinden_list_not_loaded,
    },
    pb::*,
    source::SourceList,
};

impl ShindenToAnilist {
    #[instrument(skip_all, name = "app.fetch_source_list")]
    pub async fn fetch_source_list_with_progress(
        &self,
        request: FetchSourceListRequest,
        cancellation_token: CancellationToken,
        mut emit_progress: impl FnMut(SourceFetchProgress) -> Result<(), ServiceError>,
    ) -> Result<FetchSourceListResponse, ServiceError> {
        let provider = request.provider();

        match provider {
            SourceProvider::Shinden => tokio::select! {
                biased;
                () = cancellation_token.cancelled() => {
                    Err(ServiceError::new(ErrorKind::Cancelled, "Shinden import cancelled"))
                },
                result = providers::shinden::fetch_source_list(self, request, emit_progress) => result,
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
            SourceProvider::Unspecified => Err(ServiceError::new(
                ErrorKind::InvalidSource,
                "unsupported source provider",
            )),
        }
    }

    pub(super) fn store_source_list(
        &self,
        source: SourceList,
        total_entries: u64,
        mut emit_progress: impl FnMut(SourceFetchProgress) -> Result<(), ServiceError>,
    ) -> Result<FetchSourceListResponse, ServiceError> {
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

        info!(?provider, source_version, total_entries, "source list fetched");
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
    ) -> Result<SetShindenCloudflareClearanceResponse, ServiceError> {
        let clearance = request
            .clearance
            .ok_or_else(|| ServiceError::new(ErrorKind::ClearanceInvalid, "missing Shinden clearance"))?;

        let applied = self
            .http_clients
            .apply_shinden_cloudflare_clearance(ShindenCloudflareClearance {
                user_agent: clearance.user_agent,
                cf_clearance: clearance.cf_clearance,
                domain: clearance.domain,
                path: clearance.path,
                expires_unix_seconds: clearance.expires_unix_seconds,
                captured_at_ms: clearance.captured_at_ms,
            })?;

        Ok(SetShindenCloudflareClearanceResponse {
            accepted: applied.accepted,
        })
    }

    #[instrument(skip_all, name = "app.get_source_ids")]
    pub async fn get_source_ids(
        &self,
        request: GetSourceIdsRequest,
    ) -> Result<GetSourceIdsResponse, ServiceError> {
        let sorted_by = request.sorted_by();
        let guard = self.source_list.load();
        let source = guard.get().ok_or_else(shinden_list_not_loaded)?;

        let source_version = guard.version();
        let ids = if sorted_by == AnimeListSortedBy::Urgency {
            source.ids_by_urgency()
        } else {
            source.ids()
        };

        Ok(GetSourceIdsResponse { source_version, ids })
    }

    #[instrument(skip_all, name = "app.get_source_full")]
    pub async fn get_source_full(
        &self,
        _request: GetSourceFullRequest,
    ) -> Result<GetSourceFullResponse, ServiceError> {
        let guard = self.source_list.load();
        let source = guard.get().ok_or_else(shinden_list_not_loaded)?;

        let source_version = guard.version();
        let entries = source.entries();

        Ok(GetSourceFullResponse {
            source_version,
            entries,
        })
    }

    #[instrument(skip_all, name = "app.fetch_shinden_list")]
    pub async fn fetch_shinden_list(
        &self,
        request: FetchShindenListRequest,
    ) -> Result<FetchShindenListResponse, ServiceError> {
        providers::shinden::fetch_legacy_list(self, request).await
    }

    #[instrument(skip_all, name = "app.get_shinden_ids")]
    pub async fn get_shinden_ids(
        &self,
        request: GetShindenIdsRequest,
    ) -> Result<GetShindenIdsResponse, ServiceError> {
        let sorted_by = request.sorted_by();

        let guard = self.shinden_list.load();

        let shinden = guard.get().ok_or_else(shinden_list_not_loaded)?;

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

        Ok(GetShindenIdsResponse { shinden_version, ids })
    }

    #[instrument(skip_all, name = "app.get_shinden_entries")]
    pub async fn get_shinden_entries(
        &self,
        request: GetShindenEntriesRequest,
    ) -> Result<GetShindenEntriesResponse, ServiceError> {
        let guard = self.shinden_list.load();

        let shinden = guard.get().ok_or_else(shinden_list_not_loaded)?;

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
    ) -> Result<GetShindenFullResponse, ServiceError> {
        let guard = self.shinden_list.load();

        let shinden = guard.get().ok_or_else(shinden_list_not_loaded)?;

        let shinden_version = guard.version();
        let entries: Vec<ShindenEntry> = shinden.values().map(ShindenEntry::from).collect();

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
