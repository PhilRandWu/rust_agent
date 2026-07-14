use crate::agent::flows::traditional::style_gen::prompt::style_gen_messages;
use crate::agent::flows::traditional::style_gen::{StyleGenInput, StyleGenOutput};
use crate::llm::client::LlmClient;
use crate::llm::structured::structured_chat;
use std::sync::Arc;

pub struct StyleGenNode {
    client: Arc<dyn LlmClient>,
}

impl StyleGenNode {
    pub fn new(client: Arc<dyn LlmClient>) -> Self {
        Self { client }
    }

    #[tracing::instrument(skip_all, name = "node.style_gen")]
    pub async fn run(&self, input: StyleGenInput) -> anyhow::Result<StyleGenOutput> {
        let messages = style_gen_messages(&input);
        structured_chat(self.client.as_ref(), &messages).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::flows::traditional::component::ComponentOutput;
    use crate::agent::flows::traditional::dependency::{DependencyOutput, PackageJson};
    use crate::agent::flows::traditional::page_gen::PageGenOutput;
    use crate::agent::flows::traditional::ui::{LayoutPattern, UiOutput, VisualStyle};
    use crate::llm::mock::MockLlmClient;
    use std::collections::BTreeMap;

    #[tokio::test]
    async fn parses_style_gen_output_from_llm_json() {
        let client = Arc::new(MockLlmClient::new(
            r#"{
              "path": "styles.css",
              "content": ":root {\n  --color-primary: #2563eb;\n  --color-background: #f8fafc;\n}\n\nbody {\n  margin: 0;\n  background: var(--color-background);\n}\n\n.page-container {\n  max-width: 1120px;\n  margin: 0 auto;\n  padding: 2rem;\n}\n\n@keyframes fade-in {\n  from { opacity: 0; transform: translateY(8px); }\n  to { opacity: 1; transform: translateY(0); }\n}\n\n.animate-fade-in {\n  animation: fade-in 0.25s ease-out;\n}\n",
              "description": "Global styles with CSS variables and layout helpers"
            }"#,
        ));

        let node = StyleGenNode::new(client);

        let output = node
            .run(StyleGenInput::new(
                UiOutput {
                    layout_pattern: LayoutPattern::SinglePage,
                    visual_style: VisualStyle::Modern,
                    primary_sections: vec!["Hero".to_string()],
                    interactions: Vec::new(),
                    responsive_notes: Vec::new(),
                },
                ComponentOutput {
                    components: Vec::new(),
                },
                PageGenOutput { files: Vec::new() },
                DependencyOutput::new(
                    PackageJson {
                        scripts: BTreeMap::new(),
                        dependencies: BTreeMap::new(),
                        dev_dependencies: BTreeMap::new(),
                    },
                    BTreeMap::new(),
                    "Template dependencies loaded",
                ),
            ))
            .await
            .expect("style gen node should parse output");

        assert_eq!(output.path, "styles.css");
        assert!(output.content.contains(":root"));
        assert!(output.content.contains("@keyframes fade-in"));
        assert!(output.content.contains(".page-container"));
    }
}
