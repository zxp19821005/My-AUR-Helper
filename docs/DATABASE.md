<!-- ================================================================ -->
<!-- My-AUR-Helper 数据库设计文档                                      -->
<!-- 本文档定义 SQLite 数据库的完整表结构、字段说明和表间关系          -->
<!-- 用于指导数据库层的编码实现，确保数据结构的一致性和完整性          -->
<!-- ================================================================ -->

<!-- ========== 概述：数据库选型和性能配置 ========== -->
# 数据库设计

## 概述

使用 SQLite 作为本地存储，通过 rusqlite 库操作。采用 WAL 模式提升并发性能。

<!-- ========== ER 图：展示所有表之间的关系 ========== -->
## ER 图

<!--
  实体关系说明：
  - software_info 是核心表，关联 aur_info、upstream_info
  - backup_software 和 cache_software 分别记录备份和缓存
  - proxies_info 关联 proxies_test 存储测试结果
  - enum_licenses 和 enum_programming_languages 是枚举数据
-->
```
┌──────────────────┐
│  software_info   │
│──────────────────│
│ software_id (PK) │
│ pkgname (UQ)     │
│ upstream_url     │
│ package_type_id  │
│ checker_type_id  │
│ is_outdated      │
│ check_test_ver   │
│ check_binary     │
│ auto_check       │
│ language_id (FK) │
│ version_regex    │
└────────┬─────────┘
         │
    ┌────┴────┐
    │         │
    ▼         ▼
┌──────────────┐  ┌──────────────┐
│  aur_info    │  │upstream_info │
│──────────────│  │──────────────│
│software_id(PK│  │software_id(PK│
│  FK→software │  │  FK→software │
│ pkgdesc      │  │upstream_ver  │
│ aur_version  │  │last_checked  │
│ license_id   │  │upstream_license│
│ last_updated │  │url_status    │
│ depends      │  └──────────────┘
│ makedepends  │
│ optdepends   │  ┌──────────────┐
└──────────────┘  │backup_software│
                  │ id (PK)      │
┌──────────────┐  │software_id(FK)│
│proxies_info  │  │ filename     │
│──────────────│  │ epoch        │
│proxy_id (PK) │  │ pkgrel       │
│ proxy_name   │  │ arch         │
│ proxy_type   │  │ subdirectory │
│ url          │  └──────────────┘
│ is_active    │
└──────┬───────┘  ┌──────────────┐
       │          │cache_software│
       │          │──────────────│
       │          │ id (PK)      │
       │          │software_id(FK)│
       │          │ filename     │
       │          │ epoch        │
       │          │ pkgrel       │
       │          │ arch         │
       │          │cache_directory│
       │          └──────────────┘
       │
       ▼
┌──────────────┐
│ proxies_test │
│──────────────│
│ id (PK)      │
│proxy_id (FK) │
│ test_time    │
│ avg_latency  │
│ success_count│
│ fail_count   │
└──────────────┘

┌──────────────────┐  ┌──────────────────────┐
│  enum_licenses   │  │enum_programming_langs │
│──────────────────│  │──────────────────────│
│ id (PK)          │  │ id (PK)              │
│ spdx_id (UQ)     │  │ name (UQ)            │
│ full_name        │  │ short_name           │
└──────────────────┘  └──────────────────────┘
```

<!-- ========== 表结构：各表的详细字段定义 ========== -->
## 表结构

<!-- software_info：软件包核心信息表，每行代表一个软件包 -->
### software_info
软件核心信息表，一个软件包对应一条记录。

| 字段 | 类型 | 说明 |
|------|------|------|
| software_id | INTEGER PK | 自增主键 |
| pkgname | TEXT UNIQUE | 包名（如 gitify-bin） |
| upstream_url | TEXT | 上游项目 URL |
| package_type_id | INTEGER | 包类型 (1=编译版本, 2=二进制版本, 3=git版本, 4=AppImage版本) |
| checker_type_id | INTEGER | 上游版本检查器 (1=github_release, 2=github_tag, 3=gitee, 4=gitlab, 5=redirect, 6=http, 7=manual) |
| is_outdated | INTEGER | 是否过期 (有可用更新) |
| check_test_versions | INTEGER | 是否检查测试/pre-release 版本 |
| check_binary_files | INTEGER | 是否检查二进制文件 |
| auto_check_enabled | INTEGER | 是否启用自动检查 |
| language_id | TEXT | 编程语言 ID 数组 (JSON 格式，如 [1,2,3]) |
| version_extract_regex | TEXT | 版本提取正则表达式 |

<!-- aur_info：AUR 包详细信息，通过 AUR RPC 接口获取的元数据 -->
### aur_info
AUR 软件包详细信息，通过 AUR RPC 接口获取。

| 字段 | 类型 | 说明 |
|------|------|------|
| software_id | INTEGER PK FK | 关联 software_info.software_id |
| pkgdesc | TEXT | 软件包描述 |
| aur_version | TEXT | AUR 中的版本号 |
| license_id | TEXT | License 列表 (JSON 数组格式，如 `["MIT", "GPL-3.0"]`) |
| last_updated | INTEGER | AUR 最后更新时间 (Unix 时间戳) |
| depends | TEXT | 运行依赖 (JSON 数组) |
| makedepends | TEXT | 编译依赖 (JSON 数组) |
| optdepends | TEXT | 可选依赖 (JSON 数组) |
| out_of_date | INTEGER | AUR 标记是否过期 |

<!-- upstream_info：上游版本追踪信息 -->
### upstream_info
上游版本信息。

| 字段 | 类型 | 说明 |
|------|------|------|
| software_id | INTEGER PK FK | 关联 software_info.software_id |
| upstream_version | TEXT | 上游最新版本 |
| upstream_license_id | TEXT | 上游 License 列表 (JSON 数组格式，如 `["MIT", "GPL-3.0"]`) |
| last_checked | INTEGER | 最后检查时间 (Unix 时间戳) |
| upstream_url_status | TEXT | 上游 URL 验证状态 (ok/not_found/forbidden/redirected/server_error/timeout/connection_error/other_error) |

<!-- backup_software：备份文件记录，从 pacman 缓存复制到备份目录 -->
### backup_software
备份文件信息。

| 字段 | 类型 | 说明 |
|------|------|------|
| id | INTEGER PK | 自增主键 |
| software_id | INTEGER FK | 关联 software_info.software_id |
| filename | TEXT | 文件名 (如 gitify-bin-1.0.0-1-x86_64.pkg.tar.zst) |
| epoch | INTEGER | epoch (默认 0) |
| pkgrel | TEXT | pkgrel |
| arch | TEXT | 架构 (x86_64/aarch64/any) |
| subdirectory | TEXT | 所在子目录 |

<!-- cache_software：本地缓存文件信息 -->
### cache_software
缓存文件信息。

| 字段 | 类型 | 说明 |
|------|------|------|
| id | INTEGER PK | 自增主键 |
| software_id | INTEGER FK | 关联 software_info.software_id |
| filename | TEXT | 文件名 |
| epoch | INTEGER | epoch (默认 0) |
| pkgrel | TEXT | pkgrel |
| arch | TEXT | 架构 |
| cache_directory | TEXT | 缓存目录路径 |

<!-- proxies_info：代理源信息，可能来自 Greasyfork 脚本 -->
### proxies_info
代理信息。

| 字段 | 类型 | 说明 |
|------|------|------|
| proxy_id | INTEGER PK | 自增主键 |
| proxy_name | TEXT | 代理名称 |
| proxy_type | TEXT | 代理类型 (download/clone/raw/ssh) |
| url | TEXT UNIQUE | 代理 URL |
| is_active | INTEGER | 是否启用 |

<!-- proxies_test：代理延迟测试结果记录 -->
### proxies_test
代理测试结果。

| 字段 | 类型 | 说明 |
|------|------|------|
| id | INTEGER PK | 自增主键 |
| proxy_id | INTEGER FK | 关联 proxies_info.proxy_id |
| test_time | TEXT | 测试时间 |
| avg_latency | INTEGER | 平均延迟 (ms) |
| success_count | INTEGER | 成功次数 |
| fail_count | INTEGER | 失败次数 |

<!-- enum_licenses：许可证枚举表，基于 SPDX 标准 -->
### enum_licenses
许可证枚举表，从 SPDX 同步。

| 字段 | 类型 | 说明 |
|------|------|------|
| id | INTEGER PK | 自增主键 |
| spdx_id | TEXT UNIQUE | SPDX 标识符 (如 MIT, Apache-2.0) |
| full_name | TEXT | 完整名称 |

<!-- enum_programming_languages：编程语言枚举，用于关联软件包 -->
### enum_programming_languages
编程语言枚举表。

| 字段 | 类型 | 说明 |
|------|------|------|
| id | INTEGER PK | 自增主键 |
| name | TEXT UNIQUE | 语言名称 (如 "Rust", "Python") |
| short_name | TEXT | 简称 (如 "rs", "py") |

<!-- logs：应用运行时日志，用于调试和审计 -->
### logs
应用日志表。

| 字段 | 类型 | 说明 |
|------|------|------|
| id | INTEGER PK | 自增主键 |
| level | TEXT | 日志级别 (ERROR/WARN/INFO/DEBUG) |
| message | TEXT | 日志消息 |
| module | TEXT NULL | 模块名 |
| created_at | TEXT | 创建时间 |

<!-- settings：键值对形式的应用配置 -->
### settings
应用设置表。

| 字段 | 类型 | 说明 |
|------|------|------|
| id | INTEGER PK | 自增主键 |
| key | TEXT UNIQUE | 设置键 |
| value | TEXT | 设置值 |
| description | TEXT | 描述 |
| category | TEXT | 分类 |
| created_at | TEXT | 创建时间 |