# OPC

面向单人公司和早期创业团队的全栈 ERP。当前仓库的真实入口只有 `opc-server` 和 `opc`。

## 入口

- `opc-server`: Rust / Axum HTTP API，业务能力的唯一服务入口
- `opc`: CLI，对 `opc-server` 的薄封装，适合脚本、自动化和 AI 调用链
- `frontend`: React / Vite Web 应用

## 先看这些

- [文档总入口](./docs/README.md)
- [详细参考文档](./docs/REFERENCE.md)
- [API 接口参考](./docs/REFERENCE.md#api-reference)
- [CLI 使用说明](./docs/REFERENCE.md#cli-guide)

## OpenAPI

后端标准接口文档由 `opc-server` 直接暴露：

- `GET /openapi.json`
- `GET /api/openapi.json`

生产地址是 `http://localhost:8987/openapi.json` 和 `http://localhost:8987/api/openapi.json`。文档覆盖 `/api/v1` 与 `/api` 两套业务路径，业务接口继续要求 session bearer 或 `x-api-key`，OpenAPI 文档本身公开。

## 快速启动

以下命令均从仓库根目录运行；后端、前端和 CLI 命令分别在各自终端中执行。

```bash
cd source/opc/backend
cargo run --bin opc-server
```

```bash
cd source/opc/frontend
npm install
npm run dev
```

```bash
cd source/opc/backend
cargo run --bin opc -- --help
```

## 约定

- 业务规则、鉴权和多租户隔离只维护在 HTTP API
- CLI 只负责参数解析、调用 API、格式化输出
- 文档中的命令名必须与 `backend/Cargo.toml` 保持一致
