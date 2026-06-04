# x-scraper-rs

[English](#english) | [中文](#中文)

---

## English

A high-performance Rust implementation of Twitter/X GraphQL API scraper, inspired by [twscrape](https://github.com/vladkens/twscrape).

### Features

- Pure Rust implementation — no Python/Node.js dependency
- CLI tool for account management and API queries
- Pure Rust parsing of Twitter API responses (tweets, users, trends, cards)
- SQLite-based account storage with session management
- GraphQL parameter builder for all major Twitter endpoints
- 71 unit tests covering parsing, API building, and account management

### Quick Start

#### Install

```bash
# From source
git clone https://github.com/map-A/x-scraper-rs.git
cd x-scraper-rs
cargo build --release

# Binary will be at target/release/twscrape
```

#### Add Account with Cookies

```bash
# Add account from browser cookies
twscrape add-cookie myusername "auth_token=xxx; ct0=yyy"

# Or add from file
echo "user1:pass1:email1@test.com:ep1" > accounts.txt
twscrape add-accounts accounts.txt username:password:email:email_password
```

#### List Accounts

```bash
twscrape accounts
twscrape stats
```

#### Use as Library

```rust
use serde_json::Value;
use twscrape_rs::{api_gql_op_rs, api_build_kv_rs, api_merge_features_rs, types};

fn parse_tweet_response(json: &Value) {
    let op = api_gql_op_rs("tweet_details").unwrap();
    let kv = api_build_kv_rs("tweet_details", Some("1920384454767374343"), None).unwrap();
    let features = api_merge_features_rs(None);

    let rep = types::to_old_rep(json);
    let tweets = rep.get("tweets").unwrap();
    let users = rep.get("users").unwrap();
    let res = serde_json::json!({"tweets": tweets, "users": users});

    for (_key, tweet_obj) in tweets.as_object().unwrap() {
        if let Some(tweet) = types::Tweet::parse(tweet_obj, &res) {
            let username = tweet.user.as_ref().map(|u| u.username.as_str()).unwrap_or("?");
            println!("@{username}: {}", tweet.raw_content);
        }
    }
}
```

### CLI Commands

```bash
# Account management
twscrape add-cookie <username> [cookies]     # Add account with cookies
twscrape add-accounts <file> <format>        # Add accounts from file
twscrape del-accounts <usernames...>         # Delete accounts
twscrape accounts                            # List all accounts
twscrape stats                               # Show usage stats
twscrape relogin <usernames...>              # Re-login accounts
twscrape relogin-failed                      # Retry failed logins
twscrape reset-locks                         # Reset all locks
twscrape delete-inactive                     # Delete inactive accounts

# API queries (require async HTTP client integration)
twscrape search <query> [--limit N]
twscrape tweet-details <id>
twscrape user-by-login <username>
twscrape followers <user_id> [--limit N]
twscrape following <user_id> [--limit N]
twscrape trends <trend_id> [--limit N]
# ... and more (see --help)
```

### Architecture

```
src/
├── main.rs          # CLI binary
├── lib.rs           # Library root with re-exports
├── types.rs         # Core data structures (Tweet, User, Trend, Card)
├── api.rs           # GraphQL parameter builders
├── account.rs       # Account management & SQL generation
├── db.rs            # SQLite migration helpers
├── utils.rs         # Cookie/proxy parsing
├── queue_client.rs  # Rate limit tracking
├── logger.rs        # Logging utilities
└── xclid.rs         # X-ClId generation
tests/
└── mocked-data/     # JSON fixtures for unit tests
examples/
└── test_search.rs   # End-to-end API test with real cookies
```

### Testing

```bash
cargo test --lib                     # Run all 71 tests
cargo test --lib types::tests        # Run model parsing tests
cargo test --lib api::tests          # Run API builder tests
```

### Dependencies

| Crate | Purpose |
|-------|---------|
| `reqwest` | HTTP client |
| `rusqlite` | SQLite database |
| `serde_json` | JSON parsing |
| `clap` | CLI argument parsing |
| `tokio` | Async runtime |

### License

MIT

---

## 中文

高性能 Rust 实现的 Twitter/X GraphQL API 爬虫，灵感来自 [twscrape](https://github.com/vladkens/twscrape)。

### 特性

- 纯 Rust 实现 — 无 Python/Node.js 依赖
- 命令行工具，支持账户管理和 API 查询
- 纯 Rust 解析 Twitter API 响应（推文、用户、趋势、卡片）
- 基于 SQLite 的账户存储和会话管理
- 所有主要 Twitter 端点的 GraphQL 参数构建器
- 71 个单元测试覆盖解析、API 构建和账户管理

### 快速开始

#### 安装

```bash
# 从源码编译
git clone https://github.com/map-A/x-scraper-rs.git
cd x-scraper-rs
cargo build --release

# 二进制文件位于 target/release/twscrape
```

#### 使用 Cookie 添加账户

```bash
# 从浏览器 Cookie 添加账户
twscrape add-cookie 我的用户名 "auth_token=xxx; ct0=yyy"

# 或从文件批量添加
echo "user1:pass1:email1@test.com:ep1" > accounts.txt
twscrape add-accounts accounts.txt username:password:email:email_password
```

#### 查看账户

```bash
twscrape accounts    # 列出所有账户
twscrape stats       # 查看使用统计
```

#### 作为库使用

```rust
use serde_json::Value;
use twscrape_rs::{api_gql_op_rs, api_build_kv_rs, api_merge_features_rs, types};

fn parse_tweet_response(json: &Value) {
    let op = api_gql_op_rs("tweet_details").unwrap();
    let kv = api_build_kv_rs("tweet_details", Some("1920384454767374343"), None).unwrap();
    let features = api_merge_features_rs(None);

    let rep = types::to_old_rep(json);
    let tweets = rep.get("tweets").unwrap();
    let users = rep.get("users").unwrap();
    let res = serde_json::json!({"tweets": tweets, "users": users});

    for (_key, tweet_obj) in tweets.as_object().unwrap() {
        if let Some(tweet) = types::Tweet::parse(tweet_obj, &res) {
            let username = tweet.user.as_ref().map(|u| u.username.as_str()).unwrap_or("?");
            println!("@{username}: {}", tweet.raw_content);
        }
    }
}
```

### 命令行命令

```bash
# 账户管理
twscrape add-cookie <用户名> [cookies]     # 用 cookie 添加账户
twscrape add-accounts <文件> <格式>         # 从文件添加账户
twscrape del-accounts <用户名...>           # 删除账户
twscrape accounts                          # 列出所有账户
twscrape stats                             # 查看使用统计
twscrape relogin <用户名...>               # 重新登录账户
twscrape relogin-failed                    # 重试失败的登录
twscrape reset-locks                       # 重置所有锁
twscrape delete-inactive                   # 删除非活跃账户

# API 查询（需要集成异步 HTTP 客户端）
twscrape search <查询> [--limit 数量]
twscrape tweet-details <ID>
twscrape user-by-login <用户名>
twscrape followers <用户ID> [--limit 数量]
twscrape following <用户ID> [--limit 数量]
twscrape trends <趋势ID> [--limit 数量]
# ... 更多命令请查看 --help
```

### 项目结构

```
src/
├── main.rs          # CLI 可执行文件
├── lib.rs           # 库入口，公共导出
├── types.rs         # 核心数据结构（Tweet, User, Trend, Card）
├── api.rs           # GraphQL 参数构建器
├── account.rs       # 账户管理 & SQL 生成
├── db.rs            # SQLite 迁移辅助
├── utils.rs         # Cookie/代理解析
├── queue_client.rs  # 速率限制跟踪
├── logger.rs        # 日志工具
└── xclid.rs         # X-ClId 生成
tests/
└── mocked-data/     # JSON 测试夹具
examples/
└── test_search.rs   # 使用真实 cookies 的端到端 API 测试
```

### 测试

```bash
cargo test --lib                     # 运行所有 71 个测试
cargo test --lib types::tests        # 运行模型解析测试
cargo test --lib api::tests          # 运行 API 构建器测试
```

### 依赖

| Crate | 用途 |
|-------|------|
| `reqwest` | HTTP 客户端 |
| `rusqlite` | SQLite 数据库 |
| `serde_json` | JSON 解析 |
| `clap` | 命令行参数解析 |
| `tokio` | 异步运行时 |

### 许可证

MIT
