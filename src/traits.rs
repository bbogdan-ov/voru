use std::{io, path::{Path, PathBuf}, time::Duration};

use rand::Rng;

// Traits
pub trait Expand {
    /// Replaces a tilda (`~`) at the start of the path with `$HOME` var
    ///
    /// # Errors
    ///
    /// Will return an [std::env::VarError] if `$HOME` var was not found
    fn expand(&self) -> Result<PathBuf, std::env::VarError>;
    /// Tries to read the dir if there is an asterisk (*) on the end of the path
    ///
    /// # Errors
    ///
    /// See [std::fs::read_dir]
    fn expand_to_multiple(&self) -> io::Result<Vec<PathBuf>>;
}
pub trait ToReadable {
    /// Converts [Duration] to the readable form `M:SS` or `H:MM:SS`
    fn to_readable(&self) -> String;
}
pub trait MoveTo {
    /// Move an vector item to a different position
    ///
    /// # Panics
    ///
    /// Panics if one of `from_index` or `to_index` is out of bounds
    fn move_to(&mut self, from_index: usize, to_index: usize);
}
pub trait Shuffle {
    /// Randomize order of the elements in the vector
    fn shuffle(&mut self);
}

// Implement
impl<T: AsRef<Path>> Expand for T {
    fn expand(&self) -> Result<PathBuf, std::env::VarError> {
        let path = self.as_ref();
        let path_str = path.to_string_lossy();

        if path.starts_with("~") {
            let home = std::env::var("HOME")?;
            Ok(path_str.replacen('~', &home, 1).into())
        } else {
            Ok(path.into())
        }
    }
    fn expand_to_multiple(&self) -> io::Result<Vec<PathBuf>> {
        let path = self.as_ref();

        if !path.ends_with("*") {
            return Ok(vec![path.to_path_buf()]);
        }
        let dir_path_str = path.to_string_lossy();
        let dir_path_str = dir_path_str.trim_end_matches('*');

        let mut paths = vec![];
        for entry in std::fs::read_dir(dir_path_str)? {
            let entry = entry?;
            paths.push(entry.path());
        }

        Ok(paths)
    }
}
impl ToReadable for Duration {
    fn to_readable(&self) -> String {
        let secs = self.as_secs();
        let mins = secs / 60;
        let hours = mins / 60;

        let secs = secs % 60;
        let mins = mins % 60;

        let secs =
            if secs <= 9 { format!("0{secs}") }
            else { secs.to_string() };

        if hours == 0 {
            format!("{}:{}", mins, secs)
        } else {
            let mins =
                if mins <= 9 { format!("0{mins}") }
                else { mins.to_string() };

            format!("{}:{}:{}", hours, mins, secs)
        }
    }
}
impl<T> MoveTo for Vec<T> {
    fn move_to(&mut self, from_index: usize, to_index: usize) {
        let item = self.remove(from_index);
        self.insert(to_index, item);
    }
}
impl<T> Shuffle for Vec<T> {
    fn shuffle(&mut self) {
        if self.is_empty() { return }

        self.sort_by(|_, _| rand::thread_rng()
            .gen_range(-2..2)
            .partial_cmp(&0)
            .unwrap());
    }
}
