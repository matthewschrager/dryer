use crate::report::{DuplicateCandidate, Location, Report};

pub fn format_text(report: &Report) -> String {
    if report.candidates.is_empty() {
        return "No duplicate candidates found.\n".to_string();
    }

    let mut out = String::new();
    for (index, candidate) in report.candidates.iter().enumerate() {
        if index > 0 {
            out.push('\n');
        }
        out.push_str(&format_candidate(candidate));
        out.push('\n');
    }
    out
}

pub fn format_json(report: &Report) -> serde_json::Result<String> {
    serde_json::to_string_pretty(report)
}

pub fn format_sarif(report: &Report) -> serde_json::Result<String> {
    let results = report
        .candidates
        .iter()
        .map(sarif_result)
        .collect::<Vec<_>>();

    serde_json::to_string_pretty(&serde_json::json!({
        "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "dryer",
                    "informationUri": "https://github.com/unclebob/dry4clj",
                    "rules": [{
                        "id": "structural-duplicate",
                        "name": "Structural duplicate candidate",
                        "shortDescription": {
                            "text": "Structurally similar code was found"
                        }
                    }]
                }
            },
            "results": results
        }]
    }))
}

fn format_candidate(candidate: &DuplicateCandidate) -> String {
    format!(
        "DUPLICATE {} score={:.2}\n  {}\n  {}",
        candidate.language,
        candidate.score,
        format_location(&candidate.left),
        format_location(&candidate.right)
    )
}

fn format_location(location: &Location) -> String {
    match &location.name {
        Some(name) => format!(
            "{}:{}-{} {} {}",
            location.file, location.start_line, location.end_line, location.kind, name
        ),
        None => format!(
            "{}:{}-{} {}",
            location.file, location.start_line, location.end_line, location.kind
        ),
    }
}

fn sarif_result(candidate: &DuplicateCandidate) -> serde_json::Value {
    serde_json::json!({
        "ruleId": "structural-duplicate",
        "level": "warning",
        "message": {
            "text": format!(
                "Structural duplicate candidate with score {:.2}: {} and {}",
                candidate.score,
                format_location(&candidate.left),
                format_location(&candidate.right)
            )
        },
        "locations": [
            sarif_location(&candidate.left),
            sarif_location(&candidate.right)
        ]
    })
}

fn sarif_location(location: &Location) -> serde_json::Value {
    serde_json::json!({
        "physicalLocation": {
            "artifactLocation": {
                "uri": location.file
            },
            "region": {
                "startLine": location.start_line,
                "endLine": location.end_line
            }
        }
    })
}
