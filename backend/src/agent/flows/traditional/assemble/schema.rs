use crate::agent::flows::traditional::app_gen::AppGenOutput;
use crate::agent::flows::traditional::component_gen::ComponentGenOutput;
use crate::agent::flows::traditional::dependency::DependencyOutput;
use crate::agent::flows::traditional::hooks::HooksOutput;
use crate::agent::flows::traditional::layout::LayoutOutput;
use crate::agent::flows::traditional::mock_data::MockDataOutput;
use crate::agent::flows::traditional::page_gen::PageGenOutput;
use crate::agent::flows::traditional::service::ServiceOutput;
use crate::agent::flows::traditional::style_gen::StyleGenOutput;
use crate::agent::flows::traditional::typegen::TypeOutput;
use crate::agent::flows::traditional::utils::UtilsOutput;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssembleInput {
    pub dependency: DependencyOutput,
    pub types: TypeOutput,
    pub utils: UtilsOutput,
    pub mock_data: MockDataOutput,
    pub service: ServiceOutput,
    pub hooks: HooksOutput,
    pub component_code: ComponentGenOutput,
    pub pages: PageGenOutput,
    pub layouts: LayoutOutput,
    pub styles: StyleGenOutput,
    pub app: AppGenOutput,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AssembleOutput {
    pub files: BTreeMap<String, String>,
    pub stats: AssembleStats,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AssembleStats {
    pub total_files: usize,
    pub categories: BTreeMap<String, usize>,
}

impl AssembleInput {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        dependency: DependencyOutput,
        types: TypeOutput,
        utils: UtilsOutput,
        mock_data: MockDataOutput,
        service: ServiceOutput,
        hooks: HooksOutput,
        component_code: ComponentGenOutput,
        pages: PageGenOutput,
        layouts: LayoutOutput,
        styles: StyleGenOutput,
        app: AppGenOutput,
    ) -> Self {
        Self {
            dependency,
            types,
            utils,
            mock_data,
            service,
            hooks,
            component_code,
            pages,
            layouts,
            styles,
            app,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_assemble_output() {
        let mut files = BTreeMap::new();
        files.insert(
            "/App.tsx".to_string(),
            "export default function App() {}".to_string(),
        );

        let output = AssembleOutput {
            files,
            stats: AssembleStats {
                total_files: 1,
                categories: BTreeMap::from([("app".to_string(), 1)]),
            },
        };

        let json = serde_json::to_string(&output).unwrap();

        assert!(json.contains("/App.tsx"));
        assert!(json.contains("totalFiles"));
    }
}
