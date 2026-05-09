use std::cmp::Ordering;
use std::collections::HashMap;

use crate::candidates::{scan_candidates, Candidate};
use crate::config::Config;
use crate::files::discover_files;
use crate::report::{DuplicateCandidate, Location, Report};
use crate::Result;

pub fn find_duplicates(config: &Config) -> Result<Report> {
    let files = discover_files(config)?;
    let candidates = scan_candidates(config, &files)?;
    let mut duplicates = compare_candidates(config, &candidates);

    duplicates.sort_by(compare_duplicate_order);
    if let Some(max) = config.max_candidates {
        duplicates.truncate(max);
    }

    Ok(Report {
        candidates: duplicates,
    })
}

fn compare_candidates(config: &Config, candidates: &[Candidate]) -> Vec<DuplicateCandidate> {
    if config.threshold <= 0.0 {
        return compare_all_pairs(config, candidates);
    }

    let shared_counts = shared_fingerprint_counts(candidates);
    let mut duplicates = Vec::new();

    for ((left_index, right_index), shared) in shared_counts {
        let left = &candidates[left_index];
        let right = &candidates[right_index];
        if !compatible(config, left, right) {
            continue;
        }
        if max_possible_score(left, right) < config.threshold {
            continue;
        }

        let total = left.fingerprints.len() + right.fingerprints.len() - shared;
        let score = shared as f64 / total as f64;
        if score >= config.threshold {
            duplicates.push(duplicate(left, right, score, shared, total));
        }
    }

    duplicates
}

fn compare_all_pairs(config: &Config, candidates: &[Candidate]) -> Vec<DuplicateCandidate> {
    let mut duplicates = Vec::new();

    for left_index in 0..candidates.len() {
        for right_index in (left_index + 1)..candidates.len() {
            let left = &candidates[left_index];
            let right = &candidates[right_index];
            if !compatible(config, left, right) {
                continue;
            }
            if max_possible_score(left, right) < config.threshold {
                continue;
            }

            let shared = left.fingerprints.intersection_len(&right.fingerprints);
            let total = left.fingerprints.union_len(&right.fingerprints);
            let score = if total == 0 {
                0.0
            } else {
                shared as f64 / total as f64
            };

            if score >= config.threshold {
                duplicates.push(duplicate(left, right, score, shared, total));
            }
        }
    }

    duplicates
}

fn shared_fingerprint_counts(candidates: &[Candidate]) -> HashMap<(usize, usize), usize> {
    let mut postings: HashMap<&str, Vec<usize>> = HashMap::new();
    for (index, candidate) in candidates.iter().enumerate() {
        for fingerprint in candidate.fingerprints.iter() {
            postings
                .entry(fingerprint.as_str())
                .or_default()
                .push(index);
        }
    }

    let mut shared_counts = HashMap::new();
    for indexes in postings.values() {
        for left_offset in 0..indexes.len() {
            for right_offset in (left_offset + 1)..indexes.len() {
                let left = indexes[left_offset];
                let right = indexes[right_offset];
                *shared_counts.entry((left, right)).or_insert(0) += 1;
            }
        }
    }

    shared_counts
}

fn duplicate(
    left: &Candidate,
    right: &Candidate,
    score: f64,
    shared: usize,
    total: usize,
) -> DuplicateCandidate {
    DuplicateCandidate {
        score,
        language: duplicate_language(left, right),
        left: location(left),
        right: location(right),
        left_nodes: left.nodes,
        right_nodes: right.nodes,
        shared_fingerprints: shared,
        total_fingerprints: total,
    }
}

fn compatible(config: &Config, left: &Candidate, right: &Candidate) -> bool {
    config.cross_language || left.language.family() == right.language.family()
}

fn max_possible_score(left: &Candidate, right: &Candidate) -> f64 {
    let left_len = left.fingerprints.len();
    let right_len = right.fingerprints.len();
    let larger = left_len.max(right_len);
    if larger == 0 {
        0.0
    } else {
        left_len.min(right_len) as f64 / larger as f64
    }
}

fn duplicate_language(left: &Candidate, right: &Candidate) -> String {
    if left.language.family() != right.language.family() {
        return "cross-language".to_string();
    }

    match (left.language, right.language) {
        (crate::language::Language::Daml, crate::language::Language::Daml) => "daml".to_string(),
        _ => left.language.family().to_string(),
    }
}

fn location(candidate: &Candidate) -> Location {
    Location {
        file: candidate.file.display().to_string(),
        start_line: candidate.start_line,
        end_line: candidate.end_line,
        kind: candidate.kind.clone(),
        name: candidate.name.clone(),
    }
}

fn compare_duplicate_order(left: &DuplicateCandidate, right: &DuplicateCandidate) -> Ordering {
    right
        .score
        .partial_cmp(&left.score)
        .unwrap_or(Ordering::Equal)
        .then_with(|| left.left.file.cmp(&right.left.file))
        .then_with(|| left.left.start_line.cmp(&right.left.start_line))
        .then_with(|| left.right.file.cmp(&right.right.file))
        .then_with(|| left.right.start_line.cmp(&right.right.start_line))
}
