use ahash::{AHashMap, AHashSet};
use ordered_float::OrderedFloat;
use roaring::RoaringBitmap;
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::iter;

pub fn ngrams<const N: usize>(s: &[u8]) -> impl Iterator<Item = u32> + '_ {
    s.windows(N).map(|w| {
        let mut x = 0u32;
        w.iter().for_each(|&b| {
            x = (x << 8) | b as u32;
        });
        x
    })
}

pub fn dedup_ngrams(ngrams: impl Iterator<Item = u32>) -> impl Iterator<Item = u32> {
    let mut seen = AHashSet::new();
    ngrams.into_iter().filter(move |x| seen.insert(*x))
}

pub fn pad(s: &str, pad_len: usize) -> Vec<u8> {
    let mut padded = Vec::with_capacity(s.len() + 2 * pad_len);
    padded.extend(iter::repeat_n(b'^', pad_len));
    padded.extend_from_slice(s.as_bytes());
    padded.extend(iter::repeat_n(b'$', pad_len));
    padded
}

pub fn ngrams_dedup<const N: usize>(s: &[u8]) -> impl Iterator<Item = u32> {
    dedup_ngrams(ngrams::<N>(s))
}

#[derive(Debug, Default)]
pub struct NGramIndex<const N: usize> {
    postings: AHashMap<u32, RoaringBitmap>,
    aliases: AHashMap<u32, u32>,
    count: u32,
    ngram_count: Vec<usize>,
}

impl<const N: usize> NGramIndex<N> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_ngram(&mut self, text: &str) -> u32 {
        let id = self.count;
        self.count += 1;
        let mut ngram_count = 0;
        for ngram in ngrams_dedup::<N>(&pad(text, N - 1)) {
            ngram_count += 1;
            self.postings.entry(ngram).or_default().insert(id);
        }
        self.ngram_count.push(ngram_count);
        id
    }

    pub fn add_alias(&mut self, text: &str, id: u32) -> u32 {
        let alias_id = self.add_ngram(text);
        self.aliases.insert(alias_id, id);
        alias_id
    }

    pub fn search(&self, query: &str, limit: usize, threshold: f32) -> Vec<(u32, f32)> {
        if self.count == 0 || limit == 0 {
            return Vec::new();
        }

        let mut terms: Vec<(&RoaringBitmap, usize)> = Vec::new();
        let mut query_ngram_count = 0;
        for ngram in ngrams_dedup::<N>(&pad(query, N - 1)) {
            query_ngram_count += 1;
            if let Some(bitmap) = self.postings.get(&ngram) {
                terms.push((bitmap, bitmap.len() as usize));
            }
        }

        if terms.is_empty() {
            return Vec::new();
        }

        terms.sort_unstable_by_key(|(_, df)| *df);
        let seed_terms = match terms.len() {
            0..=2 => 1,
            3..=6 => 2,
            7..=10 => 3,
            _ => 4,
        };
        let mut candidates: Option<RoaringBitmap> = None;

        for (bitmap, _) in terms.iter().copied().take(seed_terms) {
            candidates = Some(match candidates {
                None => bitmap.clone(),
                Some(acc) => &acc | bitmap,
            });
        }

        let Some(candidates) = candidates else {
            return Vec::new();
        };

        if candidates.is_empty() {
            return Vec::new();
        }

        let mut matched: AHashMap<u32, usize> = AHashMap::new();

        for (bitmap, _) in terms.iter().copied() {
            let intersect = &candidates & bitmap;
            for doc in intersect {
                *matched.entry(doc).or_insert(0) += 1;
            }
        }

        let mut score_map: AHashMap<u32, f32> = AHashMap::new();

        for (doc, matched_count) in matched {
            let ngram_count = self.ngram_count[doc as usize];
            if ngram_count == 0 {
                continue;
            }

            let recall = matched_count as f32 / query_ngram_count as f32;
            let union_count = query_ngram_count + ngram_count - matched_count;
            let jaccard = matched_count as f32 / union_count as f32;

            let alpha = 0.8;
            let score = alpha * recall + (1.0 - alpha) * jaccard;

            if score < threshold {
                continue;
            }
            let canonical_id = self.aliases.get(&doc).copied().unwrap_or(doc);

            score_map
                .entry(canonical_id)
                .and_modify(|current_score| {
                    *current_score = current_score.max(score);
                })
                .or_insert(score);
        }

        let mut heap: BinaryHeap<Reverse<(OrderedFloat<f32>, u32)>> =
            BinaryHeap::with_capacity(limit);

        for (doc, score) in score_map {
            let score = OrderedFloat(score);
            if heap.len() < limit {
                heap.push(Reverse((score, doc)));
            } else if let Some(&Reverse((min_score, _))) = heap.peek()
                && score > min_score
            {
                heap.pop();
                heap.push(Reverse((score, doc)));
            }
        }

        let mut results: Vec<(u32, f32)> = heap
            .into_iter()
            .map(|Reverse((score, doc))| (doc, *score))
            .collect();

        results.sort_unstable_by(|a, b| b.1.total_cmp(&a.1));

        results
    }
}
