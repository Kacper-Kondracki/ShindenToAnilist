use ahash::{AHashMap, AHashSet};
use ordered_float::OrderedFloat;
use roaring::RoaringBitmap;
use smallvec::SmallVec;
use std::collections::BinaryHeap;
use std::iter;

#[inline(always)]
pub fn ngrams<const N: usize>(s: &[u8]) -> impl Iterator<Item = u32> {
    s.windows(N)
        .map(|w| w.iter().fold(0u32, |acc, &x| (acc << 8) | x as u32))
}

pub trait DedupNgram {
    fn dedup_ngram(self) -> impl Iterator<Item = u32>;
}

impl<T: Iterator<Item = u32>> DedupNgram for T {
    #[inline(always)]
    fn dedup_ngram(self) -> impl Iterator<Item = u32> {
        let mut seen = AHashSet::new();
        self.into_iter().filter(move |x| seen.insert(*x))
    }
}

pub trait PadNgram {
    fn pad_ngram(&self, ngram_size: usize) -> SmallVec<[u8; 32]>;
}

impl PadNgram for str {
    #[inline(always)]
    fn pad_ngram(&self, ngram_size: usize) -> SmallVec<[u8; 32]> {
        let pad_len = ngram_size - 1;
        let mut padded = SmallVec::with_capacity(self.len() + 2 * pad_len);
        padded.extend(iter::repeat_n(b'^', pad_len));
        padded.extend_from_slice(self.as_bytes());
        padded.extend(iter::repeat_n(b'$', pad_len));
        padded
    }
}

#[derive(Debug, Default, Copy, Clone)]
struct DocData {
    pub len: u32,
    pub canonical: u32,
}

#[derive(Debug, Default)]
struct Posting {
    item: RoaringBitmap,
    df: u32,
}

pub trait Scorer {
    fn score(matched: u32, query_len: u32, doc_len: u32, idf_sum: f32) -> f32;
}

pub struct RecallJaccard;

impl Scorer for RecallJaccard {
    #[inline(always)]
    fn score(m: u32, q: u32, d: u32, _: f32) -> f32 {
        let alpha = 0.8;
        let beta = 1.0 - alpha;
        let recall = m as f32 / q as f32;
        let union = q + d - m;
        let jaccard = m as f32 / union as f32;
        alpha * recall + beta * jaccard
    }
}

#[derive(Debug, Default)]
pub struct NGramIndexBuilder<const N: usize> {
    postings: AHashMap<u32, Posting>,
    docs: Vec<DocData>,
}

impl<const N: usize> NGramIndexBuilder<N> {
    pub fn add_ngram(&mut self, text: &str) -> u32 {
        let id = self.docs.len() as u32;
        let len = ngrams::<N>(&text.pad_ngram(N))
            .dedup_ngram()
            .fold(0u32, |acc, ngram| {
                self.postings.entry(ngram).or_default().item.insert(id);
                acc + 1
            });

        self.docs.push(DocData { len, canonical: id });
        id
    }

    pub fn add_alias(&mut self, text: &str, id: u32) -> u32 {
        let alias_id = self.add_ngram(text);
        self.docs[alias_id as usize].canonical = id;
        alias_id
    }

    pub fn precalculate_dfs(&mut self) {
        for (_, posting) in self.postings.iter_mut() {
            posting.df = posting.item.len() as u32;
        }
    }

    pub fn build(mut self) -> NGramIndex<N> {
        self.precalculate_dfs();

        let NGramIndexBuilder { postings, docs } = self;
        NGramIndex { postings, docs }
    }
}

#[derive(Debug, Default)]
pub struct NGramIndex<const N: usize> {
    postings: AHashMap<u32, Posting>,
    docs: Vec<DocData>,
}

impl<const N: usize> NGramIndex<N> {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline(always)]
    fn select_candidates(terms: &[&Posting]) -> Option<RoaringBitmap> {
        const MAX_CANDIDATES: usize = 5_000;

        let mut candidates = terms[0].item.clone();

        for posting in terms.iter().copied().skip(1) {
            candidates &= &posting.item;

            if candidates.is_empty() {
                break;
            }
        }

        if !candidates.is_empty() {
            return Some(candidates);
        }

        let mut fallback = terms[0].item.clone();
        let seed_terms = (terms.len() / 3).clamp(1, 4);
        for posting in terms.iter().skip(1).copied().take(seed_terms) {
            fallback |= &posting.item;
            if fallback.len() as usize > MAX_CANDIDATES {
                break;
            }
        }

        (!fallback.is_empty()).then_some(fallback)
    }

    pub fn search<S: Scorer>(&self, query: &str, limit: usize, threshold: f32) -> Vec<(u32, f32)> {
        const DF_CUTOFF_RATIO: f32 = 1.0;

        if self.docs.is_empty() || limit == 0 {
            return Vec::new();
        }
        let mut terms: Vec<&Posting> = Vec::new();
        let q_len = ngrams::<N>(&query.pad_ngram(N))
            .dedup_ngram()
            .fold(0u32, |acc, ngram| {
                if let Some(posting) = self.postings.get(&ngram)
                    && posting.df as f32 <= DF_CUTOFF_RATIO * self.docs.len() as f32
                {
                    terms.push(posting);
                }
                acc + 1
            });

        if terms.is_empty() || q_len == 0 {
            return Vec::new();
        }

        terms.sort_unstable_by_key(|posting| posting.df);

        let Some(candidates) = Self::select_candidates(&terms) else {
            return Vec::new();
        };

        let mut matches = vec![0u32; self.docs.len()];

        for posting in terms.iter().copied() {
            let intersect = &candidates & &posting.item;
            for doc in intersect.iter() {
                matches[doc as usize] += 1;
            }
        }

        let mut score_map: AHashMap<u32, f32> = AHashMap::new();

        for doc in candidates.iter() {
            let doc_data = self.docs[doc as usize];
            let m = matches[doc as usize];
            let d_len = doc_data.len;
            if d_len == 0 || m == 0 {
                continue;
            }

            let max_recall = m as f32 / q_len as f32;
            if max_recall < threshold {
                continue;
            }

            let score = S::score(m, q_len, d_len, 0.0);

            if score < threshold {
                continue;
            }
            let canonical_id = doc_data.canonical;

            score_map
                .entry(canonical_id)
                .and_modify(|current_score| {
                    *current_score = current_score.max(score);
                })
                .or_insert(score);
        }

        let mut heap = BinaryHeap::new();
        for (doc, score) in score_map {
            let score = OrderedFloat(score);
            heap.push((score, doc));
            if heap.len() > limit {
                heap.pop();
            }
        }

        let mut results: Vec<(u32, f32)> =
            heap.into_iter().map(|(score, doc)| (doc, *score)).collect();

        results.sort_unstable_by(|a, b| b.1.total_cmp(&a.1));

        results
    }
}
