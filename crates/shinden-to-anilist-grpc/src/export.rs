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
    common::{
        AnimeId,
        AnimeList,
        ExportView,
    },
    exporter::{
        ExportExt,
        xml::{
            XmlExportError,
            XmlExporter,
        },
    },
};

use crate::{
    error::{
        IntoStatus,
        export_xml_io_error,
    },
    source::SourceList,
};

pub(crate) fn export_xml_to_path(
    source: &SourceList,
    matches: impl Iterator<Item = (AnimeId, AnimeId)>,
    path: impl AsRef<Path>,
) -> Result<(), tonic::Status> {
    let path = path.as_ref();
    let (mut temp_file, temp_path) = create_unique_temp_file(path)
        .map_err(|err| export_xml_io_error(err, path, "create").into_status())?;

    let result = write_source_xml(source, matches, &mut temp_file)
        .and_then(|_| temp_file.sync_all().map_err(ExportWriteError::Io))
        .map_err(|err| match err {
            ExportWriteError::Xml(err) => err.into_status(),
            ExportWriteError::Io(err) => export_xml_io_error(err, &temp_path, "write").into_status(),
        })
        .and_then(|_| {
            fs::rename(&temp_path, path)
                .and_then(|_| sync_parent_dir(path))
                .map_err(|err| export_xml_io_error(err, path, "rename").into_status())
        });

    if result.is_err() {
        let _ = fs::remove_file(&temp_path);
    }

    result
}

fn write_source_xml(
    source: &SourceList,
    matches: impl Iterator<Item = (AnimeId, AnimeId)>,
    temp_file: &mut File,
) -> Result<(), ExportWriteError> {
    match source {
        SourceList::Shinden(list) => write_xml(list, matches, temp_file),
        SourceList::AnimeZone(list) => write_xml(list, matches, temp_file),
        SourceList::OgladajAnime(list) => write_xml(list, matches, temp_file),
    }
}

fn write_xml<E>(
    source: &impl AnimeList<Entry = E>,
    matches: impl Iterator<Item = (AnimeId, AnimeId)>,
    temp_file: &mut File,
) -> Result<(), ExportWriteError>
where
    E: ExportView + Send + Sync,
{
    let mut writer = BufWriter::new(temp_file);
    source
        .export(&XmlExporter {}, matches, &mut writer)
        .map_err(ExportWriteError::Xml)?;
    writer.flush().map_err(ExportWriteError::Io)
}

pub(crate) fn create_unique_temp_file(path: &Path) -> io::Result<(File, PathBuf)> {
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

        match File::options().write(true).create_new(true).open(&temp_path) {
            Ok(file) => return Ok((file, temp_path)),
            Err(err) if err.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(err) => return Err(err),
        }
    }

    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "could not allocate temporary export path",
    ))
}

#[cfg(unix)]
pub(crate) fn sync_parent_dir(path: &Path) -> io::Result<()> {
    let Some(parent) = path.parent().filter(|path| !path.as_os_str().is_empty()) else {
        return Ok(());
    };

    File::open(parent)?.sync_all()
}

#[cfg(not(unix))]
pub(crate) fn sync_parent_dir(_path: &Path) -> io::Result<()> { Ok(()) }

enum ExportWriteError {
    Xml(XmlExportError),
    Io(io::Error),
}
