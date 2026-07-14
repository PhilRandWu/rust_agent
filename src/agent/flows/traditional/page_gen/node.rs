use crate::agent::flows::traditional::page_gen::prompt::page_gen_messages;
use crate::agent::flows::traditional::page_gen::{PageGenInput, PageGenOutput};
use crate::agent::flows::traditional::structure::GeneratedBy;
use crate::llm::client::LlmClient;
use crate::llm::structured::structured_chat;
use std::sync::Arc;

pub struct PageGenNode {
    client: Arc<dyn LlmClient>,
}

impl PageGenNode {
    pub fn new(client: Arc<dyn LlmClient>) -> Self {
        Self { client }
    }

    #[tracing::instrument(skip_all, name = "node.page_gen")]
    pub async fn run(&self, input: PageGenInput) -> anyhow::Result<PageGenOutput> {
        let has_page_files = input.structure.files.iter().any(|file| {
            file.generated_by == GeneratedBy::Page
                && file.path.starts_with("/pages/")
                && file.path.ends_with(".tsx")
        });

        if !has_page_files {
            return Ok(PageGenOutput { files: Vec::new() });
        }

        let messages = page_gen_messages(&input);
        structured_chat(self.client.as_ref(), &messages).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::flows::traditional::component::ComponentOutput;
    use crate::agent::flows::traditional::component_gen::{ComponentCodeFile, ComponentGenOutput};
    use crate::agent::flows::traditional::hooks::{HooksFile, HooksOutput};
    use crate::agent::flows::traditional::structure::{
        FileKind, FileNode, GeneratedBy, StructureOutput,
    };
    use crate::agent::flows::traditional::typegen::{TypeFile, TypeOutput};
    use crate::agent::flows::traditional::ui::{LayoutPattern, UiOutput, VisualStyle};
    use crate::llm::mock::MockLlmClient;

    #[tokio::test]
    async fn parses_page_gen_output_from_llm_json() {
        let client = Arc::new(MockLlmClient::new(
            r#"{
              "files": [
                {
                  "path": "/pages/Home.tsx",
                  "content": "import React from 'react';\nimport TodoListPanel from '../components/TodoListPanel';\nimport { useTodos } from '../hooks/useTodos';\n\nexport default function Home() {\n  const { todos } = useTodos();\n  return <main className=\"min-h-screen bg-slate-50 p-8\"><TodoListPanel todos={todos} /></main>;\n}\n",
                  "description": "Home page composed from todo components"
                }
              ]
            }"#,
        ));

        let node = PageGenNode::new(client);

        let output = node
            .run(PageGenInput::new(
                StructureOutput {
                    files: vec![FileNode {
                        path: "/pages/Home.tsx".to_string(),
                        kind: FileKind::New,
                        description: "Home page".to_string(),
                        source_correlation: Some("Home".to_string()),
                        generated_by: GeneratedBy::Page,
                    }],
                },
                UiOutput {
                    layout_pattern: LayoutPattern::SinglePage,
                    visual_style: VisualStyle::Modern,
                    primary_sections: vec!["Todo input".to_string(), "Todo list".to_string()],
                    interactions: vec!["Create todo".to_string()],
                    responsive_notes: vec!["Stack on mobile".to_string()],
                },
                ComponentOutput {
                    components: Vec::new(),
                },
                ComponentGenOutput {
                    files: vec![ComponentCodeFile {
                        path: "/components/TodoListPanel.tsx".to_string(),
                        content: "export default function TodoListPanel() { return <div />; }"
                            .to_string(),
                        description: Some("Todo list component".to_string()),
                    }],
                },
                HooksOutput {
                    files: vec![HooksFile {
                        path: "/hooks/useTodos.ts".to_string(),
                        content: "export function useTodos() { return { todos: [] }; }".to_string(),
                        description: "Todo hook".to_string(),
                    }],
                },
                TypeOutput {
                    files: vec![TypeFile {
                        path: "/types/todo.ts".to_string(),
                        code: "export interface Todo { id: string; title: string; }".to_string(),
                        model_id: "Todo".to_string(),
                    }],
                },
            ))
            .await
            .expect("page gen node should parse output");

        assert_eq!(output.files.len(), 1);
        assert_eq!(output.files[0].path, "/pages/Home.tsx");
        assert!(output.files[0].content.contains("useTodos"));
        assert!(output.files[0].content.contains("TodoListPanel"));
    }

    #[tokio::test]
    async fn skips_when_no_page_files_exist() {
        let client = Arc::new(MockLlmClient::new("{}"));
        let node = PageGenNode::new(client);

        let output = node
            .run(PageGenInput::new(
                StructureOutput { files: Vec::new() },
                UiOutput {
                    layout_pattern: LayoutPattern::SinglePage,
                    visual_style: VisualStyle::Modern,
                    primary_sections: Vec::new(),
                    interactions: Vec::new(),
                    responsive_notes: Vec::new(),
                },
                ComponentOutput {
                    components: Vec::new(),
                },
                ComponentGenOutput { files: Vec::new() },
                HooksOutput { files: Vec::new() },
                TypeOutput { files: Vec::new() },
            ))
            .await
            .expect("page gen node should skip");

        assert!(output.files.is_empty());
    }
}
