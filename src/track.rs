use std::{io, ops::Deref, fs, path::{Path, PathBuf}, rc::Rc, sync::atomic::{AtomicUsize, Ordering}, time::Duration};

use lofty::{file::{AudioFile, TaggedFileExt}, tag::{Accessor, TagType}};
use thiserror::Error;

use crate::cache::Cache;

// Static
static TRACK_ID: AtomicUsize = AtomicUsize::new(0);

// Errors
#[derive(Debug, Error)]
pub enum TrackDataError {
    #[error("\"{0}\" no such file")]
    NotFound(PathBuf),
    #[error("I/O error: {0}")]
    Io(io::Error),
    #[error("[lofty] Read audio error: {0}")]
    Read(lofty::error::LoftyError)
}

/// Id
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Id(usize);
impl Deref for Id {
    type Target = usize;
    fn deref(&self) -> &Self::Target { &self.0 }
}
impl From<usize> for Id {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

/// Track data
#[derive(Debug, Default)]
pub struct TrackData {
    pub title: Option<String>,
    pub album: Option<String>,
    pub artist: Option<String>,
    pub duration: Duration,
}
impl TrackData {
    /// Tries to read a audio file 
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Audio file not found -> [TrackDataError::NotFound]
    /// - Something wrong while opening an audio file -> [TrackDataError::Io]
    /// - Unable to read an audio data from the file -> [TrackDataError::Read]
    ///   (see [lofty::read_from])
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, TrackDataError> {
        let path = path.as_ref();
        let mut file = match fs::File::open(path) {
            Ok(file) => Ok(file),
            // File not found
            Err(e) if e.kind() == io::ErrorKind::NotFound => Err(TrackDataError::NotFound(path.into())),
            // Other io error
            Err(e) => Err(TrackDataError::Io(e))
        }?;
        let tagged = lofty::read_from(&mut file)
            .map_err(TrackDataError::Read)?;
        let duration = tagged.properties().duration();

        Ok(match tagged.tag(TagType::Id3v2) {
            Some(tags) => Self {
                title: tags.title().map(|t| t.to_string()),
                album: tags.album().map(|t| t.to_string()),
                artist: tags.artist().map(|t| t.to_string()),
                duration
            },
            None => Self {
                duration,
                ..Default::default()
            }
        })
    }
}

/// Track
#[derive(Debug)]
pub struct Track {
    pub id: Id,
    pub path: PathBuf,
    pub filename: Option<String>,
    pub data: Option<Rc<TrackData>>
}
impl Track {
    pub fn from_path<P: AsRef<Path>>(cache: &mut Cache, path: P) -> Result<Self, TrackDataError> {
        let path = PathBuf::from(path.as_ref());
        let data = cache.get_or_create(&path)?;

        let filename = {
            let filename = path
                .file_name()
                .and_then(|f| f.to_str());

            if let Some(filename) = filename {
                let parent = path.parent()
                    .and_then(|p| p.file_name().and_then(|f| f.to_str()));

                if let Some(parent) = parent {
                    Some(format!("{parent}/{filename}"))
                } else {
                    Some(filename.to_string())
                }
            } else {
                None
            }
        };

        Ok(Self {
            id: TRACK_ID.fetch_add(1, Ordering::Relaxed).into(),
            filename,
            path,
            data: Some(Rc::clone(data))
        })
    }

    pub fn try_title(&self) -> Option<&str> {
        self.data.as_ref().and_then(|d| d.title.as_deref())
    }
    pub fn try_album(&self) -> Option<&str> {
        self.data.as_ref().and_then(|d| d.album.as_deref())
    }
    pub fn try_artist(&self) -> Option<&str> {
        self.data.as_ref().and_then(|d| d.artist.as_deref())
    }
    pub fn try_duration(&self) -> Option<&Duration> {
        self.data.as_ref().map(|d| &d.duration)
    }

    /// Returns title if any
    /// If there is no title, returns file name 
    /// If somehow it was not possible to get the file name, returns `"<no title>"`
    pub fn title(&self) -> &str {
        self.try_title()
            .or(self.filename.as_deref())
            .unwrap_or("<no title>")
    }
    /// Returns duration if any, otherwise returns zero duration
    pub fn duration(&self) -> Duration {
        self.try_duration()
            .cloned()
            .unwrap_or_default()
    }
}
