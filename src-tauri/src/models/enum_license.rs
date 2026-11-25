use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct EnumLicense {
    pub id: Option<i64>,
    pub spdx_id: String,
    pub full_name: String,
}
