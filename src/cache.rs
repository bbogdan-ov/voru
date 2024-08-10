use std::{collections::HashMap, path::{Path, PathBuf}, rc::Rc};

use crate::track::{TrackData, TrackDataError};

#[derive(Debug)]
pub struct Cache {
    pub tracks_data: HashMap<PathBuf, Rc<TrackData>>
}
impl Cache {
    pub fn new() -> Self {
        Self { tracks_data: HashMap::new() }
    }

    pub fn get_or_create<P: AsRef<Path>>(&mut self, path: P) -> Result<&Rc<TrackData>, TrackDataError> {
        let path = PathBuf::from(path.as_ref());

        if !self.tracks_data.contains_key(&path) {
            let data = Rc::new(TrackData::from_path(&path)?);
            self.add(path.clone(), data);
        }

        Ok(self.get(path).unwrap())
    }
    pub fn add<P: AsRef<Path>, T: Into<Rc<TrackData>>>(&mut self, path: P, track_data: T) {
        let path = path.as_ref();
        if !self.has(&path) {
            self.tracks_data.insert(path.into(), track_data.into());
        }
    }
    pub fn get<P: AsRef<Path>>(&self, path: P) -> Option<&Rc<TrackData>> {
        self.tracks_data.get(path.as_ref())
    }
    pub fn has<P: AsRef<Path>>(&self, path: P) -> bool {
        self.tracks_data.contains_key(path.as_ref())
    }
}
