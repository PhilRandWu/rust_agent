use crate::agent::analysis::AnalysisOutput;
use crate::agent::plan::PlanOutput;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentEvent {
    Analysis(AnalysisOutput),
    Error(String),
    Plan(PlanOutput),
    Done,
}
