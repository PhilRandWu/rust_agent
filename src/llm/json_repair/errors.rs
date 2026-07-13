use std::fmt;
use std::fmt::Formatter;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepairStage {
    BomStripped,
    FenceStripped,
    FirstObjectExtracted,
    TrailingCommaRemoved,
    BracketBalanced,
}

impl fmt::Display for RepairStage {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let name = match self {
            RepairStage::BomStripped => "bom_stripped",
            RepairStage::FenceStripped => "fence_stripped",
            RepairStage::FirstObjectExtracted => "first_object_extracted",
            RepairStage::TrailingCommaRemoved => "trailing_comma_removed",
            RepairStage::BracketBalanced => "bracket_balanced",
        };
        f.write_str(name)
    }
}

#[derive(Debug)]
pub struct RepairError {
    pub stages_applied: Vec<RepairStage>,
    pub source_snippet: String,
    pub last_attempt_snippet: String,
    pub parse_error: String,
}

impl fmt::Display for RepairError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "failed to parse LLM JSON after {} repair stage(s) [{}]: {}",
            self.stages_applied.len(),
            self.stages_applied
                .iter()
                .map(RepairStage::to_string)
                .collect::<Vec<_>>()
                .join(","),
            self.parse_error
        )
    }
}

impl std::error::Error for RepairError {}
