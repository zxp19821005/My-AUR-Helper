mod factory;
mod gitee;
mod github;
mod gitlab;
mod http;
mod manual;
mod redirect;
mod trait_def;
mod utils;

pub use factory::{get_checker, CheckerSettings};
pub use trait_def::{CheckOptions, CheckResult, VersionChecker};
