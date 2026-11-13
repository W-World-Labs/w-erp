# 文档总入口

这个目录只负责导航，不重复展开接口、模块和部署细节。入口命名以仓库根目录和 `backend/Cargo.toml` 为准：`opc-server` 和 `opc`。

## 推荐顺序

- [仓库入口](../README.md)
- [详细参考文档](./REFERENCE.md)
- [API 接口参考](./REFERENCE.md#api-reference)
- [CLI 使用说明](./REFERENCE.md#cli-guide)

## 标准接口文档

- 生产 OpenAPI：`http://localhost:8987/openapi.json`
- API 别名：`http://localhost:8987/api/openapi.json`
- 本地 OpenAPI：`http://127.0.0.1:8987/openapi.json`
- 本地 API 别名：`http://127.0.0.1:8987/api/openapi.json`

OpenAPI 3.1 文档由后端当前 Axum 路由生成，覆盖 `/api/v1` 和 `/api` 两套路径。业务接口继续走 session bearer 或 `x-api-key` 鉴权，文档入口公开给 Radar、Gitea 和 AI 分析工具读取。

## 专题文档

- [系统架构图与技术栈](./REFERENCE.md#tech-stack)
- [项目结构](./REFERENCE.md#project-structure)
- [数据库设计](./REFERENCE.md#database-design)
- [权限体系](./REFERENCE.md#permission-system)
- [文件管理](./REFERENCE.md#file-management)
- [AI 合同审查说明](./REFERENCE.md#ai-contract-review)
- [变更记录](./REFERENCE.md#changelog)
