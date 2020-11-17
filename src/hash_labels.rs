use std::collections::HashMap;
use std::path::Path;
use std::fs;

use parking_lot::RwLock;
use crate::{hash40, Hash40};

pub struct HashLabels {
    labels: HashMap<Hash40, String>,
}

impl HashLabels {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, std::io::Error> {
        fn inner(path: &Path) -> Result<HashLabels, std::io::Error> {
            Ok(HashLabels {
                labels: fs::read_to_string(path)?
                    .split('\n')
                    .map(|line| (hash40(&line), line.to_owned()))
                    .collect()
            })
        }

        inner(path.as_ref())
    }

    pub(crate) fn add_label<S: Into<String>>(&mut self, label: S) -> Hash40 {
        let label = label.into();
        let hash = hash40(&label);
        self.labels.insert(hash, label);

        hash
    }

    pub fn new() -> Self {
        Self { labels: Default::default() }
    }
}

impl Hash40 {
    pub fn label<'a>(&self, labels: &'a HashLabels) -> Option<&'a str> {
        labels.labels.get(self).map(|x| &**x)
    }
    
    pub fn global_label(&self) -> Option<String> {
        GLOBAL_LABELS.read().labels.get(self).map(Clone::clone)
    }
    
    //pub fn global_label<'a>(self) -> MappedRwLockReadGuard<'a, Option<&'a str>> {
    //    RwLockReadGuard::map(
    //        GLOBAL_LABELS.read(),
    //        |labels| &labels.labels.get(&self).map(|x| &**x)
    //    )
    //}
    
    pub fn set_global_labels_file<P: AsRef<Path>>(label_file: P) -> Result<(), std::io::Error> {
        Ok(Self::set_global_labels(HashLabels::from_file(label_file)?))
    }

    pub fn set_global_labels(labels: HashLabels) {
        *GLOBAL_LABELS.write() = labels;
    }
}

lazy_static::lazy_static! {
    pub static ref GLOBAL_LABELS: RwLock<HashLabels> = RwLock::new(HashLabels::new());
}

