# Home NAS - 私有云存储管理平台

跨平台 NAS 文件管理系统，包含 Rust 后端和 React Native 前端。

## 功能特性

- 📁 **多应用类型**: 媒体、视频、音乐、文档，各有专属展示方式
- 🎬 **智能视频识别**: 自动识别海报、NFO 元数据、蓝光结构、剧集等
- 🎵 **音乐播放**: 支持歌词同步显示
- 📤 **文件上传**: 支持指定目录上传
- 🔗 **文件分享**: 链接过期、阅后即焚、访问次数限制
- 👥 **用户管理**: 管理员/普通用户角色
- 🐳 **容器部署**: Docker Compose 一键启动

## 技术栈

### 后端
- Rust + Axum
- SQLx + PostgreSQL
- SQLx migrations
- JWT 认证

### 前端
- React Native (Expo)
- Tamagui UI
- Zustand 状态管理
- TanStack Query 数据请求

## 快速开始

### 1. 配置

编辑 `config.yaml`，设置你的 NAS 目录路径：

```yaml
apps:
  medias:
    - photos:/your/photos/path
  videos:
    - movies:/your/movies/path
  music:
    - library:/your/music/path
  documents:
    - docs:/your/documents/path
```

### 2. 启动后端

```bash
# 使用 Docker Compose
docker-compose up -d

# 或本地开发
cd backend
cargo run
```

### 3. 启动前端

```bash
cd app
npm install
npm start
```

## 项目结构

```
home-nas/
├── backend/          # Rust 后端
│   ├── src/
│   │   ├── config/   # 配置解析
│   │   └── main.rs   # Axum 路由与 SQLx 数据访问
│   └── migrations/   # SQLx 数据库迁移
├── app/              # React Native 应用
│   ├── src/
│   │   ├── api/      # TanStack Query hooks
│   │   ├── components/ # UI 组件
│   │   ├── screens/  # 页面
│   │   ├── stores/   # Zustand 状态
│   │   └── navigation/ # 路由
│   └── tamagui.config.ts
├── config.yaml       # 配置文件
├── docs/            # 需求与设计文档
└── docker-compose.yml
```

## API 端点

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | /api/auth/login | 登录 |
| GET | /api/files/{app} | 列出文件 |
| GET | /api/media/stream/{path} | 流媒体播放 |
| POST | /api/shares | 创建分享 |
| POST | /api/files/{app}/upload/{path}?filename=xxx | 上传文件到目录 |
| GET | /s/{token} | 访问分享 |

## License

MIT


## 需求文档

详见 `docs/requirements.md`。
