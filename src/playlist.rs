use std::{cell::RefCell, fs, io, path::{Path, PathBuf}, rc::Rc, sync::atomic::{AtomicUsize, Ordering}, time::Duration};

use thiserror::Error;

use crate::{cache::Cache, config::Config, track::{Id, Track, TrackDataError}, traits::Expand};

// Static
static PLAYLIST_ID: AtomicUsize = AtomicUsize::new(0);

// Errors
#[derive(Debug, Error)]
pub enum PlaylistError {
    #[error("I/O error: {0}")]
    Io(io::Error),
    #[error("Unable to load a track: {0}")]
    Track(TrackDataError)
}
#[derive(Debug, Error)]
pub enum LoadPlaylistsError {
    #[error("I/O error: {0}")]
    Io(io::Error),
    #[error("\"{0}\" no such file or directory :(((")]
    NotFound(PathBuf),
    #[error("Wrong file type at \"{0}\"")]
    WrongFileType(PathBuf),
    #[error("Playlist error: {0}")]
    Playlist(PlaylistError)
}

/// Load playlists from a directory
/// Returns playlists and track paths that failed to load
pub fn playlists_form_config(
    cache: &mut Cache,
    config: &Config,
) -> Result<Vec<Rc<RefCell<Playlist>>>, LoadPlaylistsError> {
    let mut playlists = vec![];

    for path in &config.playlists {
        let path = path.expand()
            .unwrap_or(path.clone());

        if !path.exists() {
            return Err(LoadPlaylistsError::NotFound(path));
        }

        if path.is_dir() {
            // Read dir of playlists
            let dir = fs::read_dir(path)
                .map_err(LoadPlaylistsError::Io)?;

            for entry in dir {
                let entry = entry
                    .map_err(LoadPlaylistsError::Io)?;
                let path = entry.path();
                if path.is_dir() { continue; }

                let playlist = Playlist::from_path(cache, path)
                    .map_err(LoadPlaylistsError::Playlist)?;
                playlists.push(Rc::new(RefCell::new(playlist)));
            }
        } else if path.is_file() {
            // Try load from path
            let playlist = Playlist::from_path(cache, path)
                .map_err(LoadPlaylistsError::Playlist)?;
            playlists.push(Rc::new(RefCell::new(playlist)));
        } else {
            // The file is something else
            return Err(LoadPlaylistsError::WrongFileType(path))
        }
    }

    Ok(playlists)
}

/// Playlist
#[derive(Debug)]
pub struct Playlist {
    #[allow(unused)]
    pub id: Id,
    pub name: String,
    pub tracks: Vec<Rc<Track>>,
    pub duration: Duration
}
impl Playlist {
    pub fn new<S: ToString>(name: S, tracks: Vec<Rc<Track>>) -> Self {
        let duration = tracks
            .iter()
            .fold(Duration::default(), |acc, t| acc + t.duration());

        Self {
            id: PLAYLIST_ID.fetch_add(1, Ordering::Relaxed).into(),
            name: name.to_string(),
            tracks,
            duration
        }
    }
    /// Create a playlist from a file containing track paths
    /// Every line in the file is a absolute or relative
    /// (relative to the parent directory of the current playlist file) path to a track
    ///
    /// # Errors
    ///
    /// Retuns an error if:
    /// - Some tracks were not found -> [PlaylistError::NotFound]
    /// - The playlist file was not found, or couldn't be read -> [PlaylistError::Io]
    /// - Unable to load a track -> [PlaylistError::Track]
    ///   (see: [Track::from_path], [TrackData::from_path])
    pub fn from_path<P: AsRef<Path>>(cache: &mut Cache, path: P) -> Result<Self, PlaylistError> {
        let path = path.as_ref();
        let content = fs::read_to_string(path)
            .map_err(PlaylistError::Io)?;
        let mut duration = Duration::default();

        let mut tracks: Vec<Rc<Track>> = vec![];
        for track_path in content.split('\n') {
            let track_path = track_path.trim();
            if track_path.is_empty() { continue; }

            // Try to expand and canonicalize the path
            let track_path = track_path
                .expand()
                .unwrap_or(track_path.into());

            // Trying to load a track from the path
            let track = Track::from_path(cache, track_path)
                .map_err(PlaylistError::Track)?;

            duration += track.duration();
            tracks.push(track.into());
        }

        Ok(Self {
            id: PLAYLIST_ID.fetch_add(1, Ordering::Relaxed).into(),
            name: path
                .file_name().map(|s| s.to_string_lossy().to_string())
                .unwrap_or("<no name>".into()),
            tracks,
            duration
        })
    }
}
