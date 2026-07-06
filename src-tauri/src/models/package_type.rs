use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(into = "i32", try_from = "i32")]
pub enum PackageType {
    Compiled,
    Binary,
    Git,
    AppImage,
}

impl PackageType {
    /// 将枚举转换为数字 ID（用于数据库存储）
    pub fn as_id(&self) -> i32 {
        match self {
            PackageType::Compiled => 1,
            PackageType::Binary => 2,
            PackageType::Git => 3,
            PackageType::AppImage => 4,
        }
    }

    /// 从数字 ID 创建枚举（用于数据库查询反序列化）
    /// @param id - 数字 ID
    /// @returns 对应的枚举值，未知 ID 默认返回 Compiled
    pub fn from_id(id: i32) -> Self {
        match id {
            1 => PackageType::Compiled,
            2 => PackageType::Binary,
            3 => PackageType::Git,
            4 => PackageType::AppImage,
            _ => PackageType::Compiled,
        }
    }
}

impl From<PackageType> for i32 {
    fn from(pt: PackageType) -> Self {
        pt.as_id()
    }
}

impl TryFrom<i32> for PackageType {
    type Error = String;
    fn try_from(id: i32) -> Result<Self, Self::Error> {
        Ok(PackageType::from_id(id))
    }
}
