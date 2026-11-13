# OPC

面向单人公司和早期创业团队的本地 ERP。业务入口是 `opc-server` 和 `opc`，其余文档、部署和脚本都围绕这两个二进制组织。源码在 `source/opc`。

## 项目定位

这个项目不是泛泛的“管理后台模板”，而是一个围绕单人企业实际工作流搭起来的全栈系统。它的重点是：

- 统一处理公司、员工、发票、银行、预算、资产、合同和文件等业务对象
- 把业务规则、权限和多租户隔离放在 HTTP API 里统一维护
- 用 CLI 作为脚本化入口，便于自动化和 AI 调用链接入
- 保留独立的 Web 前端，给日常操作和运营查看使用

仓库中与这个项目直接相关的内容主要分成三块：

- `source/opc/backend`：Rust 后端，`opc-server` 和 `opc` 都从这里构建
- `source/opc/frontend`：React + Vite 前端
- `source/opc/docs`：项目文档入口和详细参考

## 创始人决策辅助、合同审查与文档识别

聊天助手读取当前公司已录入的经营汇总，不读取其他公司或发送员工明细、完整合同及凭据。提示词要求区分事实、假设和信息缺口，综合现金流、客户、交付、人力及合同约束，比较可行选项、最坏情况、机会成本和可逆性，再给出验证步骤与停止条件。空台账不代表经营健康，月度净流量不代表银行余额；建议不自动执行付款、签约或业务写入。模型输出质量仍需人工判断，不能把提示词测试当成法律结论或经营收益的验证。

聊天面板可从服务端配置的 OpenAI 兼容网关读取模型目录并为当前对话选择模型；支持 W-MaaS 的 `/v1/models`。选择不修改系统默认值、其他公司或后台合同审查配置。目录读取失败时允许明确使用服务端默认模型，指定模型请求则报错，不静默换模型。部署仍由 `AI_API_URL`、`AI_API_KEY`、`AI_MODEL` 决定，使用 MaaS SDK 不等于已经连接 MaaS；本次代码不切换生产凭据。旧设置中的 `ai_configurations` 记录并不改变上述运行时默认配置。

OCR 保留 `ollama-vision` 和 `openai-vision` 两条现有后端；XML/OFD/带结构化数据的 PDF 发票继续优先直接解析。发票确认表单以价税合计入账，按最终应收/应付类型选择交易对方；换文件或关闭表单后丢弃旧识别结果。识别结果仍需用户确认，模型两次未返回有效字段时明确失败，不自动入账。

[BREAKING CHANGE / CONTRACT IMPACT] 新增受认证保护的 `GET /api/ai/models`，聊天请求可选 `model`（仅当前网关目录允许的 ID）。空白或超限聊天/合同输入返回 400/413，空模型响应及无有效 OCR 字段返回 502；不再以 `success: true` 包装失败。合同创建/更新忽略客户端传入的 AI 审查状态、风险和结果，标题本身不触发审查，缺正文和备注时手动重审返回 400。编辑后审查完整已保存内容，旧任务不能覆盖新版本；无有效风险分级的回复不再默认低风险。HTTP/CLI/前端调用者须处理上述状态，契约见 `/openapi.json`。创业激活的收款步骤仅统计已完成、关联当前公司应收发票的收入。

## 邮件渠道

1.0.8 增加可选 W-Mail 渠道。公司在邮件设置中选择 `provider=w-mail`；原 SMTP 主机、用户名和密码保留，切换不会把平台凭据写入公司配置。服务端仅从 Radar 物化的 `W_ERP_MAIL_URL`、`W_ERP_MAIL_PROJECT_ID`、`W_ERP_MAIL_MAILBOX_ID`、敏感 `W_ERP_MAIL_API_KEY` 读取受管项目身份，实际发件人由 W-Mail 的项目邮箱固定。W-ID 证明用户身份，现有公司成员/API Key 权限继续决定可访问的业务范围；平台管理员身份不授予 ERP 公司成员资格。

W-Mail 发送通过原 `POST /api[/v1]/email/send`，正文 `body` 是 HTML，可选 `text` 是纯文本。调用者必须提供 `operation_id`（8–128 位 ASCII 字母、数字、下划线或连字符），同一业务操作的重试始终复用它。更改内容后重用原操作键返回 409。发送前在既有 `email_logs` 提交结果未知标记；正文不复制到队列或日志，W-Mail 的已发送目录负责保存邮件内容。超时、404 或丢失回执后只查询原操作，最多 6 次/2 天，停止查询后永久保留未知状态和历史回执，不自动重发或删除。修改公司所选渠道不改变已提交操作的原项目/邮箱绑定。

记录区分 `provider_accepted`（上游受理）、`delivered`（收件服务器接收）、`failed` 与 `unknown`。普通日志只记录操作/任务/提供商 ID、状态与固定错误码；公司隔离的产品发送记录继续保留收件人、主题及历史元数据。不会自动追发新增字段之前的历史记录。备份恢复后的已存在操作标记只允许查询。

[BREAKING CHANGE / CONTRACT IMPACT] 原 SMTP 路径没有实际传输实现，曾仅凭配置存在便模拟成功。现在完整 SMTP 配置的发送返回 `503 smtp_transport_unsupported`，不再误报已发送；配置和未配置/缺凭据的既有 502 行为保留。用户可选择已配置的 W-Mail 通道进行真实发送。API/CLI/前端调用方应检查状态，202 不代表已递送；契约见 `/openapi.json`。本节描述源码行为，当前生产启用与投递验收以 Radar 为准。

本源码使用 `w-einvoice =0.1.1` 作为结构化发票解析入口。[BREAKING CHANGE / CONTRACT IMPACT] 该 SDK 相对注册表 0.1.0 将 Rust `PartialEInvoice.source` 改为可选字段；ERP 在缺少来源时进入原 OCR 回退，不猜测格式，外部 HTTP 请求和响应结构不变。`genpdfi` 的传递 PDF 依赖仍存在历史漏洞债务；必须在依赖重新解析后独立检查，不宣称已通过当前审计。

## 租户归属与部署边界

W-ID 用户是身份主体；ERP 的 `companies` 是业务租户，`company_members` 保存实际成员关系。产品注册所属 workspace 不代表客户组织，当前也没有 W-ID workspace 与 ERP company 的自动映射。登录会话选择公司不授予成员资格：每次认证都检查成员仍存在且公司未删除，API Key 同样受此限制。用户提供的业务凭据属于 ERP 内对应客户，不进入平台共享资产。

当前一个实例使用一个 SQLite 数据库，业务记录按 `company_id` 逻辑隔离，并非每家公司独立数据库。按 workspace 子域名、客户 CNAME 访问尚未实现；增加这些入口时必须验证域名绑定到具体产品和客户公司，不能根据 Host 自动授予权限。多实例运行还需处理数据库、上传文件、会话、定时任务与备份的共享或唯一执行，当前不能直接复制服务实现横向扩容。

[BREAKING CHANGE / CONTRACT IMPACT] 租户成员被移除或公司删除后，已有会话和 API Key 不再允许继续访问该公司。实例备份包含所有公司及会话数据，`/api[/v1]/settings/backups`、下载及 restore 路由对全部产品用户（含公司 owner/admin）返回 403；设置页不再提供这些操作或暴露整库备份元数据，CLI 的远程 backup 命令也受此限制。服务器内部自动备份保留，由运维管理；公司范围的完整备份/恢复尚未实现。API Key 不得签发新的 API Key，创建密钥需用户会话，避免受限密钥扩大自身权限。

## 开票与收付款口径

报价转发票继承报价币种；有明细时在转换事务内重新计算金额和税额。报价及工时转换在同一 SQLite 写事务内读取资格、生成发票并关联源记录，并发请求不能重复转换同一来源。

结算只计入 `completed` 和既有 `paid` 收付款；`pending`、`cancelled`、未知或空状态不抵扣应收/应付。发票、工资、催收、工作台和相关收支报表采用此口径；付款记录列表与记录数量仍保留所有状态。修改付款金额、状态、关联或删除付款时，同事务重算关联单据；改挂会同时更新原单据与新单据。工资的付款派生结算可撤销，并且只关联实际存在的有效付款。Agent 批准付款也执行相同结算同步。

[BREAKING CHANGE / CONTRACT IMPACT] JSON `null` 清除可空字段，存为 SQL NULL；待处理/取消付款不再计入结算金额。请求及响应结构不变，旧 `paid` 状态继续兼容。此代码变更不批量重写历史单据，历史结算状态需要在上线时结合实际收付款核对。

## 目录与职责

- `source/opc/backend/src`：后端核心代码，包含 API、认证、数据库、AI 集成、工作台、运行时配置等模块
- `source/opc/backend/src/db/schema.rs` + `db/mod.rs`：schema 唯一来源（INIT_SQL + 带版本号的 ALTER）；无自动执行的 `migrations/` 目录
- `source/opc/backend/scripts`：导入、修复、全量测试和端到端验证脚本
- `source/opc/frontend`：Web 界面源码和前端构建配置
- `source/opc/docker-compose.yml`：本地容器运行方式

## 启动与运行

### 本地开发

后端：

```bash
cd source/opc/backend
cargo run --bin opc-server
```

前端：

```bash
cd source/opc/frontend
npm install
npm run dev
```

CLI：

```bash
cd source/opc/backend
cargo run --bin opc -- --help
```

### 容器运行

仓库里提供了本地 Docker Compose 方案：

```bash
cd source/opc
docker compose up -d
```

### 构建

```bash
cd source/opc/frontend
npm run build

cd ../backend
cargo build --release --bin opc-server
```

## 测试与验证

推荐按层次验证：

```bash
python3 source/opc/backend/scripts/test_rust_test_layout.py
```

```bash
cd source/opc/backend
cargo test --locked
```

```bash
cd source/opc/frontend
npm run build
```

仓库里还提供了更细的验证脚本，集中在 `source/opc/backend/scripts/`，包括 API、全栈、端到端和阶段性回归检查。

Rust 后端集成测试通过 `tests/integration.rs` 聚合为单个 test target。不要恢复 Cargo 对 `tests/s*.rs` 的自动发现，也不要在 Gitea Runner 中缓存或保留 `source/opc/backend/target/`；否则每个集成测试文件都会生成一份约数百 MB 的调试测试二进制，`target` 会快速膨胀到几十 GB。

### 语数科技完整 E2E 测试数据集

后续端到端测试可以统一基于“语数科技”测试租户。当前完整数据集覆盖 8 个功能域、63 条核心业务记录。默认种子器是非破坏性的：复用已存在记录，只补齐缺失记录；需要修复旧测试库字段时加 `--refresh-existing`。

```bash
cd source/opc/backend
python3 scripts/seed_complete_e2e_dataset.py \
  --base-url http://127.0.0.1:8987 \
  --manifest /tmp/yushu-e2e-manifest.json

python3 scripts/validate_complete_e2e_dataset.py \
  --base-url http://127.0.0.1:8987 \
  --manifest /tmp/yushu-e2e-manifest.json

python3 scripts/audit_complete_e2e_dataset_coverage.py
```

如果是全新的隔离 SQLite 测试库，先启动后端让迁移建表，再用 `--bootstrap-sqlite <database.sqlite>` 创建测试用户、语数科技公司和 session。

部署完成后，最基本的健康检查是：

```bash
curl -I http://127.0.0.1:8987/health
```

## OpenAPI 与接口发现

`opc-server` 公开标准 OpenAPI 3.1 文档，供 Radar、Gitea README 抽取、AI 分析和自动化客户端统一发现接口能力：

- 默认入口：`http://localhost:8987/openapi.json`
- API 别名：`http://localhost:8987/api/openapi.json`
- 本地入口：`http://127.0.0.1:8987/openapi.json`
- 本地 API 别名：`http://127.0.0.1:8987/api/openapi.json`

OpenAPI 文档由当前 Rust/Axum 路由声明生成，覆盖 `/api/v1` 和 legacy `/api` 两套业务前缀，也覆盖 `/health` 与 W-ID 登录路径。业务 API 仍通过 session bearer 或 `x-api-key` 鉴权；OpenAPI 文档本身保持公开，方便内部工具读取完整上下文。

## 配置与依赖

这个项目的关键运行参数主要来自环境变量和 SQLite：

- `DATABASE_URL=sqlite:data/database.sqlite?mode=rwc`
- `W_ID_SSO_URL`
- `W_ID_CLIENT_ID`
- `W_ID_CLIENT_SECRET`
- `W_ID_REDIRECT_URI`
- `ALLOWED_ORIGINS`
- `APP_URL`

本地 Compose 方案使用 `w-opc-data` 卷保存数据。

外部依赖主要是：

- W-ID 作为登录与 OAuth 来源
- SQLite 作为项目数据库
- Caddy 或等价反向代理作为公网入口

## 常见问题

- 登录回调失败时，优先核对 `W_ID_REDIRECT_URI`、`APP_URL` 和 `ALLOWED_ORIGINS` 是否一致
- 前端空白或静态资源 404 时，先确认 `npm run build` 是否已经生成 `frontend/dist`
- 数据写入异常时，先确认 SQLite 路径和运行时目录权限
- 如果部署到生产环境，公网入口应走边缘反代，不要直接把应用暴露到公开 `:80` / `:443`
- 业务与 CLI 的命令名以 `backend/Cargo.toml` 为准，文档里不要手写一个不同的入口名

后端在 HTTP 监听成功后通过 xradar 上报真实进程 capabilities，立即一次，此后每 900 秒一次。
配置 `W_RADAR_CODE`、`W_RADAR_ENDPOINT`、`W_RADAR_PROJECT_ID=w-erp` 和部署唯一的
`W_RADAR_INSTANCE_ID`；没有采集码时明确记录禁用。已有正式
`W_RADAR_ARCHITECTURE_API_KEY` 的部署同时发送 10 秒架构心跳（TTL 30 秒）；
未配置时 capabilities 仍独立工作。采集失败不阻塞业务，下周期恢复；SIGTERM/SIGINT
停止上报后排空 HTTP 请求。验证实际发布二进制：

```bash
python3 scripts/test_backend_heartbeat.py source/opc/backend/target/x86_64-unknown-linux-musl/release/opc-server
```

## OCR 回归测试

公开合同 PDF、发票示例、收据照片与合成回归样本统一维护在 [OCR 测试集](tests/ocr/README.md)。清单记录来源、许可、SHA-256 和人工标准答案；本机 Ollama 分别测试文字识别及字段抽取，公开候选仅附现有离线完整性检查脚本，未运行 CI。

合同条款理解与商业风险判断另见 [合同陷阱快速 benchmark](tests/contracts/README.md)：成对风险/保护例、缺失信息与证据定位，支持 6 例快速测试和 18 例完整测试。

## 公开源码范围与验收边界

本候选保留完整 ERP 的后端、前端、CLI、SSO、遥测、AI/OCR、发票、合同、CRM、人事、报表和租户权限实现。原应用版本为 1.0.8。
私有部署文件、私有 CI、历史 Git 对象、运行数据、凭据和未获再分发许可的 Arial 字体不在此分发中。
`THIRD-PARTY-NOTICES.md` 记录随附第三方材料；`source-provenance.json` 和 `public-files.json` 固定范围及来源。
公开源码不会配置任何生产租户、遥测码或平台客户端；请填写自己的 OAuth、AI、邮件和遥测参数。

本地候选审查不安装依赖、不编译或运行产品测试。自有代码按 MIT 许可开放，第三方材料保留各自条款。
公开 CI 的结果以具体提交的 GitHub 检查为准；它不代表生产部署或完整应用运行验收。

## 公开 CI

`.github/workflows/ci.yml` 使用标准 Linux x64 runner、Node 24 和 Rust 1.96，先构建前端，再检查后端和 CLI。
工作流执行类型检查、既有前后端测试、离线样本校验，以及面向 loopback Radar 的心跳检查；不使用生产 OAuth、邮件、模型或遥测凭据。
既有 webhook 测试会向 httpbin 发送合成测试事件，并检查网络失败时的结构化响应；其余服务测试使用内存数据库和本机 mock。
CI 不发布二进制、容器、运行数据或安装目录。源码分发保留 `public-files.json` 列出的文件、公共 SDK 入口及所有许可声明；编译产物的许可闭包和正式发布另行验收。

## 公共依赖

Rust W 库使用 `Cargo.toml` 中固定的公开 Git 提交；`w-notify` 对应公共版本 0.1.3，调用的 `Channel::from_url`、`Message::new/title` 和 `send` 入口经静态核对保留。
浏览器 `@w/http-web` 和 `@w/xradar` 来自 W-Radar 的固定公开 SDK 子目录，随附于 `vendor/`，通过本地 `file:` 依赖引用，无私有 npm token。
Cargo 锁文件仅通过公开 Git pin 和 crates.io sparse 索引重新解析；本地只执行元数据解析。公共 CI 使用锁文件安装和验证依赖，结果归属于具体源码提交。
容器从仓库根构建，内含前端 dist 和字体；它是基于 Debian 的动态容器产品，不是已验收的静态 Linux 正式二进制。

## 许可

W 自有代码按 [MIT](LICENSE) 授权。第三方材料遵循 [各自许可和来源声明](THIRD-PARTY-NOTICES.md)，不以项目 MIT 覆盖未知权属内容。
