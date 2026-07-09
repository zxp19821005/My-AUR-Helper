use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumLicense {
    pub id: Option<i64>,
    pub spdx_id: String,
    pub full_name: String,
}
