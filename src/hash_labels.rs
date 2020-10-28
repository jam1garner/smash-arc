use std::collections::HashMap;
use std::path::Path;
use std::fs;

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
}

impl Hash40 {
    pub fn label<'a>(&self, labels: &'a HashLabels) -> Option<&'a str> {
        labels.labels.get(self).map(|x| &**x)
    }
}
