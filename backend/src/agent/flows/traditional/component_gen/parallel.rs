use crate::agent::flows::traditional::component::ComponentSpec;
use crate::agent::flows::traditional::component_gen::prompt::single_component_messages;
use crate::agent::flows::traditional::component_gen::{
    ComponentCodeFile, ComponentGenInput, ComponentGenPartial, PartialProgress,
};
use crate::llm::client::LlmClient;
use crate::llm::semaphore::LlmGate;
use crate::llm::structured::parse_structured_content;
use futures::future::BoxFuture;
use futures::stream::{FuturesUnordered, StreamExt};
use std::sync::Arc;

type WorkerResult = (usize, ComponentSpec, anyhow::Result<ComponentCodeFile>);

pub struct ComponentGenStream {
    inner: FuturesUnordered<BoxFuture<'static, WorkerResult>>,
    total: usize,
    completed: usize,
    ordered: Vec<Option<ComponentCodeFile>>,
    errors: Vec<(String, anyhow::Error)>,
}

impl ComponentGenStream {
    pub fn spawn(client: Arc<dyn LlmClient>, gate: LlmGate, input: ComponentGenInput) -> Self {
        let components = input.components.components.clone();
        let total = components.len();
        let inner = FuturesUnordered::new();

        for (idx, spec) in components.into_iter().enumerate() {
            let client = Arc::clone(&client);
            let gate = gate.clone();
            let input = input.clone();
            let fut: BoxFuture<'static, WorkerResult> = Box::pin(async move {
                let permit = gate.acquire().await;
                match permit {
                    Ok(_p) => {
                        let r = generate_one(&client, &input, &spec).await;
                        (idx, spec, r)
                    }
                    Err(e) => (idx, spec, Err(e)),
                }
            });
            inner.push(fut);
        }

        Self {
            inner,
            total,
            completed: 0,
            ordered: (0..total).map(|_| None).collect(),
            errors: Vec::new(),
        }
    }

    pub async fn next_partial(&mut self) -> Option<ComponentGenPartial> {
        loop {
            let (idx, spec, result) = self.inner.next().await?;
            match result {
                Ok(file) => {
                    self.completed += 1;
                    tracing::debug!(
                        target: "component_gen.stream",
                        completed = self.completed,
                        total = self.total,
                        path = %file.path,
                        "component ready"
                    );
                    let event = ComponentGenPartial {
                        file: file.clone(),
                        progress: PartialProgress {
                            completed: self.completed,
                            total: self.total,
                        },
                    };
                    self.ordered[idx] = Some(file);
                    return Some(event);
                }
                Err(err) => {
                    tracing::warn!(
                        target: "component_gen::stream",
                        component_id = %spec.component_id,
                        error = %err,
                        "component generation failed: skipping"
                    );
                    self.errors.push((spec.component_id, err));
                }
            }
        }
    }

    pub fn finish(self) -> anyhow::Result<Vec<ComponentCodeFile>> {
        let files: Vec<ComponentCodeFile> = self.ordered.into_iter().flatten().collect();
        if files.is_empty() && !self.errors.is_empty() {
            anyhow::bail!(
                "all {} component generations failed; first error: {}",
                self.errors.len(),
                self.errors[0].1
            );
        }
        Ok(files)
    }
}

pub async fn generate_components(
    client: Arc<dyn LlmClient>,
    gate: LlmGate,
    input: ComponentGenInput,
) -> anyhow::Result<Vec<ComponentCodeFile>> {
    let mut stream = ComponentGenStream::spawn(client, gate, input);
    while (stream.next_partial().await).is_some() {}
    stream.finish()
}

async fn generate_one(
    client: &Arc<dyn LlmClient>,
    input: &ComponentGenInput,
    spec: &ComponentSpec,
) -> anyhow::Result<ComponentCodeFile> {
    let message = single_component_messages(input, spec);
    let response = client.chat(&message).await?;
    parse_structured_content::<ComponentCodeFile>(&response.content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::flows::traditional::component::{ComponentOutput, ComponentSpec};
    use crate::agent::flows::traditional::hooks::HooksOutput;
    use crate::agent::flows::traditional::service::ServiceOutput;
    use crate::agent::flows::traditional::structure::StructureOutput;
    use crate::agent::flows::traditional::typegen::TypeOutput;
    use crate::llm::mock::MockLlmClient;

    fn spec(id: &str) -> ComponentSpec {
        ComponentSpec {
            component_id: id.to_string(),
            original_id: id.to_string(),
            component_type: "Basic".into(),
            description: "test".into(),
            props: vec![],
            events: vec![],
            data_dependencies: vec![],
            shadcn_component: None,
        }
    }

    fn input_with(specs: Vec<ComponentSpec>) -> ComponentGenInput {
        ComponentGenInput {
            structure: StructureOutput { files: vec![] },
            components: ComponentOutput { components: specs },
            types: TypeOutput { files: vec![] },
            service: ServiceOutput { files: vec![] },
            hooks: HooksOutput { files: vec![] },
        }
    }

    #[tokio::test]
    async fn empty_components_returns_empty() {
        let client: Arc<dyn LlmClient> = Arc::new(MockLlmClient::new("{}"));
        let gate = LlmGate::new(2);
        let out = generate_components(client, gate, input_with(vec![]))
            .await
            .unwrap();
        assert!(out.is_empty());
    }

    #[tokio::test]
    async fn generates_all_components_in_input_order() {
        let client: Arc<dyn LlmClient> = Arc::new(MockLlmClient::new(
            r#"{"path":"/components/X.tsx","content":"export default () => <div/>;"}"#,
        ));
        let gate = LlmGate::new(4);
        let out = generate_components(
            client,
            gate,
            input_with(vec![spec("A"), spec("B"), spec("C")]),
        )
        .await
        .unwrap();
        assert_eq!(out.len(), 3);
    }

    #[tokio::test]
    async fn fails_only_when_all_components_fail() {
        let client: Arc<dyn LlmClient> = Arc::new(MockLlmClient::new("not json"));
        let gate = LlmGate::new(2);
        let err = generate_components(client, gate, input_with(vec![spec("A"), spec("B")]))
            .await
            .expect_err("all failed → should bail");
        assert!(err.to_string().contains("all"));
    }

    #[tokio::test]
    async fn stream_yields_progress_for_each_completion() {
        let client: Arc<dyn LlmClient> = Arc::new(MockLlmClient::new(
            r#"{"path":"/components/X.tsx","content":"export default () => <div/>;"}"#,
        ));
        let gate = LlmGate::new(4);
        let mut stream = ComponentGenStream::spawn(
            client,
            gate,
            input_with(vec![spec("A"), spec("B"), spec("C")]),
        );

        let mut events = Vec::new();
        while let Some(evt) = stream.next_partial().await {
            events.push(evt);
        }
        assert_eq!(events.len(), 3);

        let completed: Vec<usize> = events.iter().map(|e| e.progress.completed).collect();
        assert_eq!(completed, vec![1, 2, 3]);
        assert!(events.iter().all(|e| e.progress.total == 3));

        let files = stream.finish().expect("finish should succeed");
        assert_eq!(files.len(), 3);
    }

    #[tokio::test]
    async fn stream_skips_failed_components_but_finish_ok_if_any_succeed() {
        let client: Arc<dyn LlmClient> = Arc::new(MockLlmClient::sequence(vec![
            r#"{"path":"/components/A.tsx","content":"..."}"#,
            "not json",
            r#"{"path":"/components/C.tsx","content":"..."}"#,
        ]));
        let gate = LlmGate::new(1);
        let mut stream = ComponentGenStream::spawn(
            client,
            gate,
            input_with(vec![spec("A"), spec("B"), spec("C")]),
        );
        let mut evts = Vec::new();
        while let Some(e) = stream.next_partial().await {
            evts.push(e);
        }
        assert_eq!(evts.len(), 2, "1 failed silently, 2 succeed");
        let files = stream.finish().expect("2 succeed → finish OK");
        assert_eq!(files.len(), 2);
    }
}
