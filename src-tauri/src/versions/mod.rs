pub mod aur;
pub mod comparison;
pub mod git_version;
pub mod rules;
pub mod upstream;
pub mod utils;

pub use aur::AurVersion;
pub use comparison::compare_versions as compare_vercmp;
pub use comparison::is_prerelease;
pub use comparison::VersionComparison;
pub use git_version::extract_commit_count;
pub use git_version::is_r_format;
pub use git_version::remove_git_describe_metadata;
pub use upstream::UpstreamVersion;
pub use utils::{compare_versions, find_latest_version, is_outdated, sort_versions};
