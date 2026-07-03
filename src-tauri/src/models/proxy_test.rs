use chrono::NaiveDateTime; // 日期时间类型
use serde::{Deserialize, Serialize}; // serde 序列化/反序列化支持

/// 代理测试结果
/// 对应数据库 proxies_test 表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyTest {
    /// 测试记录 ID，数据库自增主键
    pub id: Option<i64>,
    /// 关联的代理 ID
    pub proxy_id: i64,
    /// 测试执行时间
    pub test_time: Option<NaiveDateTime>,
    /// 平均延迟（毫秒）
    pub avg_latency: Option<i64>,
    /// 成功测试次数
    pub success_count: i64,
    /// 失败测试次数
    pub fail_count: i64,
}
