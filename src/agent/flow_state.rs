use crate::agent::flows::traditional::analysis::schema::AnalysisOutput;
use crate::agent::flows::traditional::app_gen::AppGenOutput;
use crate::agent::flows::traditional::assemble::AssembleOutput;
use crate::agent::flows::traditional::capability::CapabilityOutput;
use crate::agent::flows::traditional::component::ComponentOutput;
use crate::agent::flows::traditional::component_gen::ComponentGenOutput;
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

#[derive(Debug, Clone, Default)]
pub struct TraditionalFlowState {
    pub analysis: Option<AnalysisOutput>,
    pub intent: Option<IntentOutput>,
    pub capability: Option<CapabilityOutput>,
    pub ui: Option<UiOutput>,
    pub component: Option<ComponentOutput>,
    pub structure: Option<StructureOutput>,
    pub dependency: Option<DependencyOutput>,
    pub types: Option<TypeOutput>,
    pub utils: Option<UtilsOutput>,
    pub mock_data: Option<MockDataOutput>,
    pub service: Option<ServiceOutput>,
    pub hooks: Option<HooksOutput>,
    pub component_gen: Option<ComponentGenOutput>,
    pub page_gen: Option<PageGenOutput>,
    pub layout: Option<LayoutOutput>,
    pub style_gen: Option<StyleGenOutput>,
    pub app_gen: Option<AppGenOutput>,
    pub assemble: Option<AssembleOutput>,
    pub post_process: Option<PostProcessOutput>,
    pub plan: Option<PlanOutput>,
    pub files: Option<FilesOutput>,
}

impl TraditionalFlowState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn final_files(&self) -> Option<FilesOutput> {
        if let Some(post_process) = &self.post_process {
            return Some(FilesOutput {
                files: post_process.files.clone(),
            });
        }

        if let Some(assemble) = &self.assemble {
            return Some(FilesOutput {
                files: assemble.files.clone(),
            });
        }
        self.files.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::flows::traditional::post_process::PostProcessOutput;
    use std::collections::BTreeMap;

    #[test]
    fn returns_post_processed_files_first() {
        let mut state = TraditionalFlowState::new();

        state.post_process = Some(PostProcessOutput {
            files: BTreeMap::from([(
                "/App.tsx".to_string(),
                "export default function App() {}".to_string(),
            )]),
            total_fixes: 1,
        });

        let files = state.final_files().expect("files should exist");

        assert_eq!(
            files.files.get("/App.tsx"),
            Some(&"export default function App() {}".to_string())
        );
    }
}
