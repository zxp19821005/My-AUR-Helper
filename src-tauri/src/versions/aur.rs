use log::debug;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AurVersion {
    pub epoch: Option<u32>,
    pub version: String,
    pub pkgrel: Option<String>,
    pub normalized_version: String,
}

impl AurVersion {
    pub fn parse(raw: &str) -> Self {
        let raw = raw.trim();
        let (epoch_part, rest) = if let Some(colon_idx) = raw.find(':') {
            let epoch_str = &raw[..colon_idx];
            let epoch = epoch_str.parse::<u32>().ok();
            (epoch, &raw[colon_idx + 1..])
        } else {
            (None, raw)
        };

        let (version_part, pkgrel_part) = if let Some(dash_idx) = rest.rfind('-') {
            let potential_pkgrel = &rest[dash_idx + 1..];
            if potential_pkgrel.chars().all(|c| c.is_ascii_digit()) {
                let pkgrel = Some(potential_pkgrel.to_string());
                let version = rest[..dash_idx].to_string();
                (version, pkgrel)
            } else {
                (rest.to_string(), None)
            }
        } else {
            (rest.to_string(), None)
        };

        let normalized_version = Self::normalize(&version_part);

        debug!("解析AUR版本 '{}': epoch={:?}, version={}, pkgrel={:?}, normalized={}",
               raw, epoch_part, version_part, pkgrel_part, normalized_version);

        AurVersion {
            epoch: epoch_part,
            version: version_part,
            pkgrel: pkgrel_part,
            normalized_version,
        }
    }

    pub fn normalize(version: &str) -> String {
        version.replace('-', "_")
    }

    pub fn to_string_full(&self) -> String {
        let mut result = String::new();
        if let Some(e) = self.epoch {
            result.push_str(&format!("{}:", e));
        }
        result.push_str(&self.version);
        if let Some(ref p) = self.pkgrel {
            result.push('-');
            result.push_str(p);
        }
        result
    }

    pub fn to_string_comparable(&self) -> String {
        let mut result = String::new();
        if let Some(e) = self.epoch {
            result.push_str(&format!("{}:", e));
        }
        result.push_str(&self.normalized_version);
        result
    }

    pub fn validate(&self) -> bool {
        if self.version.is_empty() {
            return false;
        }
        for c in self.version.chars() {
            if !c.is_ascii_alphanumeric() && c != '.' && c != '_' && c != '+' && c != ':' && c != '~' {
                return false;
            }
        }
        true
    }
}

impl std::fmt::Display for AurVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string_full())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic() {
        let v = AurVersion::parse("1.2.3");
        assert_eq!(v.epoch, None);
        assert_eq!(v.version, "1.2.3");
        assert_eq!(v.pkgrel, None);
        assert_eq!(v.normalized_version, "1.2.3");
    }

    #[test]
    fn test_parse_with_pkgrel() {
        let v = AurVersion::parse("1.2.3-1");
        assert_eq!(v.epoch, None);
        assert_eq!(v.version, "1.2.3");
        assert_eq!(v.pkgrel, Some("1".to_string()));
        assert_eq!(v.normalized_version, "1.2.3");
    }

    #[test]
    fn test_parse_with_epoch() {
        let v = AurVersion::parse("2:1.2.3");
        assert_eq!(v.epoch, Some(2));
        assert_eq!(v.version, "1.2.3");
        assert_eq!(v.pkgrel, None);
        assert_eq!(v.normalized_version, "1.2.3");
    }

    #[test]
    fn test_parse_full() {
        let v = AurVersion::parse("2:1.2.3-4");
        assert_eq!(v.epoch, Some(2));
        assert_eq!(v.version, "1.2.3");
        assert_eq!(v.pkgrel, Some("4".to_string()));
        assert_eq!(v.normalized_version, "1.2.3");
    }

    #[test]
    fn test_parse_hyphen_to_underscore() {
        let v = AurVersion::parse("1.2.3-alpha");
        assert_eq!(v.version, "1.2.3-alpha");
        assert_eq!(v.normalized_version, "1.2.3_alpha");
    }

    #[test]
    fn test_parse_complex() {
        let v = AurVersion::parse("3:2.4.5-beta1-2");
        assert_eq!(v.epoch, Some(3));
        assert_eq!(v.version, "2.4.5-beta1");
        assert_eq!(v.pkgrel, Some("2".to_string()));
        assert_eq!(v.normalized_version, "2.4.5_beta1");
    }

    #[test]
    fn test_to_string_full() {
        let v = AurVersion::parse("2:1.2.3-4");
        assert_eq!(v.to_string_full(), "2:1.2.3-4");
    }

    #[test]
    fn test_to_string_comparable() {
        let v = AurVersion::parse("2:1.2.3-beta-4");
        assert_eq!(v.to_string_comparable(), "2:1.2.3_beta");
    }

    #[test]
    fn test_validate() {
        assert!(AurVersion::parse("1.2.3").validate());
        assert!(AurVersion::parse("1.2.3-1").validate());
        assert!(AurVersion::parse("2:1.2.3").validate());
        assert!(AurVersion::parse("1.2.3_alpha").validate());
        assert!(AurVersion::parse("1.2.3+git").validate());
    }
}