use std::{
    io,
    path::Path,
};

use prost::Message;
use shinden_to_anilist_core::{
    database::DatabaseError,
    exporter::xml::XmlExportError,
    providers::{
        animezone::AnimeZoneError,
        ogladajanime::OgladajAnimeError,
        shinden::ShindenError,
    },
};
use tonic::{
    Code,
    Status,
};
use tracing::warn;

use crate::pb::{
    AppError,
    DigestMismatchError,
    ErrorKind,
    HttpError,
    IoError,
    JsonError,
    MissingReleaseAssetError,
    OutOfIndexError,
    ShindenApiError,
    UnloadedResourceError,
    XmlExportError as PbXmlExportError,
    app_error::Details,
};

impl AppError {
    pub fn into_status(self) -> Status {
        warn!(
            kind = ?self.kind(),
            message = %self.message,
            "returning application error"
        );
        Status::with_details(Code::Internal, self.message.clone(), self.encode_to_vec().into())
    }
}

pub trait IntoStatus {
    fn into_status(self) -> Status;
}

impl IntoStatus for DatabaseError {
    fn into_status(self) -> Status { database_error(self).into_status() }
}

impl IntoStatus for ShindenError {
    fn into_status(self) -> Status { shinden_error(self).into_status() }
}

impl IntoStatus for AnimeZoneError {
    fn into_status(self) -> Status { animezone_error(self).into_status() }
}

impl IntoStatus for OgladajAnimeError {
    fn into_status(self) -> Status { ogladajanime_error(self).into_status() }
}

impl IntoStatus for XmlExportError {
    fn into_status(self) -> Status { xml_export_error(self).into_status() }
}

pub fn shinden_list_not_loaded() -> AppError {
    unloaded_resource_error(
        ErrorKind::ShindenListNotLoaded,
        "Lista użytkownika nie została jeszcze wczytana.",
        "shinden_list",
    )
}

pub fn database_not_loaded() -> AppError {
    unloaded_resource_error(
        ErrorKind::DatabaseNotLoaded,
        "Baza danych nie została jeszcze wczytana.",
        "database",
    )
}

pub fn database_sidecar_io_error(
    error: io::Error,
    path: impl AsRef<Path>,
    operation: &'static str,
) -> AppError {
    io_error(
        ErrorKind::DatabaseSidecarIo,
        error.to_string(),
        Some(path.as_ref()),
        operation,
        Some(error.kind()),
    )
}

pub fn database_sidecar_json_error(error: serde_json::Error, path: impl AsRef<Path>) -> AppError {
    json_error(ErrorKind::DatabaseSidecarJson, error, Some(path.as_ref()))
}

pub fn export_xml_io_error(error: io::Error, path: impl AsRef<Path>, operation: &'static str) -> AppError {
    io_error(
        ErrorKind::ExportIo,
        error.to_string(),
        Some(path.as_ref()),
        operation,
        Some(error.kind()),
    )
}

pub fn database_error(error: DatabaseError) -> AppError {
    match error {
        DatabaseError::Io(error) => io_error(
            ErrorKind::DatabaseIo,
            error.to_string(),
            None,
            "database",
            Some(error.kind()),
        ),
        DatabaseError::Json(error) => json_error(ErrorKind::DatabaseJson, error, None),
        DatabaseError::Request(error) => http_error(ErrorKind::DatabaseHttp, error),
        DatabaseError::Empty => simple_error(
            ErrorKind::DatabaseEmpty,
            "Plik bazy danych jest pusty i nie można go wczytać.",
        ),
        DatabaseError::MissingReleaseAsset { asset } => AppError {
            kind: ErrorKind::DatabaseMissingReleaseAsset.into(),
            message: format!("Najnowsze wydanie anime-offline-database nie zawiera pliku {asset}."),
            details: Some(Details::MissingReleaseAsset(MissingReleaseAssetError {
                asset: asset.to_string(),
            })),
        },
        DatabaseError::DigestMismatch { expected, actual } => AppError {
            kind: ErrorKind::DatabaseDigestMismatch.into(),
            message: "Pobrana baza danych ma nieprawidłową sumę kontrolną. Spróbuj pobrać ją ponownie."
                .to_string(),
            details: Some(Details::DigestMismatch(DigestMismatchError { expected, actual })),
        },
    }
}

pub fn shinden_error(error: ShindenError) -> AppError {
    match error {
        ShindenError::Io(error) => io_error(
            ErrorKind::ShindenIo,
            error.to_string(),
            None,
            "shinden",
            Some(error.kind()),
        ),
        ShindenError::Json(error) => json_error(ErrorKind::ShindenJson, error, None),
        ShindenError::Request(error) => http_error(ErrorKind::ShindenHttp, error),
        ShindenError::Shinden(message) => AppError {
            kind: ErrorKind::ShindenApi.into(),
            message: shinden_api_user_message(&message),
            details: Some(Details::ShindenApi(ShindenApiError { message })),
        },
    }
}

pub fn animezone_error(error: AnimeZoneError) -> AppError {
    match error {
        AnimeZoneError::Io(error) => io_error(
            ErrorKind::AnimeZoneIo,
            error.to_string(),
            None,
            "animezone",
            Some(error.kind()),
        ),
        AnimeZoneError::Request(error) => http_error(ErrorKind::AnimeZoneHttp, error),
        AnimeZoneError::RetryExhausted {
            path,
            attempts,
            source,
        } => AppError {
            kind: ErrorKind::AnimeZoneRetryExhausted.into(),
            message: format!("Nie udało się pobrać danych z AnimeZone po {attempts} próbach."),
            details: Some(Details::Http(HttpError {
                message: source.to_string(),
                url: path,
                status: source
                    .status()
                    .map(|status| status.as_u16().into())
                    .unwrap_or_default(),
            })),
        },
        AnimeZoneError::Parse { path, message } => AppError {
            kind: ErrorKind::AnimeZoneParse.into(),
            message: animezone_parse_user_message(&path, &message),
            details: None,
        },
    }
}

pub fn ogladajanime_error(error: OgladajAnimeError) -> AppError {
    match error {
        OgladajAnimeError::Io(error) => io_error(
            ErrorKind::OgladajAnimeIo,
            error.to_string(),
            None,
            "ogladajanime",
            Some(error.kind()),
        ),
        OgladajAnimeError::Request(error) => http_error(ErrorKind::OgladajAnimeHttp, error),
        OgladajAnimeError::RetryExhausted {
            path,
            attempts,
            source,
        } => AppError {
            kind: ErrorKind::OgladajAnimeRetryExhausted.into(),
            message: format!("Nie udało się pobrać danych z Oglądaj Anime po {attempts} próbach."),
            details: Some(Details::Http(HttpError {
                message: source.to_string(),
                url: path,
                status: source
                    .status()
                    .map(|status| status.as_u16().into())
                    .unwrap_or_default(),
            })),
        },
        OgladajAnimeError::Parse { path, message } => AppError {
            kind: ErrorKind::OgladajAnimeParse.into(),
            message: ogladajanime_parse_user_message(&path, &message),
            details: None,
        },
    }
}

pub fn xml_export_error(error: XmlExportError) -> AppError {
    match error {
        XmlExportError::Xml(error) => AppError {
            kind: ErrorKind::ExportXml.into(),
            message: "Nie udało się utworzyć pliku XML eksportu.".to_string(),
            details: Some(Details::XmlExport(PbXmlExportError {
                message: error.to_string(),
            })),
        },
        XmlExportError::OutOfIndex(id, collection) => AppError {
            kind: ErrorKind::ExportOutOfIndex.into(),
            message: "Eksport odwołuje się do wpisu, którego nie ma na wczytanej liście.".to_string(),
            details: Some(Details::OutOfIndex(OutOfIndexError {
                id,
                collection: collection.to_string(),
            })),
        },
    }
}

fn unloaded_resource_error(
    kind: ErrorKind,
    message: impl Into<String>,
    resource: impl Into<String>,
) -> AppError {
    AppError {
        kind: kind.into(),
        message: message.into(),
        details: Some(Details::UnloadedResource(UnloadedResourceError {
            resource: resource.into(),
        })),
    }
}

fn simple_error(kind: ErrorKind, message: impl Into<String>) -> AppError {
    AppError {
        kind: kind.into(),
        message: message.into(),
        details: None,
    }
}

fn io_error(
    kind: ErrorKind,
    message: String,
    path: Option<&Path>,
    operation: impl Into<String>,
    os_error_kind: Option<io::ErrorKind>,
) -> AppError {
    let operation = operation.into();
    AppError {
        kind: kind.into(),
        message: io_user_message(kind, path, &operation),
        details: Some(Details::Io(IoError {
            message,
            path: path
                .map(|path| path.to_string_lossy().into_owned())
                .unwrap_or_default(),
            operation,
            os_error_kind: os_error_kind.map(|kind| format!("{kind:?}")).unwrap_or_default(),
        })),
    }
}

fn json_error(kind: ErrorKind, error: serde_json::Error, path: Option<&Path>) -> AppError {
    let message = error.to_string();
    AppError {
        kind: kind.into(),
        message: json_user_message(kind, path, error.line(), error.column()),
        details: Some(Details::Json(JsonError {
            message,
            path: path
                .map(|path| path.to_string_lossy().into_owned())
                .unwrap_or_default(),
            line: error.line() as u64,
            column: error.column() as u64,
            category: format!("{:?}", error.classify()),
        })),
    }
}

fn http_error(kind: ErrorKind, error: reqwest::Error) -> AppError {
    let message = error.to_string();
    let user_message = http_user_message(kind, &error);
    AppError {
        kind: kind.into(),
        message: user_message,
        details: Some(Details::Http(HttpError {
            message,
            url: error.url().map(ToString::to_string).unwrap_or_default(),
            status: error
                .status()
                .map(|status| status.as_u16().into())
                .unwrap_or_default(),
        })),
    }
}

fn io_user_message(kind: ErrorKind, path: Option<&Path>, operation: &str) -> String {
    let target = path
        .map(|path| format!(" ({})", path.to_string_lossy()))
        .unwrap_or_default();

    match kind {
        ErrorKind::DatabaseIo | ErrorKind::DatabaseSidecarIo => {
            format!("Nie udało się odczytać lub zapisać bazy danych{target}.")
        },
        ErrorKind::ExportIo => format!("Nie udało się zapisać pliku eksportu{target}."),
        ErrorKind::ShindenIo => "Nie udało się odczytać danych Shindena.".to_string(),
        ErrorKind::AnimeZoneIo => "Nie udało się odczytać danych AnimeZone.".to_string(),
        ErrorKind::OgladajAnimeIo => "Nie udało się odczytać danych Oglądaj Anime.".to_string(),
        _ => format!("Nie udało się wykonać operacji „{operation}”{target}."),
    }
}

fn json_user_message(kind: ErrorKind, path: Option<&Path>, line: usize, column: usize) -> String {
    let location = if line > 0 || column > 0 {
        format!(" Linia {line}, kolumna {column}.")
    } else {
        String::new()
    };
    let target = path
        .map(|path| format!(" ({})", path.to_string_lossy()))
        .unwrap_or_default();

    match kind {
        ErrorKind::DatabaseJson | ErrorKind::DatabaseSidecarJson => {
            format!("Plik bazy danych ma nieprawidłowy format JSON{target}.{location}")
        },
        ErrorKind::ShindenJson => format!("Shinden zwrócił nieprawidłowe dane JSON.{location}"),
        _ => format!("Nie udało się odczytać danych JSON{target}.{location}"),
    }
}

fn http_user_message(kind: ErrorKind, error: &reqwest::Error) -> String {
    let status = error
        .status()
        .map(|status| format!(" Kod odpowiedzi: {status}."))
        .unwrap_or_default();

    match kind {
        ErrorKind::DatabaseHttp => {
            format!("Nie udało się pobrać informacji o bazie danych.{status}")
        },
        ErrorKind::ShindenHttp => format!("Nie udało się połączyć z Shindenem.{status}"),
        ErrorKind::AnimeZoneHttp => format!("Nie udało się połączyć z AnimeZone.{status}"),
        ErrorKind::OgladajAnimeHttp => {
            format!("Nie udało się połączyć z Oglądaj Anime.{status}")
        },
        _ => format!("Nie udało się wykonać żądania HTTP.{status}"),
    }
}

fn shinden_api_user_message(message: &str) -> String {
    let normalized = message.to_ascii_lowercase();
    if normalized.contains("private") || normalized.contains("prywat") {
        return "Lista użytkownika Shinden jest prywatna albo niedostępna.".to_string();
    }

    if normalized.contains("not found") || normalized.contains("nie znaleziono") {
        return "Nie znaleziono listy użytkownika Shinden.".to_string();
    }

    format!("Shinden zwrócił błąd: {message}")
}

fn animezone_parse_user_message(_path: &str, message: &str) -> String {
    let normalized = message.to_ascii_lowercase();
    if normalized.contains("missing title link") && normalized.contains("anime card") {
        return "Nie udało się odczytać listy AnimeZone. Sprawdź nazwę użytkownika albo widoczność listy."
            .to_string();
    }

    "Nie udało się odczytać listy AnimeZone. Strona mogła zmienić format danych.".to_string()
}

fn ogladajanime_parse_user_message(_path: &str, message: &str) -> String {
    let normalized = message.to_ascii_lowercase();
    if normalized.contains("private") || normalized.contains("prywat") {
        return "Lista użytkownika Oglądaj Anime jest prywatna albo niedostępna.".to_string();
    }

    "Nie udało się odczytać listy Oglądaj Anime. Strona mogła zmienić format danych.".to_string()
}

#[cfg(test)]
mod tests {
    use std::io;

    use prost::Message;
    use shinden_to_anilist_core::{
        database::DatabaseError,
        providers::shinden::ShindenError,
    };
    use tonic::Code;

    use super::*;
    use crate::pb::app_error::Details;

    #[test]
    fn database_errors_map_to_expected_kinds() {
        let cases = [
            (
                database_error(DatabaseError::Io(io::Error::from(io::ErrorKind::NotFound))),
                ErrorKind::DatabaseIo,
            ),
            (
                database_error(DatabaseError::Json(
                    serde_json::from_str::<serde_json::Value>("{").unwrap_err(),
                )),
                ErrorKind::DatabaseJson,
            ),
            (database_error(DatabaseError::Empty), ErrorKind::DatabaseEmpty),
            (
                database_error(DatabaseError::MissingReleaseAsset {
                    asset: "database.jsonl.zst",
                }),
                ErrorKind::DatabaseMissingReleaseAsset,
            ),
            (
                database_error(DatabaseError::DigestMismatch {
                    expected: "expected".to_string(),
                    actual: "actual".to_string(),
                }),
                ErrorKind::DatabaseDigestMismatch,
            ),
        ];

        for (error, kind) in cases {
            assert_eq!(error.kind(), kind);
        }
    }

    #[test]
    fn shinden_errors_map_to_expected_kinds() {
        let cases = [
            (
                shinden_error(ShindenError::Io(io::Error::from(io::ErrorKind::PermissionDenied))),
                ErrorKind::ShindenIo,
            ),
            (
                shinden_error(ShindenError::Json(
                    serde_json::from_str::<serde_json::Value>("{").unwrap_err(),
                )),
                ErrorKind::ShindenJson,
            ),
            (
                shinden_error(ShindenError::Shinden("private list".to_string())),
                ErrorKind::ShindenApi,
            ),
        ];

        for (error, kind) in cases {
            assert_eq!(error.kind(), kind);
        }
    }

    #[tokio::test]
    async fn request_errors_map_to_expected_kinds() {
        let database_request_error = reqwest::Client::new()
            .get("http://[::1")
            .send()
            .await
            .unwrap_err();
        let shinden_request_error = reqwest::Client::new()
            .get("http://[::1")
            .send()
            .await
            .unwrap_err();

        assert_eq!(
            database_error(DatabaseError::Request(database_request_error)).kind(),
            ErrorKind::DatabaseHttp
        );
        assert_eq!(
            shinden_error(ShindenError::Request(shinden_request_error)).kind(),
            ErrorKind::ShindenHttp
        );
    }

    #[test]
    fn unloaded_errors_encode_as_internal_status_details() {
        let status = shinden_list_not_loaded().into_status();

        assert_eq!(status.code(), Code::Internal);

        let error = AppError::decode(status.details()).expect("status details should decode as AppError");
        assert_eq!(error.kind(), ErrorKind::ShindenListNotLoaded);
        assert!(matches!(error.details, Some(Details::UnloadedResource(_))));

        let status = database_not_loaded().into_status();
        let error = AppError::decode(status.details()).expect("status details should decode as AppError");
        assert_eq!(status.code(), Code::Internal);
        assert_eq!(error.kind(), ErrorKind::DatabaseNotLoaded);
    }

    #[test]
    fn sidecar_errors_include_typed_details() {
        let error = database_sidecar_io_error(
            io::Error::from(io::ErrorKind::PermissionDenied),
            "/tmp/database.info.json",
            "open",
        );

        assert_eq!(error.kind(), ErrorKind::DatabaseSidecarIo);
        match error.details {
            Some(Details::Io(details)) => {
                assert_eq!(details.path, "/tmp/database.info.json");
                assert_eq!(details.operation, "open");
                assert_eq!(details.os_error_kind, "PermissionDenied");
            },
            details => panic!("expected io details, got {details:?}"),
        }
    }
}
