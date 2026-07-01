use crate::agent::flows::traditional::analysis::schema::AnalysisOutput;
use crate::agent::flows::traditional::files::schema::FilesOutput;
use crate::agent::flows::traditional::plan::schema::PlanOutput;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentEvent {
    Analysis(AnalysisOutput),
    Plan(PlanOutput),
    Files(FilesOutput),
    Error(String),
    Done,
}
