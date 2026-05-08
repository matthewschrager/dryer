use std::collections::BTreeSet;

use crate::normalize::Normalized;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Fingerprints {
    values: BTreeSet<String>,
}

impl Fingerprints {
    pub fn from_normalized(normalized: &Normalized) -> Self {
        let mut reprs = Vec::new();
        normalized.child_reprs(&mut reprs);
        Self {
            values: reprs.into_iter().collect(),
        }
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &String> {
        self.values.iter()
    }

    pub fn intersection_len(&self, other: &Self) -> usize {
        self.values.intersection(&other.values).count()
    }

    pub fn union_len(&self, other: &Self) -> usize {
        self.values.union(&other.values).count()
    }
}
