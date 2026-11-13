# OPC — Reference

This file consolidates the project's detailed architecture, API, operations, and changelog documentation into a single indexed reference.

---

## 目录 / Index

### Architecture / 系统架构

- [技术栈 / Tech Stack](#tech-stack)
- [项目结构 / Project Structure](#project-structure)
- [数据库设计 / Database Design](#database-design)
- [权限体系 / Permission System](#permission-system)
- [系统功能蓝图 / System Blueprint](#system-blueprint)

### API & CLI

- [API 接口参考 / API Reference](#api-reference)
- [CLI 使用说明 / CLI Guide](#cli-guide)

### Features

- [AI 防坑卫士 / AI Contract Review](#ai-contract-review)
- [文件管理 / File Management](#file-management)

### Operations / 部署运维


### Changelog

- [更新日志 / Changelog](#changelog)

---

## Tech Stack

> 源自 `docs/01-system-architecture/01-tech-stack.md`

### 前端技术栈

#### 核心框架

##### React 18
- **选择理由**: 成熟生态、Concurrent Features、Hooks API
- **使用特性**: Function Components、Hooks、Context API
- **配置**: TypeScript 严格模式 (`strict: true`)

##### Vite
- 极快冷启动、热模块替换 (HMR)
- ESM 开发服务器、优化的生产构建

#### UI 框架

##### Tailwind CSS + shadcn/ui
- 50+ 高质量 UI 组件 (基于 Radix UI)
- HSL 色彩空间设计 token
- Dialog, Form, Table, Card, Badge, Toast 等

#### 状态管理

##### TanStack Query v5
- 服务端状态管理、自动缓存
- 请求去重、后台自动重载

##### React Hook Form + Zod
- 高性能表单管理、最小化重渲染
- 运行时类型验证、编译时类型推断

#### 数据可视化

##### Recharts
- LineChart / BarChart / PieChart / AreaChart
- React 原生支持、声明式 API

#### 路由

##### React Router v6
- 嵌套路由、动态路由
- `ProtectedRoute` 路由守卫

---

### 后端技术栈

#### 核心框架

##### Rust + Axum 0.8
- **选择理由**: 系统级性能、内存安全、零成本抽象
- **架构模式**: 模块化路由 + 中间件管道
- **异步运行时**: Tokio

```rust
// lib.rs — 应用路由构建
pub fn build_app(pool: SqlitePool) -> Router {
    let api_routes = api::router(pool.clone())
        .layer(middleware::from_fn_with_state(pool.clone(), auth::middleware::require_auth));

    Router::new()
        .route("/health", get(health_check))
        .merge(auth_router)
        .nest("/api/v1", api_routes.clone())
        .nest("/api", api_routes)
}
```

#### 数据库

##### SQLite + SQLx 0.8
- **选择理由**: 零配置、嵌入式、适合单人公司场景
- **特性**: 编译时 SQL 检查、异步 I/O
- **Schema 管理**: `schema.rs` 中定义 50+ 张表的 DDL
- **多租户**: 所有查询自动注入 `company_id` 过滤

#### 认证

##### Session Token 认证
```
GET /auth/login → 跳转到 SSO/OAuth 登录
OAuth 回调后写入 sessions 表
Authorization: Bearer {session_id} → require_auth 中间件验证
```
- 登录方式: 浏览器跳转式 SSO/OAuth
- 会话: UUID Session 存入 `sessions` 表
- 中间件: `require_auth` 解析 token → 注入 `AuthUser { user_id, company_id }`

#### AI 引擎

##### OpenAI 兼容文本 API
- **用途**: 政策分析、合同审查、经营数据分析、文本生成
- **协议**: 标准 `/v1/chat/completions` 接口
- **默认上游**: DeepSeek `deepseek-v4-flash`
- **可选模型**: `deepseek-v4-pro` 或其它 OpenAI 兼容文本模型
- **触发方式**: 合同创建/更新后 `tokio::spawn` 异步执行
- **降级策略**: 未配置 `AI_API_URL` → 标记 `unavailable`，恢复后自动补全

##### 文档解析 / OCR 后端
- **用途**: 证照/发票/合同 OCR 入库
- **当前后端**: `DOCUMENT_PARSER_BACKEND=ollama-vision`
- **预留后端**: `liteparse`、`paddleocr`（PaddleOCR 1.6 接入时使用）
- **部署**: 当前实现仍走 Ollama 本地视觉模型 (`ollama pull paddleocr-vl:1.5`)
- **特性**:
  - 0.9B 超轻量 — CPU 即可运行
  - 111 种语言支持
  - 印章识别 + 不规则文档校正
  - 表格/公式/图表结构化提取
- **前端可选**: 请求级别指定 `model` 字段切换引擎

```rust
// OCR 模型选择优先级
let ollama_model = req.model.clone().unwrap_or_else(|| {
    env::var("OLLAMA_MODEL").unwrap_or_else(|_| "paddleocr-vl:1.5".to_string())
});
```

#### API 模块架构

```
backend/src/api/
├── mod.rs          # 路由注册 + 模块导出
├── generic.rs      # 通用 CRUD (40+ 表) + AI 合同审查
├── ai.rs           # AI 端点 (parse-document, contract-review)
├── assets.rs       # 资产管理 (操作日志)
├── invoices.rs     # 发票管理 (操作日志)
├── employees.rs    # 员工管理 (操作日志)
├── budgets.rs      # 预算管理 (操作日志)
├── bank.rs         # 银行账户 + 交易 (操作日志)
├── companies.rs    # 公司管理
├── files.rs        # 文件上传/下载
├── policies.rs     # 政策智能 + 匹配算法
├── global_opc_hubs.rs  # OPC 出海沙盘
└── error.rs        # 统一错误处理
```

#### 通用 CRUD 引擎 (generic.rs)

核心设计模式 — 一套引擎驱动 40+ 张表的 CRUD：

```rust
// 动态列映射 INSERT
generic_create(&pool, "contracts", body, &["contract_name"], &company_id, &user_id)

// 特性:
// - 自动注入 company_id (多租户隔离)
// - 必填字段校验
// - 金额非负校验 (amount, budget_amount, base_salary...)
// - 外键引用校验 (project_id, contract_id, employee_id...)
// - 操作日志记录 (insert → operation_logs)
// - SQL 注入防护 (列名白名单 is_valid_column_name)
```

---

### 安全体系

#### 多租户隔离
- 所有 API 查询自动注入 `WHERE company_id = ?`
- `AuthUser` 中间件解析 session → 获取 `company_id`
- 跨租户访问返回 404 (不泄露数据存在性)

#### CORS 白名单
```bash
ALLOWED_ORIGINS=http://localhost:5173,http://localhost:8080
```

#### 财务安全
- 金额字段强制 `≥ 0` (防负数注入)
- 外键引用校验 (防孤立记录)
- 事务包裹 (atomicity)

---

### 部署架构

#### 开发环境
```bash
cargo run          # 后端 (默认 :8987)
npm run dev        # 前端 (默认 :5173)
ollama serve       # OCR 服务 (默认 :11434)
```

#### 生产环境
- 后端: Rust 编译为单一二进制、systemd 管理
- 前端: `npm run build` → 静态文件 → Cloudflare Pages / Nginx
- 数据库: SQLite 文件 (自动备份)
- OCR: Ollama + PaddleOCR-VL 1.5

---

*最后更新: 2026-03*

---

## Project Structure

> 源自 `docs/01-system-architecture/02-project-structure.md`

### 目录总览

```
opc/
├── backend/                    # Rust 后端 (Axum 0.8)
│   ├── Cargo.toml              # Rust 依赖管理
│   ├── src/
│   │   ├── main.rs             # 入口：初始化 DB、CORS、播种、启动服务
│   │   ├── lib.rs              # 应用路由构建 (build_app)
│   │   ├── api/                # API 路由层
│   │   │   ├── mod.rs          # 模块注册 + 公共导出
│   │   │   ├── generic.rs      # 通用 CRUD 引擎 (40+ 表)
│   │   │   ├── ai.rs           # AI 端点 (OCR + 合同审查)
│   │   │   ├── assets.rs       # 资产管理
│   │   │   ├── invoices.rs     # 发票管理
│   │   │   ├── employees.rs    # 员工管理
│   │   │   ├── budgets.rs      # 预算管理
│   │   │   ├── bank.rs         # 银行账户与交易
│   │   │   ├── companies.rs    # 公司管理
│   │   │   ├── files.rs        # 文件上传下载
│   │   │   ├── policies.rs     # 政策智能 + 匹配
│   │   │   ├── global_opc_hubs.rs  # OPC 出海沙盘
│   │   │   └── error.rs        # 统一错误处理
│   │   ├── auth/               # 认证模块
│   │   │   ├── mod.rs          # 路由 (login/register/logout/me)
│   │   │   └── middleware.rs   # require_auth 中间件
│   │   └── db/                 # 数据层
│   │       ├── mod.rs          # 连接池初始化 + 迁移
│   │       ├── schema.rs       # DDL (50+ 表)
│   │       ├── models.rs       # Rust 数据结构
│   │       ├── seed_policies.rs    # 全球政策播种 (60+ 条)
│   │       └── seed_opc_hubs.rs    # OPC 圣地播种 (5+ 地区)
│   ├── tests/                  # E2E 集成测试
│   │   ├── common/mod.rs       # TestApp 测试框架
│   │   ├── s01_company_setup.rs
│   │   ├── s02_auth_session.rs
│   │   ├── ...
│   │   ├── s15_global_opc_tax.rs
│   │   └── s16_mandatory_contract_review.rs
│   └── uploads/                # 上传文件存储目录
│
├── frontend/                   # React 18 前端
│   ├── package.json
│   ├── vite.config.ts
│   ├── src/
│   │   ├── main.tsx            # 入口
│   │   ├── App.tsx             # 路由定义
│   │   ├── pages/              # 页面组件
│   │   │   ├── Dashboard.tsx   # 仪表盘
│   │   │   ├── Contracts.tsx   # 合同管理 (含 AI 风险列)
│   │   │   ├── Invoices.tsx    # 发票管理
│   │   │   ├── Assets.tsx      # 资产管理
│   │   │   ├── Employees.tsx   # 员工管理
│   │   │   ├── Policies.tsx    # 政策智能
│   │   │   ├── OPCSandbox.tsx  # OPC 出海沙盘
│   │   │   └── ...
│   │   ├── components/         # UI 组件
│   │   │   ├── ui/             # shadcn/ui 基础组件 (50+)
│   │   │   ├── contracts/      # 合同相关组件
│   │   │   │   └── ContractDialog.tsx
│   │   │   ├── common/         # 通用组件
│   │   │   │   └── AutoParseButton.tsx
│   │   │   └── ...
│   │   ├── contexts/           # React Context
│   │   │   └── AuthContext.tsx  # 认证上下文
│   │   ├── hooks/              # 自定义 Hooks
│   │   │   ├── useDocumentParser.ts
│   │   │   ├── useSmartWorkbench.ts
│   │   │   └── useFileUpload.ts
│   │   ├── lib/                # 工具库
│   │   │   ├── api.ts          # REST API 客户端
│   │   │   ├── fieldNormalizer.ts
│   │   │   └── dateUtils.ts
│   │   └── integrations/
│   │       └── api/client.ts   # API 基础配置
│   └── public/
│
└── docs/                       # 项目文档
    ├── README.md               # 文档首页
    ├── REFERENCE.md            # 详细参考文档（本文件）
    └── architecture/
        ├── system_architecture.mmd
        ├── data_flow.mmd
        └── component_architecture.mmd
```

### 后端模块职责

#### `api/generic.rs` — 通用 CRUD 引擎
最核心的模块，一套引擎驱动 40+ 张表：
- `generic_create` / `generic_update` / `generic_delete` / `generic_list` / `generic_get`
- 自动注入 `company_id` (多租户隔离)
- 必填字段校验 + 金额非负校验
- 外键引用校验
- 操作日志记录
- SQL 注入防护 (列名白名单)
- **合同专用**: 创建/更新后 `tokio::spawn` 异步 AI 审查

#### `api/ai.rs` — AI 引擎
- `POST /api/ai/parse-document` — OCR 文档解析 (PaddleOCR-VL 1.5)
- `POST /api/v1/ai/contract-review` — AI 合同审查
- 支持请求级别模型选择

#### `api/policies.rs` — 政策智能
- 政策匹配算法 (国家 +40, 学历 +15, 海归 +15)
- 政策申请生命周期 (preparing → submitted → approved)
- 租户隔离 (申请数据按 company_id 隔离)

#### `db/schema.rs` — 数据库 DDL
- 50+ 张表定义
- 索引定义
- 默认值与约束

### 前端架构模式

#### 页面组件
每个页面遵循统一模式：
1. `useQuery` 获取数据
2. `useMutation` 处理增删改
3. 状态过滤 + 搜索
4. 统计卡片 + 数据表格
5. Dialog 表单 (React Hook Form + Zod)

#### API 客户端
```typescript
// lib/api.ts — 统一 REST API 封装
const api = {
  get: (url) => fetch(`${BASE_URL}${url}`, { headers: authHeaders() }),
  post: (url, data) => fetch(url, { method: 'POST', body: JSON.stringify(data) }),
  put: (url, data) => fetch(url, { method: 'PUT', body: JSON.stringify(data) }),
  delete: (url) => fetch(url, { method: 'DELETE' }),
};
```

---

*最后更新: 2026-03*

---

## Database Design

> 源自 `docs/01-system-architecture/03-database-design.md`

### 概述

OPC 使用 **SQLite** 作为嵌入式数据库，零配置部署。所有表定义在 `backend/src/db/schema.rs` 中，服务启动时自动执行 DDL。

- **数据库引擎**: SQLite 3.x
- **驱动**: SQLx 0.8 (异步)
- **Schema 管理**: 应用内 DDL (非迁移文件)
- **多租户**: 通过 `company_id` 列实现数据隔离
- **总计**: 51 张表

### 表清单

#### 用户与认证 (3)
| 表名 | 说明 |
|---|---|
| `users` | 用户账户 (email, password_hash) |
| `sessions` | 会话 Token (session_token) |
| `user_roles` | 用户角色 |

#### 企业管理 (5)
| 表名 | 说明 |
|---|---|
| `companies` | 公司信息 (含 country 字段) |
| `company_members` | 用户-公司多对多关系 |
| `company_info` | 公司详细信息 (创始人背景等) |
| `branches` | 分支机构 |
| `milestones` | 里程碑 |

#### 财务管理 (8)
| 表名 | 说明 |
|---|---|
| `invoices` | 发票 (进项/销项) |
| `payments` | 付款记录 |
| `salary_records` | 工资记录 |
| `rent_records` | 租金记录 |
| `utilities` | 水电费 |
| `budgets` | 预算 |
| `tax_records` | 税务记录 |
| `exchange_rates` | 汇率 |

#### 资产管理 (3)
| 表名 | 说明 |
|---|---|
| `assets` | 资产 (固定/虚拟) |
| `asset_records` | 资产操作记录 |
| `asset_categories` | 资产分类 |

#### 合同管理 (1)
| 表名 | 说明 | 特殊字段 |
|---|---|---|
| `contracts` | 合同 | `ai_review_status`, `ai_risk_level`, `ai_review_result`, `ai_reviewed_at`, `counterparty_type` |

#### 项目与申报 (7)
| 表名 | 说明 |
|---|---|
| `projects` | 项目 |
| `applications` | 项目申报 |
| `application_materials` | 申报材料 |
| `material_library` | 材料库 |
| `material_templates` | 材料模板 |
| `material_versions` | 材料版本 |
| `licenses` | 证照 |

#### 云成本与 SaaS (5)
| 表名 | 说明 |
|---|---|
| `cloud_providers` | 云厂商 |
| `cloud_resources` | 云资源 |
| `cloud_bills` | 云账单 |
| `software_subscriptions` | SaaS 订阅 |
| `domain_subscriptions` | 域名订阅 |

#### 人事管理 (3)
| 表名 | 说明 |
|---|---|
| `employees` | 员工 |
| `kpi_configs` | KPI 配置 |
| `kpi_records` | KPI 记录 |
| `kpi_metrics` | KPI 指标 |
| `social_insurance_configs` | 社保配置 |

#### 银行 (2)
| 表名 | 说明 |
|---|---|
| `bank_accounts` | 银行账户 |
| `bank_transactions` | 银行交易 |

#### 政策智能 (2)
| 表名 | 说明 |
|---|---|
| `policies` | 全球政策库 (60+ 条预置) |
| `policy_applications` | 政策申请 |

#### OPC 出海 (1)
| 表名 | 说明 |
|---|---|
| `global_opc_hubs` | OPC 圣地数据 (5+ 地区) |

#### 系统 (6)
| 表名 | 说明 |
|---|---|
| `operation_logs` | 操作审计日志 |
| `reminders` | 提醒 |
| `profiles` | 用户配置 |
| `files` | 文件元数据 |
| `ai_configurations` | AI 配置 |
| `email_configurations` | 邮件配置 |

#### 其他 (3)
| 表名 | 说明 |
|---|---|
| `vendors` | 友商管理 |
| `rd_costs` | 研发成本 |
| `data_asset_meta` | 数据资产元数据 |

### 核心表结构

#### contracts (合同 — 含 AI 审查)
```sql
CREATE TABLE IF NOT EXISTS contracts (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    user_id TEXT,
    contract_name TEXT NOT NULL,
    contract_number TEXT,
    contract_type TEXT,
    counterparty TEXT,
    counterparty_type TEXT,          -- vc/angel/government/incubator/employee/agency/client/landlord
    party_a TEXT,
    party_b TEXT,
    amount REAL,
    start_date TEXT,
    end_date TEXT,
    content TEXT,
    notes TEXT,
    status TEXT DEFAULT 'active',
    file_path TEXT,
    file_name TEXT,
    ai_review_status TEXT DEFAULT 'pending',  -- pending/reviewing/completed/failed/unavailable
    ai_risk_level TEXT,               -- 🔴 high / 🟡 medium / 🟢 low
    ai_review_result TEXT,            -- Markdown 审查报告
    ai_reviewed_at TEXT,              -- 审查时间
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP
);
```

#### companies (公司)
```sql
CREATE TABLE IF NOT EXISTS companies (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    legal_name TEXT,
    tax_id TEXT,
    country TEXT DEFAULT 'CN',
    -- ... 其他字段
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);
```

#### policies (政策)
```sql
CREATE TABLE IF NOT EXISTS policies (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    country TEXT DEFAULT '*',
    category TEXT,
    eligibility_criteria TEXT,       -- JSON
    benefits TEXT,
    max_amount REAL,
    deadline TEXT,
    source_url TEXT,
    is_global INTEGER DEFAULT 1
);
```

### 索引策略

```sql
-- 多租户查询加速
CREATE INDEX IF NOT EXISTS idx_invoices_company ON invoices(company_id);
CREATE INDEX IF NOT EXISTS idx_assets_company ON assets(company_id);
CREATE INDEX IF NOT EXISTS idx_employees_company ON employees(company_id);
CREATE INDEX IF NOT EXISTS idx_contracts_company ON contracts(company_id);
CREATE INDEX IF NOT EXISTS idx_operation_logs_company ON operation_logs(company_id);
-- ... 所有含 company_id 的表均有索引
```

### 设计原则

1. **TEXT 主键**: 使用 UUID v4，便于分布式扩展
2. **时间戳**: 使用 TEXT 存储 (ISO 8601 格式)
3. **多租户**: 所有业务表含 `company_id` 列 + 索引
4. **软删除**: 部分表支持 `is_deleted` / `deleted_at`
5. **审计**: operation_logs 记录所有写操作
6. **JSON**: 使用 TEXT 存储 JSON (如 eligibility_criteria)

---

*最后更新: 2026-03*

---

## Permission System

> 源自 `docs/01-system-architecture/04-permission-system.md`

### 架构概述

OPC 采用 **Session Token + 多租户隔离** 的安全架构。所有 API 请求通过 `require_auth` 中间件验证，自动注入 `company_id` 实现数据隔离。

### 认证流程

```
用户访问登录入口
    ↓
GET /auth/login
    ↓
跳转到 SSO/OAuth 提供方
    ↓
OAuth 回调成功
    ↓
生成 UUID session_id → 存入 sessions 表
    ↓
前端/CLI 携带 Authorization: Bearer {session_id}
```

### 中间件管道

```rust
// lib.rs
let api_routes = api::router(pool.clone())
    .layer(middleware::from_fn_with_state(pool.clone(), auth::middleware::require_auth));

Router::new()
    .route("/health", get(health_check))    // 无需认证
    .merge(auth_router)                      // 登录/注册 无需认证
    .nest("/api/v1", api_routes.clone())     // 需要认证
    .nest("/api", api_routes)                // 需要认证
```

### AuthUser 结构

```rust
pub struct AuthUser {
    pub user_id: String,      // 当前用户 ID
    pub company_id: String,   // 所属公司 ID (多租户隔离键)
}
```

由 `require_auth` 中间件自动注入到每个请求的 Extension 中。

### 多租户隔离

#### 写入隔离
```rust
// generic_create 自动注入 company_id
async fn generic_create(pool, table, body, required_fields, company_id, user_id) {
    // 自动将 company_id 作为列插入
    columns.push("company_id");
    values.push(company_id);
}
```

#### 读取隔离
```rust
// generic_list 自动过滤 company_id
async fn generic_list(pool, table, company_id, query) {
    // WHERE company_id = ?
    let sql = format!("SELECT * FROM {} WHERE company_id = ? ...", table);
}
```

#### 删除/更新隔离
```rust
// generic_update / generic_delete 同时匹配 id + company_id
// 跨租户访问返回 404 (不泄露数据存在性)
let sql = format!(
    "UPDATE {} SET ... WHERE id = ? AND company_id = ?", table
);
```

### 安全策略

#### SQL 注入防护
```rust
// 列名白名单验证
fn is_valid_column_name(name: &str) -> bool {
    name.chars().all(|c| c.is_alphanumeric() || c == '_')
        && !name.is_empty()
        && name.len() < 64
}
```

#### 财务安全
- 金额字段 (`amount`, `budget_amount`, `base_salary` 等) 强制 `≥ 0`
- 防止负数注入
- 外键引用校验 (防孤立记录)

#### CORS 白名单
```bash
# 生产环境: 精确指定允许的前端域名
ALLOWED_ORIGINS=https://your-domain.com

# 开发环境: 默认允许 localhost
ALLOWED_ORIGINS=http://localhost:5173,http://localhost:8080
```

#### 操作审计
- 所有 CRUD 操作自动记录到 `operation_logs` 表
- 记录: 操作类型 (create/update/delete)、表名、记录 ID、操作人、公司

### 角色管理

- `user_roles` 表支持用户角色配置
- 删除操作限定管理员 (通过前端权限控制实现)
- 未来路线: 基于 RBAC 的细粒度权限控制

---

*最后更新: 2026-03*

---

## System Blueprint

> 源自 `backend/SystemBlueprint.md`

> 版本: 1.0 | 日期: 2026-03-21  
> 本文件是前端 / CLI / 后端三端对齐的**唯一规范来源**。

### 模块命名规则

| 约定 | 说明 |
|---|---|
| 规范路径 | 通用实体: `/api/v1/<entity>`、专用模块: `/api/<module>` |
| 实体名 | 全小写，下划线分隔，复数形式 (e.g. `tax_deadlines`) |
| CLI 命令 | `opc list <entity>` / `opc <module> <action>` |

### A. 核心业务模块

#### A1. 财务管理

| 实体 | 规范路径 | 操作 | 前端页面 | CLI |
|---|---|---|---|---|
| invoices (发票) | `/api/invoices` | CRUD | ✅ Invoices.tsx | ✅ `list invoices` |
| payments (收支) | `/api/v1/payments` | CRUD | ✅ Payments.tsx | ✅ `list payments` |
| bank_accounts (银行账户) | `/api/bank/accounts` | CRUD | ✅ BankAccounts.tsx | ✅ `bank accounts` |
| bank_transactions (交易) | `/api/bank/transactions` | Create | ✅ BankAccounts.tsx | ✅ `bank transact` |
| budgets (预算) | `/api/budgets` | CRUD | ✅ BudgetManagement.tsx | ✅ `list budgets` |
| tax (税务) | `/api/v1/tax` | CRUD | ✅ TaxManagement.tsx | ✅ `list tax` |
| tax_declarations (申报) | `/api/v1/tax_declarations` | CRUD | ✅ TaxManagement.tsx | ✅ `list tax_declarations` |
| tax_jurisdictions (管辖区) | `/api/v1/tax_jurisdictions` | CRUD | ✅ TaxCalendar.tsx | ✅ `list tax_jurisdictions` |
| tax_deadlines (截止日) | `/api/v1/tax_deadlines` | CRUD | ✅ TaxCalendar.tsx | ✅ `list tax_deadlines` |
| exchange_rates (汇率) | `/api/v1/exchange_rates` | CRUD | ✅ Settings.tsx | ✅ `list exchange_rates` |
| categories (分类) | `/api/v1/categories` | CRUD | ✅ Settings.tsx | ✅ `list categories` |
| recurring_invoices (周期发票) | `/api/v1/recurring_invoices` | CRUD | ✅ RecurringInvoices.tsx | ✅ |
| invoice_reminders (发票催款) | `/api/v1/invoice_reminders` | CRUD | ✅ RecurringInvoices.tsx | ✅ |

#### A2. 人力资源

| 实体 | 规范路径 | 操作 | 前端页面 | CLI |
|---|---|---|---|---|
| employees (员工) | `/api/employees` | CRUD | ✅ Employees.tsx | ✅ `list employees` |
| salary / salary_records (薪资) | `/api/v1/salary` | CRUD | ✅ Salary.tsx | ✅ `list salary` |
| social_insurance_config (社保) | `/api/v1/social_insurance_config` | CRUD | ✅ Salary.tsx | ✅ |
| time_entries (工时) | `/api/v1/time_entries` | CRUD | ✅ TimeTracking.tsx | ✅ |
| insurance_policies (保险) | `/api/v1/insurance_policies` | CRUD | ✅ ComplianceManagement.tsx | ✅ |
| pension_plans (年金) | `/api/v1/pension_plans` | CRUD | ✅ ComplianceManagement.tsx | ✅ |

#### A3. 合同 & 项目

| 实体 | 规范路径 | 操作 | 前端页面 | CLI |
|---|---|---|---|---|
| contracts (合同) | `/api/v1/contracts` | CRUD | ✅ Contracts.tsx | ✅ `list contracts` |
| projects (项目) | `/api/v1/projects` | CRUD | ✅ Projects.tsx | ✅ `list projects` |
| milestones (里程碑) | `/api/v1/milestones` | CRUD | ✅ Projects.tsx | ✅ |
| reminders (提醒) | `/api/v1/reminders` | CRUD | ✅ Reminders.tsx | ✅ |

#### A4. CRM 客户管理

| 实体 | 规范路径 | 操作 | 前端页面 | CLI |
|---|---|---|---|---|
| clients (客户) | `/api/v1/clients` | CRUD | ✅ ClientManagement.tsx | ✅ |
| contacts (联系人) | `/api/v1/contacts` | CRUD | ✅ ClientManagement.tsx | ✅ |
| interactions (互动) | `/api/v1/interactions` | CRUD | ✅ ClientManagement.tsx | ✅ |
| deals (交易机会) | `/api/v1/deals` | CRUD | ✅ DealsManagement.tsx | ✅ |
| quotes (报价单) | `/api/v1/quotes` | CRUD | ✅ QuotesManagement.tsx | ✅ |
| quote_items (报价明细) | `/api/v1/quote_items` | CRUD | ✅ QuotesManagement.tsx | ✅ |

#### A5. 资产管理

| 实体 | 规范路径 | 操作 | 前端页面 | CLI |
|---|---|---|---|---|
| assets (资产) | `/api/assets` | CRUD | ✅ Assets.tsx | ✅ `list assets` |
| asset_records (资产记录) | `/api/v1/asset_records` | CRUD | ✅ Assets.tsx | ✅ |
| rent / rent_records (租赁) | `/api/v1/rent` | CRUD | ✅ Rent.tsx | ✅ |
| licenses (证照) | `/api/v1/licenses` | CRUD | ✅ Licenses.tsx | ✅ |
| vendors (供应商) | `/api/v1/vendors` | CRUD | ✅ Vendors.tsx | ✅ |

---

### B. 运营模块

#### B1. 云 & IT

| 实体 | 规范路径 | 操作 | 前端 | CLI |
|---|---|---|---|---|
| cloud_providers (云服务商) | `/api/v1/cloud_providers` | CRUD | ✅ | ✅ |
| cloud_resources (云资源) | `/api/v1/cloud_resources` | CRUD | ✅ | ✅ |
| cloud_bills (云账单) | `/api/v1/cloud_bills` | CRUD | ✅ | ✅ |
| domain_subscriptions (域名) | `/api/v1/domain_subscriptions` | CRUD | ✅ | ✅ |
| software_subscriptions (软件) | `/api/v1/software_subscriptions` | CRUD | ✅ | ✅ |

#### B2. 项目申报 & 材料

| 实体 | 规范路径 | 操作 | 前端 | CLI |
|---|---|---|---|---|
| applications (项目申报) | `/api/v1/applications` | CRUD | ✅ | ✅ |
| application_materials (申报材料) | `/api/v1/application_materials` | CRUD | ✅ | ✅ |
| material_library (材料库) | `/api/v1/material_library` | CRUD | ✅ | ✅ |
| material_templates (材料模板) | `/api/v1/material_templates` | CRUD | ✅ | ✅ |
| material_versions (材料版本) | `/api/v1/material_versions` | CRUD | ✅ | ✅ |

#### B3. KPI 绩效

| 实体 | 规范路径 | 操作 | 前端 | CLI |
|---|---|---|---|---|
| kpi (KPI 指标) | `/api/v1/kpi` | CRUD | ✅ | ✅ |
| kpi_configs (KPI 配置) | `/api/v1/kpi_configs` | CRUD | ✅ | ✅ |
| kpi_records (KPI 记录) | `/api/v1/kpi_records` | CRUD | ✅ | ✅ |

#### B4. 数据 & 研发

| 实体 | 规范路径 | 操作 | 前端 | CLI |
|---|---|---|---|---|
| data_asset_meta (数据资产) | `/api/v1/data_asset_meta` | CRUD | ✅ | ✅ |
| rd_costs (研发费用) | `/api/v1/rd_costs` | CRUD | ✅ | ✅ |

---

### C. 专用模块 (独立路由)

#### C1. 公司管理

| 端点 | 方法 | 前端 | CLI |
|---|---|---|---|
| `/api/companies` | GET / POST | ✅ | ✅ `company list/create` |
| `/api/companies/{id}` | GET / PUT / DELETE | ✅ | ✅ `company get/update/delete` |
| `/api/companies/{id}/members` | GET / POST | ✅ | ✅ `company members list/add` |
| `/api/companies/{id}/members/{mid}` | GET / PUT / DELETE | ✅ | ✅ `company members remove/update-role` |
| `/api/companies/{id}/switch` | GET | ✅ | ✅ `company switch` |

#### C2. AI 智能

| 端点 | 方法 | 前端 | CLI |
|---|---|---|---|
| `/api/ai/chat` | POST | ✅ | ✅ `ai chat` |
| `/api/ai/analyze` | POST | ✅ | ✅ `ai analyze` |
| `/api/ai/audit` | POST | ✅ | ✅ `ai audit` |
| `/api/ai/parse-document` | POST | ✅ | ✅ `ai parse-doc` |
| `/api/ai/contract-review` | POST | ✅ | ✅ `ai contract-review` |

#### C3. 文件管理

| 端点 | 方法 | 前端 | CLI |
|---|---|---|---|
| `/api/files/upload` | POST | ✅ | ⬜ (需 multipart) |
| `/api/files/upload/{entity_type}/{entity_id}` | POST | ✅ | ⬜ |
| `/api/files/{id}` | GET / DELETE | ✅ | ✅ `files download/delete` |
| `/api/files/list/{entity_type}/{entity_id}` | GET | ✅ | ✅ `files list` |

#### C4. API Key 管理

| 端点 | 方法 | 前端 | CLI |
|---|---|---|---|
| `/api/api-keys` | GET / POST | ✅ | ⬜ 需添加 |
| `/api/api-keys/{id}/revoke` | POST | ✅ | ⬜ 需添加 |
| `/api/api-keys/{id}` | DELETE | ✅ | ⬜ 需添加 |

#### C5. 全球 OPC

| 端点 | 方法 | 前端 | CLI |
|---|---|---|---|
| `/api/global_opc_hubs` | GET | ✅ | ✅ `opc-hubs` |
| `/api/global_opc_hubs/calculator` | POST | ✅ | ⬜ 需添加 |

#### C6. 政策

| 端点 | 方法 | 前端 | CLI |
|---|---|---|---|
| `/api/policies` | GET | ✅ | ✅ `policies list` |
| `/api/policies/match` | GET | ✅ | ✅ `policies match` |
| `/api/policies/{id}` | GET | ✅ | ✅ `policies get` |
| `/api/policies/applications` | CRUD | ✅ | ✅ `policies applications` |

---

### D. 特殊端点

#### D1. 报表

| 端点 | 前端 | CLI |
|---|---|---|
| `/api/v1/reports/financial-health` | ✅ FinancialReports.tsx | ✅ `report financial-health` |
| `/api/v1/reports/cashflow-forecast` | ✅ CashFlowForecast.tsx | ✅ `report cashflow` |
| `/api/v1/reports/profit-analysis` | ✅ FinancialReports.tsx | ✅ `report profit` |
| `/api/v1/reports/deals-conversion` | ✅ DealsManagement.tsx | ✅ `report deals` |

#### D2. PDF 生成

| 端点 | 前端 | CLI |
|---|---|---|
| `/api/v1/invoices/{id}/pdf` | ✅ | ✅ `pdf invoice` |
| `/api/v1/quotes/{id}/pdf` | ✅ | ✅ `pdf quote` |

#### D3. 系统

| 端点 | 前端 | CLI |
|---|---|---|
| `/api/v1/stats/dashboard` | ✅ Dashboard.tsx | ✅ `dashboard` |
| `/api/v1/system/status` | ✅ Settings.tsx | ✅ `status` |
| `/api/v1/operation_logs` | ✅ OperationLogs.tsx | ✅ `logs` |
| `/api/v1/error_logs` | ✅ AuditCenter.tsx | ✅ `list error_logs` |
| `/api/v1/api-catalog` | — | ⬜ 需添加 |
| `/api/v1/security/audit` | — | ⬜ 需添加 |

#### D4. 税务工具

| 端点 | 前端 | CLI |
|---|---|---|
| `/api/v1/tax/calculate` | ✅ | ⬜ 需添加 |
| `/api/v1/tax/compliance-check` | ✅ | ⬜ 需添加 |
| `/api/v1/tax/invoice-requirements` | ✅ | ⬜ 需添加 |
| `/api/v1/tax/input-credit` | ✅ | ⬜ 需添加 |

#### D5. 其它工具

| 端点 | 前端 | CLI |
|---|---|---|
| `/api/v1/currency/convert` | ✅ | ⬜ 需添加 |
| `/api/v1/currency/balances` | ✅ | ⬜ 需添加 |
| `/api/v1/bank/reconcile` | ✅ ReconciliationCenter.tsx | ⬜ 需添加 |
| `/api/v1/ar/aging` | ✅ | ⬜ 需添加 |
| `/api/v1/ar/summary` | ✅ | ⬜ 需添加 |
| `/api/v1/webhooks` | ✅ | ⬜ 需添加 |
| `/api/v1/email/send` | ✅ | ⬜ 需添加 |
| `/api/v1/time_entries/invoice` | ✅ | ⬜ 需添加 |
| `/api/v1/quotes/{id}/convert-to-invoice` | ✅ | ⬜ 需添加 |
| `/api/v1/audit/export` | ✅ | ⬜ 需添加 |
| `/api/v1/export/{table}` | ✅ | ✅ `export` |

---

### E. 系统配置

| 实体 | 规范路径 | 前端 | CLI |
|---|---|---|---|
| company_info (企业信息) | `/api/v1/company_info` | ✅ CompanyInfo.tsx | ✅ |
| branches (分支机构) | `/api/v1/branches` | ✅ CompanyInfo.tsx | ✅ |
| profiles (用户档案) | `/api/v1/profiles` | ✅ Profile.tsx | ✅ |
| user_roles (用户角色) | `/api/v1/user_roles` | ✅ UserManagement.tsx | ✅ |
| ai_configurations (AI配置) | `/api/v1/ai_configurations` | ✅ Settings.tsx | ✅ |
| email_configurations (邮件配置) | `/api/v1/email_configurations` | ✅ Settings.tsx | ✅ |
| utilities (水电费) | `/api/v1/utilities` | ✅ Settings.tsx | ✅ |

---

### F. CLI 需补齐的命令 (14 项)

| 序号 | 功能 | CLI 命令 |
|---|---|---|
| 1 | API Key 管理 | `opc api-keys list/create/revoke/delete` |
| 2 | OPC 税务计算 | `opc opc-hubs calculate` |
| 3 | 税务计算 | `opc tax calculate` |
| 4 | 税务合规检查 | `opc tax compliance-check` |
| 5 | 税务发票要求 | `opc tax invoice-requirements` |
| 6 | 进项抵扣 | `opc tax input-credit` |
| 7 | 汇率转换 | `opc currency convert` |
| 8 | 多币种余额 | `opc currency balances` |
| 9 | 银行对账 | `opc bank reconcile` |
| 10 | 应收账龄 | `opc ar aging` |
| 11 | 应收汇总 | `opc ar summary` |
| 12 | Webhook 管理 | `opc webhooks list/create/delete/test` |
| 13 | 时间→发票转换 | `opc time-to-invoice` |
| 14 | 报价→发票转换 | `opc quote-to-invoice` |

---

### G. 路径别名映射

后端为兼容性注册了多个别名。以下为规范路径和别名：

| 规范路径 | 别名 | 说明 |
|---|---|---|
| `/api/v1/salary` | `/api/v1/salaries`, `/api/v1/salary_records` | 薪资 |
| `/api/v1/rent` | `/api/v1/rents`, `/api/v1/rent_records` | 租赁 |
| `/api/bank/transactions` | `/api/v1/bank/transactions`, `/api/v1/bank_transactions` | 银行交易 |

---

## API Reference

> 源自 `docs/api-reference.md`

> 当前以 `opc-server` 的 HTTP API 为唯一业务真源。本文只记录当前主线仍在使用的真实端点。

### 认证

除 `/health` 和 `/auth/*` 外，`/api/` 与 `/api/v1/` 路由都需要认证：

```http
Authorization: Bearer <session_id>
```

也支持 API Key：

```http
X-API-Key: <api_key>
```

#### 认证端点

| 方法 | 端点 | 说明 |
|---|---|---|
| GET | `/auth/login` | 跳转到 SSO/OAuth 登录页 |
| GET | `/auth/callback` | OAuth 回调 |
| GET | `/auth/logout` | 销毁当前会话 |
| GET | `/auth/me` | 当前登录用户信息 |

### 路由约定

- 服务端把同一套路由同时挂载到 `/api` 与 `/api/v1`
- CLI 当前推荐路径大致分三类：
  - 专用/typed CRUD：`/api/invoices`、`/api/assets`、`/api/employees`、`/api/budgets`
  - 业务模块：`/api/companies`、`/api/bank/*`、`/api/ai/*`、`/api/files/*`、`/api/api-keys/*`、`/api/policies/*`、`/api/global_opc_hubs*`
  - 通用 CRUD 与工具接口：`/api/v1/<entity>`、`/api/v1/reports/*`、`/api/v1/tax/*`、`/api/v1/currency/*`、`/api/v1/ar/*`、`/api/v1/webhooks*`

### 响应约定

- 通用列表接口通常返回 `{ "data": [...], "count": N }`
- `api-keys` 列表单独返回 `{ "keys": [...], "total": N }`
- 创建/更新接口通常返回对象本身，至少包含 `id`
- 删除接口既有 `204 No Content`，也有 `{ "success": true, "message": ... }` 这种 JSON；不要假设所有删除端点都同形
- PDF 下载接口返回二进制文件，不是 JSON
- `/api/v1/export/{table}` 返回 CSV 文本；失败时才会返回 JSON 错误体

### 通用 CRUD

所有通用实体遵循这一模式：

| 方法 | 端点 | 说明 |
|---|---|---|
| GET | `/api/v1/<entity>` | 列表 |
| GET | `/api/v1/<entity>/{id}` | 详情 |
| POST | `/api/v1/<entity>` | 创建 |
| PUT | `/api/v1/<entity>/{id}` | 更新 |
| DELETE | `/api/v1/<entity>/{id}` | 删除 |

常见实体包括：

- 财务：`payments` `tax` `tax_declarations` `categories` `exchange_rates` `recurring_invoices` `invoice_reminders`
- 合同/项目：`contracts` `projects` `milestones` `reminders`
- CRM：`clients` `contacts` `interactions` `deals` `quotes` `quote_items`
- 资产扩展：`asset_records` `rent` `licenses` `vendors`
- 云与 IT：`cloud_providers` `cloud_resources` `cloud_bills` `domain_subscriptions` `software_subscriptions`
- 申报：`applications` `application_materials` `material_library` `material_templates` `material_versions`
- 配置：`company_info` `branches` `profiles` `user_roles` `ai_configurations` `email_configurations` `utilities`
- 系统：`error_logs` `tax_jurisdictions` `tax_deadlines`

### 专用模块

#### 资产

| 方法 | 端点 |
|---|---|
| GET | `/api/assets` |
| POST | `/api/assets` |
| GET | `/api/assets/{id}` |
| PUT | `/api/assets/{id}` |
| DELETE | `/api/assets/{id}` |

#### 发票

| 方法 | 端点 |
|---|---|
| GET | `/api/invoices` |
| POST | `/api/invoices` |
| GET | `/api/invoices/{id}` |
| PUT | `/api/invoices/{id}` |
| DELETE | `/api/invoices/{id}` |

#### 员工

| 方法 | 端点 |
|---|---|
| GET | `/api/employees` |
| POST | `/api/employees` |
| GET | `/api/employees/{id}` |
| PUT | `/api/employees/{id}` |
| DELETE | `/api/employees/{id}` |

#### 预算

| 方法 | 端点 |
|---|---|
| GET | `/api/budgets` |
| POST | `/api/budgets` |
| GET | `/api/budgets/{id}` |
| PUT | `/api/budgets/{id}` |
| DELETE | `/api/budgets/{id}` |

#### 银行

| 方法 | 端点 | 说明 |
|---|---|---|
| GET | `/api/bank/accounts` | 银行账户列表 |
| POST | `/api/bank/accounts` | 创建银行账户 |
| GET | `/api/bank/accounts/{id}` | 账户详情 |
| PUT | `/api/bank/accounts/{id}` | 更新账户 |
| DELETE | `/api/bank/accounts/{id}` | 删除账户 |
| POST | `/api/bank/transactions` | 创建交易 |

#### 公司

| 方法 | 端点 | 说明 |
|---|---|---|
| GET | `/api/companies` | 当前用户可见公司列表 |
| POST | `/api/companies` | 创建公司 |
| GET | `/api/companies/{id}` | 公司详情 |
| PUT | `/api/companies/{id}` | 更新公司 |
| DELETE | `/api/companies/{id}` | 删除公司 |
| GET | `/api/companies/{id}/switch` | 切换当前活跃公司 |
| GET | `/api/companies/{id}/members` | 成员列表 |
| POST | `/api/companies/{id}/members` | 添加成员 |
| GET | `/api/companies/{id}/members/{member_id}` | 成员详情 |
| PUT | `/api/companies/{id}/members/{member_id}` | 更新成员角色 |
| DELETE | `/api/companies/{id}/members/{member_id}` | 移除成员 |

添加成员请求体：

```json
{
  "email": "user@example.com",
  "role": "member"
}
```

#### 政策

| 方法 | 端点 |
|---|---|
| GET | `/api/policies` |
| GET | `/api/policies/match` |
| GET | `/api/policies/{id}` |
| GET | `/api/policies/applications` |
| POST | `/api/policies/applications` |
| GET | `/api/policies/applications/{id}` |
| PUT | `/api/policies/applications/{id}` |
| DELETE | `/api/policies/applications/{id}` |

#### 全球 OPC 出海沙盘

| 方法 | 端点 | 说明 |
|---|---|---|
| GET | `/api/global_opc_hubs` | 注册地列表 |
| POST | `/api/global_opc_hubs/calculator` | 税务推演计算器 |

请求体示例：

```json
{
  "est_annual_revenue": 500000,
  "current_base_col_index": 50
}
```

#### API Key

| 方法 | 端点 | 说明 |
|---|---|---|
| GET | `/api/api-keys` | 列出当前用户在当前公司下的 API Key |
| POST | `/api/api-keys` | 创建 API Key |
| POST | `/api/api-keys/{id}/revoke` | 吊销 API Key |
| DELETE | `/api/api-keys/{id}` | 永久删除 API Key |

创建请求体：

```json
{
  "name": "CI Key",
  "scopes": "*",
  "expires_in_days": 30
}
```

创建响应示例：

```json
{
  "id": "uuid",
  "name": "CI Key",
  "key": "om_xxx",
  "key_prefix": "om_12345678",
  "scopes": "*",
  "created_at": "2026-04-16 12:00:00",
  "expires_at": "2026-05-16 12:00:00"
}
```

注意：`key` 只会在创建时返回一次。

### AI 接口

#### 聊天

```http
POST /api/ai/chat
```

```json
{
  "message": "分析一下本月收支情况",
  "context": "可选上下文",
  "history": []
}
```

#### 数据分析

```http
POST /api/ai/analyze
```

```json
{
  "data_type": "invoices",
  "data": {"revenue": 100000, "costs": 60000},
  "question": "请分析这些数据并给出建议"
}
```

#### 合规审计

```http
POST /api/ai/audit
```

```json
{
  "audit_type": "contract",
  "contract_id": "uuid"
}
```

`audit_type` 支持：`application` `material` `contract` `employee` `budget` `project`

#### 文档解析（OCR）

```http
POST /api/ai/parse-document
```

```json
{
  "file_base64": "base64内容",
  "document_type": "contract",
  "model": "paddleocr-vl:1.5"
}
```

`document_type` 常见值：`contract` `invoice` `material` `budget` `asset` `utility` `business_license` `vendor` `license` `employee`

#### 合同审查

```http
POST /api/ai/contract-review
```

```json
{
  "contract_text": "合同全文",
  "contract_type": "服务合同",
  "counterparty_type": "client"
}
```

响应示例：

```json
{
  "review": "Markdown 审查报告",
  "contract_type": "服务合同",
  "counterparty_type": "client",
  "success": true
}
```

### 文件管理 API

| 方法 | 端点 | 说明 |
|---|---|---|
| POST | `/api/files/upload` | 上传未关联文件 |
| POST | `/api/files/upload/{entity_type}/{entity_id}` | 上传并直接关联到业务实体 |
| GET | `/api/files/list/{entity_type}/{entity_id}` | 列出实体附件 |
| GET | `/api/files/{id}` | 下载文件 |
| DELETE | `/api/files/{id}` | 删除文件 |
| GET | `/api/files/static/{company_id}/{filename}` | 静态预览路径 |

注意：

- 应用当前全局 body limit 为 `50MB`
- 文件记录按 `company_id` 隔离
- 下载/删除使用文件 `id`，不是原始文件名

### 系统与报表

| 方法 | 端点 |
|---|---|
| GET | `/health` |
| GET | `/api/v1/system/status` |
| GET | `/api/v1/stats/dashboard` |
| GET | `/api/v1/operation_logs` |
| GET | `/api/v1/reports/financial-health` |
| GET | `/api/v1/reports/cashflow-forecast` |
| GET | `/api/v1/reports/profit-analysis` |
| GET | `/api/v1/reports/deals-conversion` |
| GET | `/api/v1/export/{table}` |

### 其它工具型端点

| 方法 | 端点 |
|---|---|
| GET | `/api/v1/tax/calculate` |
| GET | `/api/v1/tax/compliance-check` |
| GET | `/api/v1/tax/invoice-requirements` |
| GET | `/api/v1/tax/input-credit` |
| GET | `/api/v1/currency/convert` |
| GET | `/api/v1/currency/balances` |
| GET | `/api/v1/ar/aging` |
| GET | `/api/v1/ar/summary` |
| GET / POST | `/api/v1/webhooks` |
| DELETE | `/api/v1/webhooks/{id}` |
| POST | `/api/v1/webhooks/{id}/test` |
| POST | `/api/v1/time_entries/invoice` |
| POST | `/api/v1/quotes/{id}/convert-to-invoice` |
| GET | `/api/v1/invoices/{id}/pdf` |
| GET | `/api/v1/quotes/{id}/pdf` |

Webhook 创建请求体当前真实契约是字符串 `events`：

```json
{
  "url": "https://hooks.example.com/pay",
  "events": "invoice.created,payment.received",
  "description": "optional",
  "secret": "optional"
}
```

Webhook 测试响应会返回 JSON，常见字段包括：

```json
{
  "status": "success",
  "http_status": 200,
  "url": "https://hooks.example.com/pay",
  "payload_sent": {}
}
```

### 错误码

| 状态码 | 含义 |
|---|---|
| 200 | 成功 |
| 400 | 参数错误 |
| 401 | 未认证 |
| 403 | 权限不足 |
| 404 | 资源不存在或跨租户不可见 |
| 500 | 服务端错误 |
| 503 | AI 服务不可用 |

---

*最后更新: 2026-04-16*

---

## CLI Guide

> 源自 `backend/CliGuide.md`

> `opc` 是一个全功能的 ERP 命令行工具，支持管理发票、合同、员工、资产、银行、税务、CRM 等 40+ 业务模块。

### 安装 & 配置

```bash
# 编译
cd backend && cargo build --release --bin opc

# 配置环境变量（可选，避免每次传参）
export OPC_SERVER=http://localhost:8987
export OPC_SESSION=<your-session-id>
```

### 认证

```bash
# SSO / OAuth 登录（打开浏览器）
opc auth login

# 查看当前会话
opc auth status

# 手动指定 session
opc --session <session_id> status
```

---

### 核心 CRUD

> 所有 40+ 实体均支持 `list / get / create / update / delete / export`

#### 创建记录

```bash
# 方式1: -f key=value（推荐，简洁）
opc create invoices -f invoice_number=INV-001 -f amount=5000 -f client_name=张三

# 方式2: --json（复杂数据）
opc create contracts --json '{"contract_name":"服务合同","amount":50000,"counterparty":"XX公司"}'

# 安静模式 —— 只输出 ID（方便脚本串联）
ID=$(opc -q create invoices -f amount=100)
```

约束说明：

- `-f/--field` 必须是非空 `key=value`；像 `-f '=value'` 或漏掉 `=` 都会在发请求前直接报错
- `--json` 必须是合法 JSON；报错会带上具体 flag 名，例如 `invalid JSON for --json: ...`

#### 查询

```bash
# 列表（默认20条）
opc list invoices

# 分页
opc list contracts --limit 50 --page 2

# 查看单条
opc get invoices <uuid>
```

#### 更新 & 删除

```bash
opc update invoices <uuid> -f status=paid
opc delete invoices <uuid>
```

#### 导出 CSV

```bash
opc export contracts > contracts.csv
```

---

### 支持的实体列表

| 模块 | 实体名称 |
|---|---|
| **财务** | `invoices` `payments` `tax` `tax_declarations` `categories` `exchange_rates` `recurring_invoices` `invoice_reminders` |
| **HR** | `employees` `salary` `social_insurance_config` `time_entries` `insurance_policies` `pension_plans` |
| **合同/项目** | `contracts` `projects` `milestones` `reminders` |
| **CRM** | `clients` `contacts` `interactions` `deals` `quotes` `quote_items` |
| **资产** | `assets` `asset_records` `rent` `licenses` `vendors` |
| **云/IT** | `cloud_providers` `cloud_resources` `cloud_bills` `domain_subscriptions` `software_subscriptions` |
| **申报** | `applications` `application_materials` `material_library` `material_templates` `material_versions` |
| **KPI** | `kpi` `kpi_configs` `kpi_records` |
| **研发** | `data_asset_meta` `rd_costs` |
| **配置** | `company_info` `branches` `profiles` `user_roles` `ai_configurations` `email_configurations` `utilities` |
| **系统** | `error_logs` `tax_jurisdictions` `tax_deadlines` |

---

### 专用模块

#### 公司管理

```bash
opc company list                         # 列出所有公司
opc company create -f name="新公司"
opc company switch <company_id>          # 切换活跃公司
opc company members list <company_id>    # 查看成员
opc company members add <company_id> --email user@test-com.example.invalid --role member
```

#### 银行

```bash
opc bank accounts                        # 列出银行账户
opc bank create-account -f bank_name=中国银行 -f account_name=主账户 -f account_number=123456 -f balance=100000
opc bank transact --account <id> --type deposit --amount 5000 --date 2026-03-21
```

#### AI 智能分析

```bash
opc ai chat --message "分析一下本月收支情况"
opc ai analyze --data-type invoices --data '{"revenue":100000,"costs":60000}'
opc ai audit --audit-type contract --contract-id <contract_id>
opc ai parse-doc --file report.pdf --document-type contract
opc ai contract-review --file contract.txt --contract-type service --counterparty-type client
```

#### 报表

```bash
opc report financial-health    # 财务健康
opc report cashflow            # 现金流预测
opc report profit              # 利润分析
opc report deals               # 交易转化漏斗
```

#### PDF 生成

```bash
opc pdf invoice <invoice_id>              # 生成发票 PDF
opc pdf invoice <id> --output my-inv.pdf  # 指定文件名
opc pdf quote <quote_id>                  # 生成报价单 PDF
```

#### 税务工具

```bash
opc tax calculate --amount 100000        # 税额计算
opc tax compliance-check                 # 合规检查
opc tax invoice-requirements             # 发票要求
opc tax input-credit                     # 进项抵扣汇总
```

#### 多币种

```bash
opc currency convert --from USD --to CNY --amount 1000
opc currency balances                    # 多币种余额
```

#### 应收账款分析

```bash
opc ar aging                             # 账龄分析（0-30/31-60/61-90/91-180/180+天）
opc ar summary                           # 应收汇总
```

#### 政策

```bash
opc policies list                        # 全部政策
opc policies match                       # 智能匹配适合你公司的政策
opc policies get <id>                    # 查看政策详情
opc policies applications list           # 已申请的政策
```

#### OPC 全球注册地

```bash
opc locations                             # 列出全球OPC友好注册地
opc locations calculate --revenue 500000  # 各注册地税务对比
```

#### 文件管理

```bash
opc files upload report.pdf                                     # 上传文件
opc files upload invoice.pdf --entity-type invoices --entity-id <id>  # 关联到实体
opc files list --entity-type invoices --entity-id <id>          # 列出附件
opc files download <file_id>                                    # 下载文件
opc files download <file_id> -o output.pdf                      # 指定保存名
opc files delete <file_id>                                      # 删除文件
```

#### API Key

```bash
opc api-keys list                        # 列出 API Key
opc api-keys create --name "CI Key"      # 创建
opc api-keys revoke <id>                 # 吊销
opc api-keys delete <id>                 # 删除
```

#### Webhook

```bash
opc webhooks list                        # 列出
opc webhooks create --url https://hooks.example.com/pay --events invoice.created,payment.received
opc webhooks test <id>                   # 发送测试事件
opc webhooks delete <id>                 # 删除
```

#### 转换

```bash
# 工时 → 发票
opc time-to-invoice --entries <id1>,<id2>,<id3> --rate 150

# 报价单 → 发票
opc quote-to-invoice <quote_id>
```

---

### 全局参数

| 参数 | 环境变量 | 说明 |
|---|---|---|
| `--server <url>` | `OPC_SERVER` | 后端地址，默认 `http://localhost:8987` |
| `--session <id>` | `OPC_SESSION` | 登录会话 ID |
| `--api-key <key>` | `OPC_API_KEY` | API Key（替代 session） |
| `--format <fmt>` | — | 输出格式: `table`(默认) / `json` / `csv` |
| `-q, --quiet` | — | 安静模式；变更命令输出单个稳定值（通常是 ID，下载命令为路径） |

---

### 输出契约

默认输出是面向人的表格/提示信息；脚本场景建议固定使用下面两种模式：

| 模式 | stdout 契约 |
|---|---|
| `--format json` | 只输出单个 JSON 文档；不会再混入 `✅` 成功提示 |
| `--quiet` | 对创建/更新/删除/吊销/测试这类变更命令，只输出一个稳定值：通常是目标资源 `id`；下载类命令输出最终文件路径 |

补充说明：

- `list` 类命令在默认模式下会把 `Total: N` 打到 `stderr`；`--quiet` 会抑制这条计数，但不会把列表改写成别的结构
- API 返回 `{ "data": [...], "count": n }` 的列表接口时，CLI 在 `--format json` 下会直接输出 `data` 数组，方便管道消费
- `opc pdf invoice ...` / `opc files download ...` 在 `--format json` 下输出 `{ "id": "...", "output": "..." }`

---

### 脚本使用示例

```bash
#!/bin/bash
# 批量导入合同
export OPC_SESSION=xxx

while IFS=, read -r name counterparty amount; do
  opc -q create contracts \
    -f "contract_name=$name" \
    -f "counterparty=$counterparty" \
    -f "amount=$amount"
done < contracts.csv

# 批量导出所有模块
for entity in invoices contracts employees assets payments; do
  opc export $entity > "${entity}.csv"
  echo "✅ Exported $entity"
done

# 链式操作：创建发票 → 生成 PDF
INV_ID=$(opc -q create invoices -f invoice_number=AUTO-001 -f amount=15000)
opc --format json pdf invoice $INV_ID --output "invoice-${INV_ID}.pdf"
```

---

### 系统诊断

```bash
opc status                   # 系统健康检查
opc dashboard                # 仪表盘概览
opc logs --limit 50          # 最近操作日志
```

---

## AI Contract Review

> 源自 `docs/ai-contract-review.md`

### 概述

OPC 内置强制 AI 合同审查机制。当用户创建或更新合同时，系统会自动在后台触发 AI 审查，无需用户主动操作。审查结果以风险标签的形式直接展示在合同列表中。

### 工作流程

```
用户创建/更新合同
       ↓
   generic_create / generic_update
       ↓
   返回 200 OK（不阻塞用户）
       ↓
   tokio::spawn 后台任务
       ↓
   ┌──────────────────────┐
   │ 检查 AI_API_URL      │
   │ ├─ 未配置 → unavailable │
   │ └─ 已配置 → 调用 AI   │
   └──────────────────────┘
       ↓
   解析风险等级
   🔴 高风险 / 🟡 中风险 / 🟢 低风险
       ↓
   UPDATE contracts SET ai_review_result, ai_risk_level
       ↓
   前端刷新自动看到风险标签
```

### 数据库字段

| 字段 | 类型 | 说明 |
|---|---|---|
| `ai_review_status` | TEXT | `pending` → `reviewing` → `completed` / `failed` / `unavailable` |
| `ai_risk_level` | TEXT | `🔴 high` / `🟡 medium` / `🟢 low` |
| `ai_review_result` | TEXT | 完整 Markdown 审查报告 |
| `ai_reviewed_at` | TEXT | 审查时间戳 |
| `counterparty_type` | TEXT | 合同对方身份 |

### 对手方类型

| 类型 | AI 重点检测 |
|---|---|
| `vc` | 对赌条款、回购权、一票否决、清算优先权 |
| `angel` | 股权稀释、反稀释条款、退出条款 |
| `government` | 政策合规、知识产权归属、验收标准 |
| `incubator` | 股权绑定、毕业条件、排他条款 |
| `employee` | 竞业协议合理性、期权行权条件 |
| `agency` | 中介费比例、独家条款、违约金 |
| `client` | 付款账期、IP归属、验收标准 |
| `landlord` | 租金递增、押金退还、提前解约 |

### 降级策略

1. **未配置 AI** (`AI_API_URL` 不存在)
   - 合同正常保存
   - `ai_review_status` = `unavailable`
   - 不影响任何用户操作

2. **AI 调用失败** (网络错误、超限等)
   - `ai_review_status` = `failed`
   - 下次服务器启动时自动重试

3. **自动补全** (Auto-catchup)
   - 服务器启动时扫描 `pending` / `unavailable` / `failed` 合同
   - 自动批量重新审查 (500ms 间隔防限流)

### 前端展示

#### 统计卡片
- **AI 防坑卫士** — 显示高风险合同数量或 `✓ 安全`
- 高风险时卡片边框变红

#### 合同列表
- **AI 风险** 列：
  - 🔴 高风险 Badge (红色)
  - 🟡 中风险 Badge (黄色)
  - 🟢 低风险 Badge (绿色)
  - ⏳ 审查中 (Spinner 动画)
  - 灰色 "待审查" / 红色 "审查失败"

### OCR 模型选择

| 模型 | 参数量 | 适用场景 | 部署方式 |
|---|---|---|---|
| **paddleocr-vl:1.5** (当前默认) | 0.9B | 证照/发票/表格 | `DOCUMENT_PARSER_BACKEND=ollama-vision` + `ollama pull paddleocr-vl:1.5` |
| qwen3-vl:30b | 30B | 复杂文档/多语言 | GPU 推荐 |
| liteparse | 本地解析库 | PDF 版面文本、bbox、截图、OCR 服务桥接 | 预留接口，待接入 |
| paddleocr | OCR 服务 | PaddleOCR 1.6 服务化 OCR | 预留接口，待接入 |

请求时可通过 `model` 字段指定：
```json
{
  "file_base64": "...",
  "document_type": "invoice",
  "model": "qwen3-vl:30b"
}
```

---

*最后更新: 2026-03*

---

## File Management

> 源自 `docs/file-management.md`

### 概述

OPC 的文件管理能力通过后端本地文件系统和 `files` 表共同实现，支持合同、发票、证照等附件上传、下载、删除和按业务实体关联。

### 当前实现

#### 存储方式

- 物理目录：`backend/data/uploads/<company_id>/`
- 文件名：内容哈希 `filename`
- 元数据：保存在 `files` 表
- 隔离方式：按 `company_id` 目录和数据库字段双重隔离
- 单文件限制：`10MB`

#### API 端点

| 方法 | 端点 | 说明 |
|---|---|---|
| POST | `/api/files/upload` | 上传未关联文件 |
| POST | `/api/files/upload/{entity_type}/{entity_id}` | 上传并直接关联实体 |
| GET | `/api/files/list/{entity_type}/{entity_id}` | 查看实体附件列表 |
| GET | `/api/files/{id}` | 下载文件 |
| DELETE | `/api/files/{id}` | 删除文件 |
| GET | `/api/files/static/{company_id}/{filename}` | 静态预览 |

#### 上传流程

```text
用户选择文件
    ↓
POST /api/files/upload 或 /api/files/upload/{entity_type}/{entity_id}
    ↓
后端校验大小与权限
    ↓
写入 data/uploads/<company_id>/ + 插入 files 表
    ↓
返回文件 id 与元数据
```

### OCR 集成

文件上传后可进入 OCR / AI 解析流程：

```text
文件读取
    ↓
前端转 base64
    ↓
POST /api/ai/parse-document
    ↓
DOCUMENT_PARSER_BACKEND
    ├─ ollama-vision: Ollama + PaddleOCR-VL
    ├─ liteparse: 预留
    └─ paddleocr: 预留
    ↓
返回结构化字段
```

常见 `document_type`：

| document_type | 用途 |
|---|---|
| `contract` | 合同识别 |
| `invoice` | 发票识别 |
| `material` | 材料识别 |
| `budget` | 预算/收据识别 |
| `asset` | 资产识别 |
| `utility` | 水电燃气账单识别 |
| `business_license` / `vendor` | 企业资质识别 |
| `license` | 证照识别 |
| `employee` | 人事文档识别 |

### 前端集成点

- 上传入口：`FileUploader` / `AttachmentManager`
- 自动识别：`AutoParseButton`
- 预览：`FilePreviewDialog`

### `files` 表结构

```sql
CREATE TABLE IF NOT EXISTS files (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    filename TEXT NOT NULL,
    original_name TEXT NOT NULL,
    content_type TEXT DEFAULT 'application/octet-stream',
    size INTEGER DEFAULT 0,
    entity_type TEXT,
    entity_id TEXT,
    uploaded_by TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);
```

### 注意事项

- 下载和删除使用文件 `id`，不是原始文件名
- 同内容文件会走内容哈希，减少重复写盘
- 静态预览路径只适合前端展示，不替代受控下载

---

*最后更新: 2026-04-16*

---

## Self-hosting configuration

See the repository README, `.env.example` templates and `source/opc/docker-compose.yml`.
Provide your own OAuth client, tenant data directories and provider credentials.
The application binds to loopback by default; put TLS at your own managed ingress.
Telemetry and binary replacement require explicit configuration and are disabled by default.

## Changelog

> 源自 `docs/CHANGELOG.md`

### 2026-03 — AI 防坑卫士 & OPC 出海沙盘

#### 强制 AI 合同审查
- 合同创建/更新自动触发 AI 后台审查
- 风险等级: 🔴 高 / 🟡 中 / 🟢 低
- 对手方识别: VC、天使、政府、孵化器、员工、中介、客户、房东
- 降级策略: AI 不可用时标记 `unavailable`，恢复后自动补全
- 前端: "AI 防坑卫士" 统计卡 + 风险 Badge 列

#### OPC 出海沙盘
- 5+ 全球 OPC 圣地数据 (Dubai, Singapore, Wyoming, HK, Estonia)
- 穿透税制 vs 双重征税对比
- MRR 温饱线推演
- 最优选址推荐

#### PaddleOCR-VL 1.5
- 默认 OCR 引擎切换为 PaddleOCR-VL 1.5 (0.9B)
- 30-50x 推理加速，~2GB VRAM
- 前端可选模型: 请求级别指定

#### 政策智能增强
- 新增大学生创业补贴、社保补贴、免息贷款等基层政策
- 政策匹配算法: 国家 +40, 学历 +15, 海归 +15

#### Bug 修复
- `companies` 表补全 `country` 列 (默认 CN)
- `operation_logs` 路由注册
- `software_subscriptions` 必填 `next_billing_date` 校验
- 合同字段前后端对齐: `title` → `contract_name`

#### 测试
- 新增 `s16_mandatory_contract_review.rs` (6 tests)
- 修复 `s09_policy_intelligence.rs` (4 tests — 政策播种)
- 修复 `s11_cloud_cost_lifecycle.rs` (tenant isolation)
- 修复 `s13_audit_and_reconciliation.rs` (operation_logs 路由)
- **全量: s02-s16 共 69 tests, 0 failures**

---

*生成时间: 2026-03-01*
