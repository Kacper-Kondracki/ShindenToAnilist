use std::{
    cmp::Reverse,
    collections::BinaryHeap,
    iter,
};

use ahash::{
    AHashMap,
    AHashSet,
};
use indexmap::IndexMap;
use ordered_float::OrderedFloat;
use roaring::RoaringBitmap;

#[inline(always)]
#[allow(unused)]
pub(crate) fn ngrams<const N: usize>(s: &[u8]) -> impl Iterator<Item = u32> {
    s.windows(N)
        .map(|w| w.iter().fold(0u32, |acc, &x| (acc << 8) | x as u32))
}
#[inline(always)]
const fn ngram_mask<const N: usize>() -> u32 { if N == 4 { u32::MAX } else { (1u32 << (8 * N)) - 1 } }
#[inline(always)]
pub(crate) fn ngrams_padded<const N: usize>(s: &str) -> impl Iterator<Item = u32> + '_ {
    let mask = ngram_mask::<N>();

    let mut state = 0u32;
    for _ in 0..N - 1 {
        state = ((state << 8) | b'^' as u32) & mask;
    }

    s.bytes()
        .chain(iter::repeat_n(b'$', N - 1))
        .scan(state, move |state, byte| {
            *state = ((*state << 8) | byte as u32) & mask;
            Some(*state)
        })
}
#[inline(always)]
pub(crate) fn dedup_ngram(ngram: impl Iterator<Item = u32>) -> impl Iterator<Item = u32> {
    let mut seen = AHashSet::with_capacity(30);
    ngram.into_iter().filter(move |x| seen.insert(*x))
}

#[inline(always)]
pub(crate) fn ngram_padded_dedup<const N: usize>(s: &str) -> impl Iterator<Item = u32> {
    dedup_ngram(ngrams_padded::<N>(s))
}

#[derive(Debug, Default, Copy, Clone)]
struct DocData {
    len: u32,
    canonical: u32,
}
#[derive(Debug, Default, Clone)]
struct Posting {
    item: RoaringBitmap,
    df: u32,
}

pub(crate) trait Scorer {
    fn score(matched: u32, query_len: u32, doc_len: u32, idf_sum: f32) -> f32;
}

#[derive(Debug, Default)]
pub(crate) struct RecallJaccard;
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

#[derive(Debug, Default, Clone)]
pub(crate) struct NGramIndexBuilder<const N: usize> {
    postings: AHashMap<u32, Posting>,
    docs: Vec<DocData>,
}
impl<const N: usize> NGramIndexBuilder<N> {
    pub(crate) fn add_ngram(&mut self, text: &str) -> u32 {
        let id = self.docs.len() as u32;

        let len = ngram_padded_dedup::<N>(text).fold(0u32, |acc, ngram| {
            self.postings.entry(ngram).or_default().item.insert(id);
            acc + 1
        });

        self.docs.push(DocData { len, canonical: id });

        id
    }

    pub(crate) fn add_alias(&mut self, text: &str, id: u32) -> u32 {
        let alias_id = self.add_ngram(text);

        self.docs[alias_id as usize].canonical = id;

        alias_id
    }

    fn precalculate_dfs(&mut self) {
        for (_, posting) in self.postings.iter_mut() {
            posting.df = posting.item.len() as u32;
        }
    }

    pub(crate) fn build(mut self) -> NGramIndex<N> {
        self.precalculate_dfs();

        NGramIndex {
            postings: self.postings,
            docs: self.docs,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub(crate) struct NGramIndex<const N: usize> {
    postings: AHashMap<u32, Posting>,
    docs: Vec<DocData>,
}
impl<const N: usize> NGramIndex<N> {
    #[inline(always)]
    fn select_candidates(terms: &[&Posting], is_and: bool) -> Option<RoaringBitmap> {
        if is_and {
            let mut candidates = terms[0].item.clone();

            for posting in &terms[1..] {
                candidates &= &posting.item;
                if candidates.is_empty() {
                    break;
                }
            }
            if !candidates.is_empty() {
                return Some(candidates);
            }
        }

        let mut candidates = terms[0].item.clone();
        let seed_terms = (terms.len() / 3).clamp(1, 4);

        for posting in &terms[1..=seed_terms] {
            candidates |= &posting.item;
        }

        (!candidates.is_empty()).then_some(candidates)
    }

    pub(crate) fn search<S: Scorer>(
        &self,
        query: &str,
        limit: usize,
        threshold: f32,
        mode: SearchMode,
    ) -> Vec<(u32, f32)> {
        const DF_CUTOFF_RATIO: f32 = 0.2;

        if self.docs.is_empty() || limit == 0 {
            return Vec::new();
        }

        let mut terms: Vec<&Posting> = Vec::new();

        let q_len = ngram_padded_dedup::<N>(query).fold(0u32, |acc, ngram| {
            if let Some(posting) = self.postings.get(&ngram)
                && posting.df < (DF_CUTOFF_RATIO * self.docs.len() as f32) as u32
            {
                terms.push(posting);
            }
            acc + 1
        });

        if terms.is_empty() || q_len == 0 {
            return Vec::new();
        }

        terms.sort_by_key(|posting| posting.df);
        let Some(candidates) = Self::select_candidates(&terms, mode == SearchMode::And) else {
            return Vec::new();
        };

        let mut matches: IndexMap<u32, u32> = IndexMap::new();

        for posting in terms {
            let intersect = &candidates & &posting.item;
            for doc in intersect.iter() {
                *matches.entry(doc).or_default() += 1;
            }
        }

        let mut score_map: IndexMap<u32, f32> = IndexMap::with_capacity(matches.len());

        for (doc, m) in matches {
            let doc_data = self.docs[doc as usize];
            let d_len = doc_data.len;

            if d_len == 0 || m == 0 {
                continue;
            }

            let max_recall = m as f32 / q_len as f32;
            if max_recall < threshold {
                continue;
            }

            let score = S::score(m, q_len, d_len, 0.0).clamp(0.0, 1.0);
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

        let mut heap: BinaryHeap<Reverse<(OrderedFloat<f32>, u32)>> = BinaryHeap::with_capacity(limit);

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

        results.sort_by_key(|&(k, s)| (Reverse(OrderedFloat(s)), k));

        results
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub(crate) enum SearchMode {
    #[default]
    And,
    Or,
}
