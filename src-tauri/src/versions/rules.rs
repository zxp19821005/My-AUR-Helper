#[derive(Debug, Clone)]
pub struct CleanupRules {
    pub prefixes: Vec<String>,
    pub suffixes: Vec<String>,
}

impl Default for CleanupRules {
    fn default() -> Self {
        CleanupRules {
            prefixes: vec![
                "release-".to_string(),
                "v".to_string(),
                "V".to_string(),
                "version-".to_string(),
                "ver-".to_string(),
                "tag-".to_string(),
            ],
            suffixes: vec![
                "-release".to_string(),
                "-uos".to_string(),
                "-arch".to_string(),
                "-linux".to_string(),
                "-debian".to_string(),
                "-ubuntu".to_string(),
                "-fedora".to_string(),
                "-centos".to_string(),
                "-alpha".to_string(),
                "-beta".to_string(),
                "-rc".to_string(),
                "-pre".to_string(),
                "-dev".to_string(),
                "-stable".to_string(),
                "-latest".to_string(),
            ],
        }
    }
}

impl CleanupRules {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_prefix(&mut self, prefix: &str) {
        if !self.prefixes.contains(&prefix.to_string()) {
            self.prefixes.push(prefix.to_string());
        }
    }

    pub fn remove_prefix(&mut self, prefix: &str) {
        self.prefixes.retain(|p| p != prefix);
    }

    pub fn add_suffix(&mut self, suffix: &str) {
        if !self.suffixes.contains(&suffix.to_string()) {
            self.suffixes.push(suffix.to_string());
        }
    }

    pub fn remove_suffix(&mut self, suffix: &str) {
        self.suffixes.retain(|s| s != suffix);
    }

    pub fn clear_prefixes(&mut self) {
        self.prefixes.clear();
    }

    pub fn clear_suffixes(&mut self) {
        self.suffixes.clear();
    }

    pub fn extend_prefixes(&mut self, prefixes: &[&str]) {
        for p in prefixes {
            self.add_prefix(p);
        }
    }

    pub fn extend_suffixes(&mut self, suffixes: &[&str]) {
        for s in suffixes {
            self.add_suffix(s);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_rules() {
        let rules = CleanupRules::default();
        assert!(rules.prefixes.contains(&"v".to_string()));
        assert!(rules.prefixes.contains(&"V".to_string()));
        assert!(rules.suffixes.contains(&"-release".to_string()));
        assert!(rules.suffixes.contains(&"-uos".to_string()));
    }

    #[test]
    fn test_add_prefix() {
        let mut rules = CleanupRules::default();
        rules.add_prefix("custom-");
        assert!(rules.prefixes.contains(&"custom-".to_string()));
    }

    #[test]
    fn test_add_duplicate_prefix() {
        let mut rules = CleanupRules::default();
        let initial_len = rules.prefixes.len();
        rules.add_prefix("v");
        assert_eq!(rules.prefixes.len(), initial_len);
    }

    #[test]
    fn test_remove_prefix() {
        let mut rules = CleanupRules::default();
        rules.remove_prefix("v");
        assert!(!rules.prefixes.contains(&"v".to_string()));
    }

    #[test]
    fn test_add_suffix() {
        let mut rules = CleanupRules::default();
        rules.add_suffix("-custom");
        assert!(rules.suffixes.contains(&"-custom".to_string()));
    }

    #[test]
    fn test_remove_suffix() {
        let mut rules = CleanupRules::default();
        rules.remove_suffix("-release");
        assert!(!rules.suffixes.contains(&"-release".to_string()));
    }

    #[test]
    fn test_extend_prefixes() {
        let mut rules = CleanupRules::default();
        rules.extend_prefixes(&["app-", "tool-"]);
        assert!(rules.prefixes.contains(&"app-".to_string()));
        assert!(rules.prefixes.contains(&"tool-".to_string()));
    }

    #[test]
    fn test_extend_suffixes() {
        let mut rules = CleanupRules::default();
        rules.extend_suffixes(&["-win", "-mac"]);
        assert!(rules.suffixes.contains(&"-win".to_string()));
        assert!(rules.suffixes.contains(&"-mac".to_string()));
    }

    #[test]
    fn test_clear_rules() {
        let mut rules = CleanupRules::default();
        rules.clear_prefixes();
        rules.clear_suffixes();
        assert!(rules.prefixes.is_empty());
        assert!(rules.suffixes.is_empty());
    }
}