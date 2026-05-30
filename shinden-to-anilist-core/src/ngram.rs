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
use ordered_float::{
    OrderedFloat,
    Pow,
};
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
    norm: f32,
    canonical: u32,
}
#[derive(Debug, Default, Clone)]
struct Posting {
    item: RoaringBitmap,
    df: u32,
}

pub(crate) trait Scorer {
    #[inline(always)]
    fn finalize(x: f32) -> f32 { x }
    fn update_match(current: f32, idf: f32) -> f32;
    fn score(matched: f32, query_val: f32, doc_len: u32, doc_norm: f32) -> f32;
}

#[derive(Debug, Default)]
#[allow(unused)]
pub(crate) struct RecallJaccard;
impl Scorer for RecallJaccard {
    #[inline(always)]
    fn update_match(current: f32, _idf: f32) -> f32 { current + 1.0 }
    #[inline(always)]
    fn score(m: f32, q_len: f32, d_len: u32, _d_norm: f32) -> f32 {
        let alpha = 0.8;
        let beta = 1.0 - alpha;
        let recall = m / q_len;
        let union = q_len + (d_len as f32) - m;
        let jaccard = m / union;
        alpha * recall + beta * jaccard
    }
}

#[derive(Debug, Default)]
#[allow(unused)]
pub(crate) struct TfIdfCosine;
impl Scorer for TfIdfCosine {
    #[inline(always)]
    fn finalize(x: f32) -> f32 { x.sqrt() }
    #[inline(always)]
    fn update_match(current: f32, idf: f32) -> f32 { current + idf.pow(2) }
    #[inline(always)]
    fn score(dot_product: f32, q_norm: f32, _d_len: u32, d_norm: f32) -> f32 {
        if q_norm == 0.0 || d_norm == 0.0 {
            return 0.0;
        }
        dot_product / (q_norm * d_norm)
    }
}

#[derive(Debug, Default, Clone)]
pub(crate) struct NGramIndexBuilder<const N: usize> {
    postings: AHashMap<u32, Posting>,
    docs: Vec<DocData>,
}
impl<const N: usize> NGramIndexBuilder<N> {
    #[inline(always)]
    pub(crate) fn add_ngram(&mut self, text: &str) -> u32 {
        let id = self.docs.len() as u32;

        let mut len = 0;
        for ngram in ngram_padded_dedup::<N>(text) {
            self.postings.entry(ngram).or_default().item.insert(id);
            len += 1;
        }

        self.docs.push(DocData {
            len,
            canonical: id,
            norm: 0.0,
        });

        id
    }

    #[inline(always)]
    pub(crate) fn add_alias(&mut self, text: &str, id: u32) -> u32 {
        let alias_id = self.add_ngram(text);

        self.docs[alias_id as usize].canonical = id;

        alias_id
    }

    pub(crate) fn build(mut self) -> NGramIndex<N> {
        for posting in self.postings.values_mut() {
            posting.item.optimize();
        }

        let num_docs = self.docs.len() as f32;
        let mut doc_norms_sq = vec![0.0f32; self.docs.len()];

        for posting in self.postings.values_mut() {
            let df = posting.item.len() as u32;
            posting.df = df;

            if df == 0 {
                continue;
            }

            let idf = (1.0 + num_docs / (df as f32)).ln();
            let weight_sq = idf * idf;

            for doc_id in posting.item.iter() {
                doc_norms_sq[doc_id as usize] += weight_sq;
            }
        }

        for (doc, norm_sq) in self.docs.iter_mut().zip(doc_norms_sq) {
            doc.norm = norm_sq.sqrt();
        }

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

            for posting in terms.iter().skip(1) {
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

        for posting in terms.iter().skip(1).take(seed_terms) {
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

        let mut terms: Vec<(&Posting, f32)> = Vec::with_capacity(30);
        let df_cutoff = (DF_CUTOFF_RATIO * self.docs.len() as f32) as u32;
        let num_docs = self.docs.len() as f32;

        let mut query_val = 0.0f32;

        for ngram in ngram_padded_dedup::<N>(query) {
            if let Some(posting) = self.postings.get(&ngram) {
                let idf = (1.0 + num_docs / (posting.df as f32)).ln();
                query_val = S::update_match(query_val, idf);

                if posting.df < df_cutoff {
                    terms.push((posting, idf));
                }
            }
        }

        let query_val = S::finalize(query_val);

        if terms.is_empty() || query_val == 0.0 {
            return Vec::new();
        }

        terms.sort_by_key(|&(p, _)| p.df);
        let raw_postings: Vec<&Posting> = terms.iter().map(|&(p, _)| p).collect();
        let Some(candidates) = Self::select_candidates(&raw_postings, mode == SearchMode::And) else {
            return Vec::new();
        };

        let mut matches: IndexMap<u32, f32> = IndexMap::new();

        for (posting, idf) in terms {
            let intersect = &candidates & &posting.item;
            for doc in intersect.iter() {
                matches
                    .entry(doc)
                    .and_modify(|val| *val = S::update_match(*val, idf))
                    .or_insert_with(|| S::update_match(0.0, idf));
            }
        }

        let mut score_map: IndexMap<u32, f32> = IndexMap::with_capacity(matches.len());

        for (doc, m) in matches {
            let doc_data = self.docs[doc as usize];

            let score = S::score(m, query_val, doc_data.len, doc_data.norm);
            let score = score.clamp(0.0, 1.0);

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
