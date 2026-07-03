use crate::agent::flows::traditional::app_gen::prompt::app_gen_messages;
use crate::agent::flows::traditional::app_gen::{AppGenInput, AppGenOutput};
use crate::llm::client::LlmClient;
use crate::llm::structured::structured_chat;
use std::sync::Arc;

pub struct AppGenNode {
    client: Arc<dyn LlmClient>,
}

impl AppGenNode {
    pub fn new(client: Arc<dyn LlmClient>) -> Self {
        Self { client }
    }

    pub async fn run(&self, input: AppGenInput) -> anyhow::Result<AppGenOutput> {
        let messages = app_gen_messages(&input);
        structured_chat(self.client.as_ref(), &messages).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::flows::traditional::layout::LayoutOutput;
    use crate::agent::flows::traditional::page_gen::{PageCodeFile, PageGenOutput};
    use crate::agent::flows::traditional::structure::StructureOutput;
    use crate::agent::flows::traditional::ui::{LayoutPattern, UiOutput, VisualStyle};
    use crate::llm::mock::MockLlmClient;
    use std::collections::BTreeMap;

    #[tokio::test]
    async fn parses_app_gen_output_from_llm_json() {
        let client = Arc::new(MockLlmClient::new(
            r#"{
              "path": "/App.tsx",
              "content": "import React from 'react';\nimport Home from './pages/Home';\n\nexport default function App() {\n  return <Home />;\n}\n",
              "description": "Application root rendering Home page"
            }"#,
        ));

        let node = AppGenNode::new(client);

        let output = node
            .run(AppGenInput::new(
                UiOutput {
                    layout_pattern: LayoutPattern::SinglePage,
                    visual_style: VisualStyle::Modern,
                    primary_sections: vec!["Todo app".to_string()],
                    interactions: Vec::new(),
                    responsive_notes: Vec::new(),
                },
                StructureOutput { files: Vec::new() },
                PageGenOutput {
                    files: vec![PageCodeFile {
                        path: "/pages/Home.tsx".to_string(),
                        content: "export default function Home() { return <main />; }".to_string(),
                        description: "Home page".to_string(),
                    }],
                },
                LayoutOutput {
                    layouts_code: Vec::new(),
                    route_structure: BTreeMap::new(),
                },
            ))
            .await
            .expect("app gen node should parse output");

        assert_eq!(output.path, "/App.tsx");
        assert!(output.content.contains("import Home from './pages/Home'"));
        assert!(output.content.contains("export default function App"));
    }
}
