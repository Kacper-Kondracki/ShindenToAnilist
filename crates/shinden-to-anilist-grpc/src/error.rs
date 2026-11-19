use std::{
    error::Error,
    fmt,
    io,
    path::Path,
};

#[cfg(feature = "grpc")]
use prost::Message;
#[cfg(feature = "grpc")]
use prost_types::Any;
use shinden_to_anilist_core::{
    database::DatabaseError,
    exporter::xml::XmlExportError,
    providers::{
        animezone::AnimeZoneError,
        ogladajanime::OgladajAnimeError,
        shinden::ShindenError,
    },
};
#[cfg(feature = "grpc")]
use tonic::{
    Code,
    Status,
};
use tracing::warn;

use crate::{
    cloudflare::ClearanceError,
    pb::{
        AppError,
        ErrorKind,
    },
};

#[derive(Debug)]
pub struct ServiceError {
    kind: ErrorKind,
    context: String,
    source: Option<Box<dyn Error + Send + Sync>>,
    http_status: Option<u32>,
}

impl fmt::Display for ServiceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} ({})", self.context, self.kind.as_str_name())
    }
}

impl Error for ServiceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source.as_deref().map(|error| error as &dyn Error)
    }
}

impl ServiceError {
    pub fn new(kind: ErrorKind, context: impl Into<String>) -> Self {
        Self {
            kind,
            context: context.into(),
            source: None,
            http_status: None,
        }
    }

    pub fn caused_by(
        kind: ErrorKind,
        context: impl Into<String>,
        source: impl Error + Send + Sync + 'static,
    ) -> Self {
        Self {
            source: Some(Box::new(source)),
            ..Self::new(kind, context)
        }
    }

    pub(crate) fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = context.into();
        self
    }

    pub fn report(&self) -> AppError {
        if self.kind != ErrorKind::Cancelled {
            let mut causes = Vec::new();
            let mut current = self.source();
            while let Some(error) = current {
                causes.push(error.to_string());
                current = error.source();
            }
            warn!(kind = ?self.kind, context = %self.context, http_status = self.http_status, ?causes, "application operation failed");
        }

        AppError {
            kind: self.kind.into(),
            http_status: self.http_status,
        }
    }

    #[cfg(feature = "grpc")]
    pub(crate) fn into_status(self) -> Status {
        let code = self.grpc_code();
        let error = self.report();
        let message = self.kind.as_str_name();
        let status = RpcStatus {
            code: code as i32,
            message: message.to_owned(),
            details: vec![Any {
                type_url: "type.googleapis.com/shinden_to_anilist.v1.AppError".to_owned(),
                value: error.encode_to_vec(),
            }],
        };
        Status::with_details(code, message, status.encode_to_vec().into())
    }

    #[cfg(feature = "grpc")]
    fn grpc_code(&self) -> Code {
        use ErrorKind::*;
        match self.kind {
            InvalidSource | InvalidSourceUser | ClearanceInvalid => Code::InvalidArgument,
            Cancelled => Code::Cancelled,
            DeadlineExceeded => Code::DeadlineExceeded,
            SourceListNotFound => Code::NotFound,
            CloudflareChallenge | ClearanceExpired | ShindenListNotLoaded | DatabaseNotLoaded
            | ExportOutOfIndex => Code::FailedPrecondition,
            SourceListPrivate | DatabasePermissionDenied | ExportPermissionDenied => Code::PermissionDenied,
            DatabaseDigestMismatch => Code::DataLoss,
            ServiceUnavailable => Code::Unavailable,
            DatabaseMissingReleaseAsset => Code::FailedPrecondition,
            DatabaseHttp
            | ShindenHttp
            | AnimeZoneHttp
            | OgladajAnimeHttp
            | AnimeZoneRetryExhausted
            | OgladajAnimeRetryExhausted => match self.http_status {
                Some(429) => Code::ResourceExhausted,
                None | Some(500..=599) => Code::Unavailable,
                _ => Code::Internal,
            },
            _ => Code::Internal,
        }
    }

    fn with_http_status(mut self, status: Option<reqwest::StatusCode>) -> Self {
        self.http_status = status.map(|status| u32::from(status.as_u16()));
        self
    }
}

#[cfg(feature = "grpc")]
// Wire representation of google.rpc.Status, including the standard Any detail.
#[derive(Message)]
struct RpcStatus {
    #[prost(int32, tag = "1")]
    code: i32,
    #[prost(string, tag = "2")]
    message: String,
    #[prost(message, repeated, tag = "3")]
    details: Vec<Any>,
}

#[cfg(feature = "grpc")]
impl From<ServiceError> for Status {
    fn from(error: ServiceError) -> Self { error.into_status() }
}

pub(crate) fn shinden_list_not_loaded() -> ServiceError {
    ServiceError::new(ErrorKind::ShindenListNotLoaded, "source list is not loaded")
}

pub(crate) fn database_not_loaded() -> ServiceError {
    ServiceError::new(ErrorKind::DatabaseNotLoaded, "database is not loaded")
}

pub(crate) fn database_sidecar_io_error(error: io::Error, path: &Path, operation: &str) -> ServiceError {
    let kind = if error.kind() == io::ErrorKind::PermissionDenied {
        ErrorKind::DatabasePermissionDenied
    } else {
        ErrorKind::DatabaseSidecarIo
    };
    ServiceError::caused_by(
        kind,
        format!("{operation} database metadata at {}", path.display()),
        error,
    )
}

pub(crate) fn database_sidecar_json_error(error: serde_json::Error, path: &Path) -> ServiceError {
    ServiceError::caused_by(
        ErrorKind::DatabaseSidecarJson,
        format!("process database metadata at {}", path.display()),
        error,
    )
}

pub(crate) fn export_xml_io_error(error: io::Error, path: &Path, operation: &str) -> ServiceError {
    let kind = if error.kind() == io::ErrorKind::PermissionDenied {
        ErrorKind::ExportPermissionDenied
    } else {
        ErrorKind::ExportIo
    };
    ServiceError::caused_by(kind, format!("{operation} export at {}", path.display()), error)
}

impl From<DatabaseError> for ServiceError {
    fn from(error: DatabaseError) -> Self {
        let (kind, status) = match &error {
            DatabaseError::Io(source) if source.kind() == io::ErrorKind::PermissionDenied => {
                (ErrorKind::DatabasePermissionDenied, None)
            },
            DatabaseError::Io(_) => (ErrorKind::DatabaseIo, None),
            DatabaseError::Json(_) => (ErrorKind::DatabaseJson, None),
            DatabaseError::Request(source) => {
                (request_kind(ErrorKind::DatabaseHttp, source), source.status())
            },
            DatabaseError::Empty => (ErrorKind::DatabaseEmpty, None),
            DatabaseError::MissingReleaseAsset { .. } => (ErrorKind::DatabaseMissingReleaseAsset, None),
            DatabaseError::DigestMismatch { .. } => (ErrorKind::DatabaseDigestMismatch, None),
        };
        Self::caused_by(kind, "access database", error).with_http_status(status)
    }
}

impl From<ShindenError> for ServiceError {
    fn from(error: ShindenError) -> Self {
        let (kind, status) = match &error {
            ShindenError::Io(_) => (ErrorKind::ShindenIo, None),
            ShindenError::Json(_) => (ErrorKind::ShindenJson, None),
            ShindenError::Request(source) => (request_kind(ErrorKind::ShindenHttp, source), source.status()),
            ShindenError::Http {
                status, cf_mitigated, ..
            } if cf_mitigated.eq_ignore_ascii_case("challenge") => {
                (ErrorKind::CloudflareChallenge, Some(*status))
            },
            ShindenError::Http {
                status: reqwest::StatusCode::NOT_FOUND,
                ..
            } => (
                ErrorKind::SourceListNotFound,
                Some(reqwest::StatusCode::NOT_FOUND),
            ),
            ShindenError::Http { status, .. } => (ErrorKind::ShindenHttp, Some(*status)),
            ShindenError::Shinden(_) => (ErrorKind::ShindenApi, None),
        };
        Self::caused_by(kind, "fetch Shinden list", error).with_http_status(status)
    }
}

impl From<AnimeZoneError> for ServiceError {
    fn from(error: AnimeZoneError) -> Self {
        let (kind, status) = match &error {
            AnimeZoneError::Io(_) => (ErrorKind::AnimeZoneIo, None),
            AnimeZoneError::Request(source) => {
                (request_kind(ErrorKind::AnimeZoneHttp, source), source.status())
            },
            AnimeZoneError::RetryExhausted { source, .. } => {
                (ErrorKind::AnimeZoneRetryExhausted, source.status())
            },
            AnimeZoneError::Parse { .. } => (ErrorKind::AnimeZoneParse, None),
        };
        Self::caused_by(kind, "fetch AnimeZone list", error).with_http_status(status)
    }
}

impl From<OgladajAnimeError> for ServiceError {
    fn from(error: OgladajAnimeError) -> Self {
        let (kind, status) = match &error {
            OgladajAnimeError::Io(_) => (ErrorKind::OgladajAnimeIo, None),
            OgladajAnimeError::Request(source) => {
                (request_kind(ErrorKind::OgladajAnimeHttp, source), source.status())
            },
            OgladajAnimeError::RetryExhausted { source, .. } => {
                (ErrorKind::OgladajAnimeRetryExhausted, source.status())
            },
            OgladajAnimeError::PrivateList { .. } => (ErrorKind::SourceListPrivate, None),
            OgladajAnimeError::Parse { .. } => (ErrorKind::OgladajAnimeParse, None),
        };
        Self::caused_by(kind, "fetch Oglądaj Anime list", error).with_http_status(status)
    }
}

impl From<XmlExportError> for ServiceError {
    fn from(error: XmlExportError) -> Self {
        let kind = match error {
            XmlExportError::Xml(_) => ErrorKind::ExportXml,
            XmlExportError::OutOfIndex(..) => ErrorKind::ExportOutOfIndex,
        };
        Self::caused_by(kind, "write export XML", error)
    }
}

impl From<ClearanceError> for ServiceError {
    fn from(error: ClearanceError) -> Self {
        let kind = match error {
            ClearanceError::Expired => ErrorKind::ClearanceExpired,
            ClearanceError::Client(_) | ClearanceError::InvalidCookieUrl => ErrorKind::Unknown,
            _ => ErrorKind::ClearanceInvalid,
        };
        Self::caused_by(kind, "apply Shinden verification", error)
    }
}

fn request_kind(default: ErrorKind, source: &reqwest::Error) -> ErrorKind {
    if source.is_timeout() {
        ErrorKind::DeadlineExceeded
    } else {
        default
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "grpc")]
    #[test]
    fn grpc_encodes_status_and_app_error_separately() {
        let failure = database_not_loaded().into_status();
        let status = RpcStatus::decode(failure.details()).unwrap();
        assert_eq!(failure.code(), Code::FailedPrecondition);
        assert_eq!(status.code, failure.code() as i32);
        let [detail] = status.details.as_slice() else {
            panic!("expected one detail")
        };
        assert_eq!(
            detail.type_url,
            "type.googleapis.com/shinden_to_anilist.v1.AppError"
        );
        let error = AppError::decode(detail.value.as_slice()).unwrap();
        assert_eq!(error.kind(), ErrorKind::DatabaseNotLoaded);
    }

    #[test]
    fn cloudflare_is_classified_from_the_header() {
        for (header, expected) in [
            ("challenge", ErrorKind::CloudflareChallenge),
            ("", ErrorKind::ShindenHttp),
        ] {
            let failure = ServiceError::from(ShindenError::Http {
                status: reqwest::StatusCode::FORBIDDEN,
                url: "https://lista.shinden.pl/".to_owned(),
                content_type: "text/html".to_owned(),
                cf_mitigated: header.to_owned(),
            });
            assert_eq!(failure.kind, expected);
            assert_eq!(failure.http_status, Some(403));
        }
    }

    #[test]
    fn local_permission_errors_keep_the_cause_and_operation() {
        let failure = export_xml_io_error(
            io::ErrorKind::PermissionDenied.into(),
            Path::new("example.xml"),
            "create",
        );
        assert_eq!(failure.kind, ErrorKind::ExportPermissionDenied);
        assert_eq!(failure.context, "create export at example.xml");
        assert!(failure.source.unwrap().is::<io::Error>());
    }
}
