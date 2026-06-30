use crate::agent::analysis::AnalysisOutput;
use crate::agent::files::FilesOutput;
use crate::agent::plan::PlanOutput;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentEvent {
    Analysis(AnalysisOutput),
    Plan(PlanOutput),
    Files(FilesOutput),
    Error(String),
    Done,
}
