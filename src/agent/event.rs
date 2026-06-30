use crate::agent::analysis::AnalysisOutput;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentEvent {
    Analysis(AnalysisOutput),
    Error(String),
    Done,
}
