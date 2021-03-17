use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::{hash40, Hash40};
use parking_lot::RwLock;

pub struct HashLabels {
    labels: HashMap<Hash40, String>,
}

impl HashLabels {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, std::io::Error> {
        fn inner(path: &Path) -> Result<HashLabels, std::io::Error> {
            Ok(HashLabels::from_string(&fs::read_to_string(path)?))
        }

        inner(path.as_ref())
    }

    pub fn from_string(text: &str) -> Self {
        HashLabels {
            labels: text
                .lines()
                .map(|line| (hash40(&line), line.to_owned()))
                .collect(),
        }
    }

    pub(crate) fn add_label<S: Into<String>>(&mut self, label: S) -> Hash40 {
        let label = label.into();
        let hash = hash40(&label);
        self.labels.insert(hash, label);

        hash
    }

    pub fn new() -> Self {
        Self {
            labels: Default::default(),
        }
    }
}

impl Hash40 {
    pub fn label<'a>(self, labels: &'a HashLabels) -> Option<&'a str> {
        labels.labels.get(&self).map(|x| &**x)
    }

    pub fn global_label(self) -> Option<String> {
        GLOBAL_LABELS.read().labels.get(&self).map(Clone::clone)
    }

    //pub fn global_label<'a>(self) -> MappedRwLockReadGuard<'a, Option<&'a str>> {
    //    RwLockReadGuard::map(
    //        GLOBAL_LABELS.read(),
    //        |labels| &labels.labels.get(&self).map(|x| &**x)
    //    )
    //}

    pub fn set_global_labels_file<P: AsRef<Path>>(label_file: P) -> Result<(), std::io::Error> {
        Self::set_global_labels(HashLabels::from_file(label_file)?);
        Ok(())
    }

    pub fn set_global_labels(labels: HashLabels) {
        *GLOBAL_LABELS.write() = labels;
    }
}

lazy_static::lazy_static! {
    pub static ref GLOBAL_LABELS: RwLock<HashLabels> = RwLock::new(HashLabels::new());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_string_line_feed() {
        let text = "a\nbc\ndef\n";
        let labels = HashLabels::from_string(&text);

        let hash_a = hash40("a");
        let hash_bc = hash40("bc");
        let hash_def = hash40("def");

        assert_eq!("a", hash_a.label(&labels).unwrap());
        assert_eq!("bc", hash_bc.label(&labels).unwrap());
        assert_eq!("def", hash_def.label(&labels).unwrap());
    }

    #[test]
    fn from_string_carriage_return_line_feed() {
        // Ensure the hash label file still works when edited on Windows.
        let text = "a\r\nbc\r\ndef\r\n";
        let labels = HashLabels::from_string(&text);

        let hash_a = hash40("a");
        let hash_bc = hash40("bc");
        let hash_def = hash40("def");

        assert_eq!("a", hash_a.label(&labels).unwrap());
        assert_eq!("bc", hash_bc.label(&labels).unwrap());
        assert_eq!("def", hash_def.label(&labels).unwrap());
    }
}