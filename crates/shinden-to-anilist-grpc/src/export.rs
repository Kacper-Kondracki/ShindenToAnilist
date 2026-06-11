use std::{
    fs::{
        self,
        File,
    },
    io::{
        self,
        BufWriter,
        Write,
    },
    path::{
        Path,
        PathBuf,
    },
};

use shinden_to_anilist_core::{
    common::AnimeId,
    exporter::{
        ExportExt,
        xml::{
            XmlExportError,
            XmlExporter,
        },
    },
    providers::shinden::ShindenList,
};

use crate::error::{
    IntoStatus,
    export_xml_io_error,
};

pub(crate) fn export_xml_to_path(
    shinden: &ShindenList,
    matches: impl Iterator<Item = (AnimeId, AnimeId)>,
    path: impl AsRef<Path>,
) -> Result<(), tonic::Status> {
    let path = path.as_ref();
    let temp_path =
        unique_temp_path(path).map_err(|err| export_xml_io_error(err, path, "temp_path").into_status())?;
    let mut temp_file = File::options()
        .write(true)
        .create_new(true)
        .open(&temp_path)
        .map_err(|err| export_xml_io_error(err, &temp_path, "create").into_status())?;

    let result = write_xml(shinden, matches, &mut temp_file)
        .and_then(|_| temp_file.sync_all().map_err(ExportWriteError::Io))
        .map_err(|err| match err {
            ExportWriteError::Xml(err) => err.into_status(),
            ExportWriteError::Io(err) => export_xml_io_error(err, &temp_path, "write").into_status(),
        })
        .and_then(|_| {
            fs::rename(&temp_path, path).map_err(|err| export_xml_io_error(err, path, "rename").into_status())
        });

    if result.is_err() {
        let _ = fs::remove_file(&temp_path);
    }

    result
}

fn write_xml(
    shinden: &ShindenList,
    matches: impl Iterator<Item = (AnimeId, AnimeId)>,
    temp_file: &mut File,
) -> Result<(), ExportWriteError> {
    let mut writer = BufWriter::new(temp_file);
    shinden
        .export(&XmlExporter {}, matches, &mut writer)
        .map_err(ExportWriteError::Xml)?;
    writer.flush().map_err(ExportWriteError::Io)
}

fn unique_temp_path(path: &Path) -> io::Result<PathBuf> {
    let file_name = path.file_name().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "export path must include a file name",
        )
    })?;
    let parent = path.parent().filter(|path| !path.as_os_str().is_empty());

    for attempt in 0..1000 {
        let mut temp_file_name = file_name.to_os_string();
        temp_file_name.push(format!(".tmp.{}.{}", std::process::id(), attempt));

        let temp_path = parent
            .map(|parent| parent.join(&temp_file_name))
            .unwrap_or_else(|| PathBuf::from(&temp_file_name));

        if !temp_path.exists() {
            return Ok(temp_path);
        }
    }

    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "could not allocate temporary export path",
    ))
}

enum ExportWriteError {
    Xml(XmlExportError),
    Io(io::Error),
}
