use crate::llm::json_repair::errors::{RepairError, RepairStage};
use crate::llm::json_repair::stages::{
    balance_brackets, extract_first_json_object, strip_bom, strip_code_fence, strip_trailing_commas,
};
use serde::de::DeserializeOwned;

mod errors;
mod stages;
#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct RepairOutcome<T> {
    pub value: T,
    pub stages_applied: Vec<RepairStage>,
}

pub fn parse_with_repair<T>(raw: &str) -> Result<RepairOutcome<T>, RepairError>
where
    T: DeserializeOwned,
{
    if let Ok(value) = serde_json::from_str::<T>(raw) {
        return Ok(RepairOutcome {
            value,
            stages_applied: vec![],
        });
    }

    let pipeline: [(RepairStage, fn(&str) -> String); 5] = [
        (RepairStage::BomStripped, strip_bom),
        (RepairStage::FenceStripped, strip_code_fence),
        (RepairStage::FirstObjectExtracted, extract_first_json_object),
        (RepairStage::TrailingCommaRemoved, strip_trailing_commas),
        (RepairStage::BracketBalanced, balance_brackets),
    ];

    let mut current = raw.to_string();
    let mut applied = Vec::new();
    let mut last_error: Option<serde_json::Error> = None;

    for (stage, apply) in pipeline {
        let next = apply(&current);
        if next == current {
            continue;
        }
        current = next;
        applied.push(stage);

        match serde_json::from_str::<T>(&current) {
            Ok(value) => {
                tracing::debug!(
                    target: "llm::json_repair",
                    stages = ?applied,
                    "json repaired successfully"
                );
                return Ok(RepairOutcome {
                    value,
                    stages_applied: applied,
                });
            }
            Err(err) => last_error = Some(err),
        }
    }

    Err(RepairError {
        stages_applied: applied,
        source_snippet: snippet(raw),
        last_attempt_snippet: snippet(&current),
        parse_error: last_error
            .map(|e| e.to_string())
            .unwrap_or_else(|| "no repair stage changed input".to_string()),
    })
}

fn snippet(input: &str) -> String {
    const MAX: usize = 400;
    if input.len() < MAX {
        input.to_string()
    } else {
        format!(
            "{}...<truncated {} bytes>",
            &input[..MAX],
            input.len() - MAX
        )
    }
}
