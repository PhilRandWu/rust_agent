use crate::agent::flows::traditional::typegen::TypeInput;
use crate::llm::message::LlmMessage;

pub fn type_messages(input: &TypeInput) -> Vec<LlmMessage> {
    let components_json =
        serde_json::to_string(&input.components).unwrap_or_else(|_| "{}".to_string());

    let structure_json =
        serde_json::to_string(&input.structure).unwrap_or_else(|_| "{}".to_string());

    vec![
        LlmMessage::system(
            r#"You are a senior TypeScript domain model designer.

Your task is to generate TypeScript type files based on component contracts and planned project structure.

Return only valid JSON. Do not include markdown.

Output JSON shape:
{
  "files": [
    {
      "path": "/types/todo.ts",
      "code": "export interface Todo {\n  id: string;\n  title: string;\n  completed: boolean;\n}\n",
      "modelId": "Todo"
    }
  ]
}

Rules:
1. Generate one type file for each business entity found in component dataDependencies.
2. Use planned /types/*.ts paths from structure when available.
3. If no planned path exists, use /types/{lowercaseModel}.ts.
4. Export every interface or type.
5. Use TypeScript only.
6. Do not include React code.
7. Do not include mock data.
8. Do not import from runtime libraries.
9. Keep fields practical for the app request.
10. Always include id: string for entity interfaces."#,
        ),
        LlmMessage::user(format!(
            "Component JSON: {}\nStructure JSON: {}",
            components_json, structure_json
        )),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::flows::traditional::component::{
        ComponentEvent, ComponentEventParameter, ComponentOutput, ComponentProp, ComponentSpec,
    };
    use crate::agent::flows::traditional::structure::{
        FileKind, FileNode, GeneratedBy, StructureOutput,
    };
    use crate::llm::message::LlmRole;

    #[test]
    fn builds_type_messages_with_components_and_structure() {
        let messages = type_messages(&TypeInput::new(
            ComponentOutput {
                components: vec![ComponentSpec {
                    component_id: "TodoListPanel".to_string(),
                    original_id: "Todo list".to_string(),
                    component_type: "List".to_string(),
                    description: "Render todo items".to_string(),
                    props: vec![ComponentProp {
                        name: "todos".to_string(),
                        prop_type: "Todo[]".to_string(),
                        description: "Todo list data".to_string(),
                        required: true,
                    }],
                    events: vec![ComponentEvent {
                        name: "onToggleTodo".to_string(),
                        description: "Toggle todo completion".to_string(),
                        parameters: vec![ComponentEventParameter {
                            name: "id".to_string(),
                            parameter_type: "string".to_string(),
                        }],
                    }],
                    data_dependencies: vec!["Todo".to_string()],
                    shadcn_component: None,
                }],
            },
            StructureOutput {
                files: vec![FileNode {
                    path: "/types/todo.ts".to_string(),
                    kind: FileKind::New,
                    description: "Todo types".to_string(),
                    source_correlation: Some("Todo".to_string()),
                    generated_by: GeneratedBy::TypeDefinition,
                }],
            },
        ));

        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, LlmRole::System);
        assert_eq!(messages[1].role, LlmRole::User);
        assert!(messages[0].content.contains("dataDependencies"));
        assert!(
            messages[1]
                .content
                .contains(r#""dataDependencies":["Todo"]"#)
        );
        assert!(messages[1].content.contains("/types/todo.ts"));
    }
}
