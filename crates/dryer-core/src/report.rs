use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Report {
    pub candidates: Vec<DuplicateCandidate>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DuplicateCandidate {
    pub score: f64,
    pub language: String,
    pub left: Location,
    pub right: Location,
    pub left_nodes: usize,
    pub right_nodes: usize,
    pub shared_fingerprints: usize,
    pub total_fingerprints: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Location {
    pub file: String,
    pub start_line: usize,
    pub end_line: usize,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}
