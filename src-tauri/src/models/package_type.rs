use serde::{Deserialize, Serialize}; // serde 序列化/反序列化支持

/// 软件包类型枚举
/// 定义包的来源和构建方式的分类
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PackageType {
    /// 编译安装 (ID: 1) - 从源码编译的 AUR 包
    Compiled,
    /// 二进制包 (ID: 2) - 预编译的二进制包
    Binary,
    /// Git 仓库 (ID: 3) - 直接从 Git 仓库安装
    Git,
    /// AppImage (ID: 4) - AppImage 格式的便携应用
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
            _ => PackageType::Compiled, // 默认值
        }
    }
}
