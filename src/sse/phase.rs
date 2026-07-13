use crate::sse::event::FrontendEventEnum;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq)]
pub enum TraditionalPhase {
    #[serde(rename = "planning")]
    Planning,

    #[serde(rename = "foundation")]
    Foundation,

    #[serde(rename = "logic")]
    Logic,

    #[serde(rename = "view")]
    View,

    #[serde(rename = "assembly")]
    Assembly,
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub enum Phase {
    Traditional(TraditionalPhase),
}

pub fn phase_for(event_type: FrontendEventEnum) -> Option<Phase> {
    use FrontendEventEnum::*;
    use TraditionalPhase::*;

    let phase = match event_type {
        Analysis | Intent | Capability | Ui | Component | Structure | Dependency => Planning,
        Type | Utils | MockData => Foundation,
        Service | Hooks => Logic,
        ComponentGen | PageGen | Layout | StyleGen => View,
        AppGen | Files => Assembly,
        Plan | Done | Error => return None,
    };
    Some(Phase::Traditional(phase))
}

#[cfg(test)]
mod tests {
    use super::*;
    use FrontendEventEnum::*;
    use TraditionalPhase::*;

    #[test]
    fn phase_mapping_matches_frontend_flow() {
        let cases: &[(FrontendEventEnum, Option<TraditionalPhase>)] = &[
            (Analysis, Some(Planning)),
            (Intent, Some(Planning)),
            (Capability, Some(Planning)),
            (Ui, Some(Planning)),
            (Component, Some(Planning)),
            (Structure, Some(Planning)),
            (Dependency, Some(Planning)),
            (Type, Some(Foundation)),
            (Utils, Some(Foundation)),
            (MockData, Some(Foundation)),
            (Service, Some(Logic)),
            (Hooks, Some(Logic)),
            (ComponentGen, Some(View)),
            (PageGen, Some(View)),
            (Layout, Some(View)),
            (StyleGen, Some(View)),
            (AppGen, Some(Assembly)),
            (Files, Some(Assembly)),
            (Plan, None),
            (Done, None),
            (Error, None),
        ];
        for (event_type, expected) in cases {
            let got = phase_for(*event_type).map(|p| match p {
                Phase::Traditional(t) => t,
            });
            assert_eq!(
                got, *expected,
                "phase for {event_type:?} should be {expected:?}"
            );
        }
    }

    #[test]
    fn traditional_phase_serializes_as_frontend_string() {
        let cases: &[(TraditionalPhase, &str)] = &[
            (Planning, "planning"),
            (Foundation, "foundation"),
            (Logic, "logic"),
            (View, "view"),
            (Assembly, "assembly"),
        ];
        for (variant, expected) in cases {
            let s = serde_json::to_string(variant).unwrap();
            assert_eq!(s, format!("\"{expected}\""));
        }
    }

    #[test]
    fn phase_top_level_serializes_untagged() {
        let phase = Phase::Traditional(TraditionalPhase::View);
        // untagged 应直接输出 "view" 而不是 { "Traditional": "view" }
        let s = serde_json::to_string(&phase).unwrap();
        assert_eq!(s, "\"view\"");
    }
}
