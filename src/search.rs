use crate::{ArcFile, HashLabels, Hash40};

use rayon::prelude::*;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;

use std::collections::HashMap;

impl HashLabels {
    pub(crate) fn get_ordered_matches(&self, search: &str) -> Vec<Hash40> {
        let matcher = SkimMatcherV2::default();

        let mut labels: Vec<(i64, Hash40)> = self.labels
            .par_iter()
            .filter_map(|(hash, label)| {
                matcher.fuzzy_match(&label, search).map(|score| (score, *hash))
            })
            .collect();

        labels.par_sort_unstable_by_key(|x| -x.0);

        labels.into_iter().map(|(_, hash)| hash).collect()
    }
}

impl ArcFile {
    pub fn generate_search_cache(&self) -> SearchCache {
        let mut cache = HashMap::<Hash40, Vec<Hash40>>::new();
        for file_path in &self.file_system.file_paths {
            cache.entry(file_path.file_name.hash40())
                .or_default()
                .push(file_path.path.hash40());

            cache.entry(file_path.parent.hash40())
                .or_default()
                .push(file_path.path.hash40());
        }
            
        SearchCache(cache)
    }
}

#[repr(C)]
pub struct SearchCache(HashMap<Hash40, Vec<Hash40>>);

impl SearchCache {
    pub fn search(&self, term: &str, labels: &HashLabels, max: usize) -> Vec<Hash40> {
        let matches = labels.get_ordered_matches(term);

        matches.into_iter()
            .map(|search_match| {
                self.0
                    .get(&search_match)
                    .map(|x| &x[..])
                    .unwrap_or_else(|| &[][..])
                    .iter()
            })
            .flatten()
            .take(max)
            .copied()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search() {
        let labels = HashLabels::from_file("hash_labels.txt").unwrap();
        let arc = ArcFile::open("data.arc").unwrap();


        let gen_cache = std::time::Instant::now();
        let search_cache = arc.generate_search_cache();
        dbg!(gen_cache.elapsed());

        let search = std::time::Instant::now();
        let found = search_cache.search("mari", &labels, 20);
        dbg!(search.elapsed());

        dbg!(found);
    }
}
