# Home NAS 需求文档（重构版）

## 1. 项目目标
Home NAS 是一个私有云存储管理平台，面向家庭/小团队场景，提供多类型目录统一管理、媒体浏览播放、分享链接和基础用户权限控制。

本次重构后的后端技术基线：
- Rust
- Axum Web Framework
- PostgreSQL
- SQLx

## 2. 角色与权限

### 2.1 角色定义
- **admin**：系统管理员
- **user**：普通用户

### 2.2 权限矩阵
- 登录：admin / user
- 查看自身信息：admin / user
- 浏览目录、文件操作：admin / user
- 创建/查看/删除自己的分享：admin / user
- 用户管理（查看、修改、删除用户）：仅 admin
- 新建用户：仅 admin

## 3. 功能需求

### 3.1 认证与用户
1. 系统提供账号密码登录接口，返回 JWT。
2. JWT 用于后续受保护接口鉴权（Bearer Token）。
3. 提供 `/api/auth/me` 返回当前登录用户信息。
4. 仅 admin 可以创建新用户。
5. 仅 admin 可以查看用户列表、修改用户角色或密码、删除用户。

### 3.2 文件管理
1. 系统支持按应用分类浏览目录：`medias`、`videos`、`music`、`documents`。
2. 路径基于 `config.yaml` 中的目录映射解析。
3. 提供目录列表能力，返回名称、路径、目录标识、大小等基础信息。
4. 支持文件 copy/move 操作。
5. 支持删除文件或目录。

### 3.3 分享能力
1. 登录用户可创建分享链接。
2. 分享包含：文件路径、应用类型、过期时间（可选）、阅后即焚（可选）、最大访问次数（可选）。
3. 用户可查看和删除自己的分享。
4. 通过短链接 token 访问分享时，系统需校验有效期和访问次数。
5. 每次访问成功后增加访问计数。

## 4. 非功能需求

### 4.1 安全
- 所有受保护接口都必须校验 JWT。
- 密码使用 bcrypt 哈希存储。
- 管理员接口必须进行角色校验。

### 4.2 可运维性
- 支持 `CONFIG_PATH`、`DATABASE_URL`、`BIND_ADDR` 等环境变量覆盖。
- 启动时输出结构化日志。
- CORS 默认放开（开发阶段），后续可收敛。

### 4.3 数据存储
- 数据库为 PostgreSQL。
- 数据访问统一采用 SQLx。
- 通过 SQLx migrations 管理并初始化核心表（users、shares）。

## 5. 数据模型要求

### 5.1 users
- id (PK)
- username (unique)
- password_hash
- role
- created_at
- updated_at

### 5.2 shares
- id (PK)
- user_id (FK -> users.id)
- file_path
- app_type
- token (unique)
- expires_at (nullable)
- burn_after_read
- max_views (nullable)
- view_count
- created_at

## 6. API 需求（当前阶段）

### 6.1 认证
- `POST /api/auth/login`
- `POST /api/auth/register`（admin）
- `GET /api/auth/me`

### 6.2 用户管理
- `GET /api/users`（admin）
- `PATCH /api/users/:id`（admin）
- `DELETE /api/users/:id`（admin）

### 6.3 分享
- `POST /api/shares`
- `GET /api/shares`
- `DELETE /api/shares/:id`
- `GET /s/:token`

### 6.4 文件
- `GET /api/files/:app`
- `GET /api/files/:app/*path`
- `PATCH /api/files/:app/*path`
- `DELETE /api/files/:app/*path`

## 7. 配置需求
`config.yaml` 需支持：
- `global.web-url`
- `global.jwt-secret`
- `apps.medias/videos/music/documents`（支持 `name:path`）
- `database.host/port/name/user/password`

## 8. 当前边界与后续开发建议
1. 媒体缩略图/歌词/媒体信息接口当前处于迁移中状态，后续需按旧行为补齐。
2. 建议增加集成测试：认证流程、分享过期与访问次数、文件路径安全校验。
3. 建议收紧 CORS 配置并引入刷新令牌机制。
