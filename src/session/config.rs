use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub struct SessionConfig {
    pub per_project_cap: usize,
    pub global_cap: usize,
}

impl SessionConfig {
    pub const DEFAULT_PER_PROJECT: usize = 20;
    pub const DEFAULT_GLOBAL: usize = 100;

    pub fn from_values(values: &HashMap<String, String>) -> Self {
        let per_project_cap = values
            .get("SESSION_PER_PROJECT_CAP")
            .and_then(|s| s.parse::<usize>().ok())
            .filter(|&n| n >= 1)
            .unwrap_or(Self::DEFAULT_PER_PROJECT);

        let global_cap = values
            .get("SESSION_GLOBAL_CAP")
            .and_then(|s| s.parse::<usize>().ok())
            .filter(|&n| n >= 1)
            .unwrap_or(Self::DEFAULT_GLOBAL);

        Self {
            per_project_cap,
            global_cap,
        }
    }
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            per_project_cap: Self::DEFAULT_PER_PROJECT,
            global_cap: Self::DEFAULT_GLOBAL,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uses_defaults_when_env_missing() {
        let cfg = SessionConfig::from_values(&HashMap::new());
        assert_eq!(cfg.per_project_cap, 20);
        assert_eq!(cfg.global_cap, 100);
    }

    #[test]
    fn reads_env_overrides() {
        let mut v = HashMap::new();
        v.insert("SESSION_PER_PROJECT_CAP".into(), "5".into());
        v.insert("SESSION_GLOBAL_CAP".into(), "50".into());
        let cfg = SessionConfig::from_values(&v);
        assert_eq!(cfg.per_project_cap, 5);
        assert_eq!(cfg.global_cap, 50);
    }

    #[test]
    fn zero_values_fall_back_to_defaults() {
        let mut v = HashMap::new();
        v.insert("SESSION_PER_PROJECT_CAP".into(), "0".into());
        let cfg = SessionConfig::from_values(&v);
        assert_eq!(cfg.per_project_cap, 20);
    }
}
