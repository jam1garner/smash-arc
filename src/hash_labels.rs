use std::collections::HashMap;
use std::path::Path;
use std::fs;

use parking_lot::RwLock;
use crate::{hash40, Hash40};

pub struct HashLabels {
    labels: HashMap<Hash40, String>,
}

impl HashLabels {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Self {
        fn inner(path: &Path) -> HashLabels {
            HashLabels {
                labels: fs::read_to_string(path).unwrap()
                .split('\n')
                .map(|line| (hash40(&line), line.to_owned()))
                .collect()
            }
        }

        inner(path.as_ref())
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
    
    pub fn set_global_labels_file<P: AsRef<Path>>(label_file: P) {
        Self::set_global_labels(HashLabels::from_file(label_file))
    }

    pub fn set_global_labels(labels: HashLabels) {
        *GLOBAL_LABELS.write() = labels;
    }
}

lazy_static::lazy_static! {
    pub static ref GLOBAL_LABELS: RwLock<HashLabels> = RwLock::new(HashLabels::new());
}

