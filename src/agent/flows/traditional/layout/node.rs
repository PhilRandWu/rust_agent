use crate::agent::flows::traditional::layout::prompt::layout_messages;
use crate::agent::flows::traditional::layout::{LayoutInput, LayoutOutput};
use crate::agent::flows::traditional::ui::LayoutPattern;
use crate::llm::client::LlmClient;
use crate::llm::structured::structured_chat;
use std::collections::BTreeMap;
use std::sync::Arc;

pub struct LayoutNode {
    client: Arc<dyn LlmClient>,
}

impl LayoutNode {
    pub fn new(client: Arc<dyn LlmClient>) -> Self {
        Self { client }
    }

    pub async fn run(&self, input: LayoutInput) -> anyhow::Result<LayoutOutput> {
        if input.ui.layout_pattern == LayoutPattern::SinglePage || input.pages.files.len() <= 1 {
            return Ok(LayoutOutput {
                layouts_code: Vec::new(),
                route_structure: BTreeMap::new(),
            });
        }

        let messages = layout_messages(&input);
        structured_chat(self.client.as_ref(), &messages).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::flows::traditional::layout::LayoutInput;
    use crate::agent::flows::traditional::page_gen::{PageCodeFile, PageGenOutput};
    use crate::agent::flows::traditional::structure::StructureOutput;
    use crate::agent::flows::traditional::ui::{LayoutPattern, UiOutput, VisualStyle};
    use crate::llm::mock::MockLlmClient;

    #[tokio::test]
    async fn returns_empty_layouts_for_single_page_app() {
        let client = Arc::new(MockLlmClient::new("{}"));
        let node = LayoutNode::new(client);

        let output = node
            .run(LayoutInput::new(
                UiOutput {
                    layout_pattern: LayoutPattern::SinglePage,
                    visual_style: VisualStyle::Modern,
                    primary_sections: Vec::new(),
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
            ))
            .await
            .expect("layout node should skip single page app");

        assert!(output.layouts_code.is_empty());
        assert!(output.route_structure.is_empty());
    }

    #[tokio::test]
    async fn parses_layout_output_from_llm_json() {
        let client = Arc::new(MockLlmClient::new(
            r#"{
              "layoutsCode": [
                {
                  "path": "/layouts/MainLayout.tsx",
                  "content": "import React from 'react';\n\ninterface MainLayoutProps {\n  children: React.ReactNode;\n}\n\nexport default function MainLayout({ children }: MainLayoutProps) {\n  return <main className=\"min-h-screen bg-slate-50\">{children}</main>;\n}\n",
                  "description": "Main shared layout",
                  "pages": ["Home", "Settings"]
                }
              ],
              "routeStructure": {
                "MainLayout": ["Home", "Settings"]
              }
            }"#,
        ));

        let node = LayoutNode::new(client);

        let output = node
            .run(LayoutInput::new(
                UiOutput {
                    layout_pattern: LayoutPattern::MultiPage,
                    visual_style: VisualStyle::Modern,
                    primary_sections: vec!["Navigation".to_string(), "Content".to_string()],
                    interactions: Vec::new(),
                    responsive_notes: Vec::new(),
                },
                StructureOutput { files: Vec::new() },
                PageGenOutput {
                    files: vec![
                        PageCodeFile {
                            path: "/pages/Home.tsx".to_string(),
                            content: "export default function Home() { return <main />; }"
                                .to_string(),
                            description: "Home page".to_string(),
                        },
                        PageCodeFile {
                            path: "/pages/Settings.tsx".to_string(),
                            content: "export default function Settings() { return <main />; }"
                                .to_string(),
                            description: "Settings page".to_string(),
                        },
                    ],
                },
            ))
            .await
            .expect("layout node should parse output");

        assert_eq!(output.layouts_code.len(), 1);
        assert_eq!(output.layouts_code[0].path, "/layouts/MainLayout.tsx");
        assert_eq!(
            output.route_structure.get("MainLayout"),
            Some(&vec!["Home".to_string(), "Settings".to_string()])
        );
    }
}
