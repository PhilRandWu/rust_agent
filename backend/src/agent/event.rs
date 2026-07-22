use crate::agent::flows::traditional::analysis::schema::AnalysisOutput;
use crate::agent::flows::traditional::app_gen::AppGenOutput;
use crate::agent::flows::traditional::assemble::AssembleOutput;
use crate::agent::flows::traditional::capability::CapabilityOutput;
use crate::agent::flows::traditional::component::ComponentOutput;
use crate::agent::flows::traditional::component_gen::{ComponentGenOutput, ComponentGenPartial};
use crate::agent::flows::traditional::dependency::DependencyOutput;
use crate::agent::flows::traditional::files::schema::FilesOutput;
use crate::agent::flows::traditional::hooks::HooksOutput;
use crate::agent::flows::traditional::intent::IntentOutput;
use crate::agent::flows::traditional::layout::LayoutOutput;
use crate::agent::flows::traditional::mock_data::MockDataOutput;
use crate::agent::flows::traditional::page_gen::PageGenOutput;
use crate::agent::flows::traditional::plan::schema::PlanOutput;
use crate::agent::flows::traditional::post_process::PostProcessOutput;
use crate::agent::flows::traditional::service::ServiceOutput;
use crate::agent::flows::traditional::structure::StructureOutput;
use crate::agent::flows::traditional::style_gen::StyleGenOutput;
use crate::agent::flows::traditional::typegen::TypeOutput;
use crate::agent::flows::traditional::ui::UiOutput;
use crate::agent::flows::traditional::utils::UtilsOutput;
use crate::session::VersionMeta;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentEvent {
    Analysis(AnalysisOutput),
    Intent(IntentOutput),
    Capability(CapabilityOutput),
    Ui(UiOutput),
    Component(ComponentOutput),
    Structure(StructureOutput),
    Dependency(DependencyOutput),
    Type(TypeOutput),
    Utils(UtilsOutput),
    MockData(MockDataOutput),
    Service(ServiceOutput),
    Hooks(HooksOutput),
    ComponentGen(ComponentGenOutput),
    PageGen(PageGenOutput),
    Layout(LayoutOutput),
    StyleGen(StyleGenOutput),
    AppGen(AppGenOutput),
    Assemble(AssembleOutput),
    PostProcess(PostProcessOutput),
    Plan(PlanOutput),
    Files(FilesOutput),
    ComponentGenPartial(ComponentGenPartial),
    SessionCommit(VersionMeta),
    Error(String),
    Done,
}
