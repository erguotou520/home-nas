<!-- OPENSPEC:START -->
# OpenSpec Instructions

These instructions are for AI assistants working in this project.

Always open `@/openspec/AGENTS.md` when the request:
- Mentions planning or proposals (words like proposal, spec, change, plan)
- Introduces new capabilities, breaking changes, architecture shifts, or big performance/security work
- Sounds ambiguous and you need the authoritative spec before coding

Use `@/openspec/AGENTS.md` to learn:
- How to create and apply change proposals
- Spec format and conventions
- Project structure and guidelines

Keep this managed block so 'openspec update' can refresh the instructions.

<!-- OPENSPEC:END -->

# Home NAS - Project Overview

A cross-platform NAS (Network Attached Storage) file management system with Rust backend and React Native frontend.

## Tech Stack

### Backend ([backend/](backend/))
- **Language**: Rust
- **Web Framework**: Actix-web
- **ORM**: SeaORM
- **Database**: PostgreSQL
- **Auth**: JWT

### Frontend ([app/](app/))
- **Framework**: React Native (Expo)
- **UI Library**: Tamagui
- **State Management**: Zustand
- **Data Fetching**: TanStack Query
- **Language**: TypeScript

## Project Structure

```
home-nas/
├── backend/              # Rust backend server
│   ├── src/
│   │   ├── bin/         # Entry points
│   │   ├── config/      # Configuration parsing (config.yaml)
│   │   ├── handlers/    # API route handlers
│   │   ├── main.rs      # Main entry point
│   │   ├── middleware/  # JWT authentication
│   │   ├── models/      # SeaORM database entities
│   │   ├── services/    # Business logic layer
│   │   └── utils/       # Video/music parsing utilities
│   ├── migrations/      # Database migrations
│   ├── Cargo.toml       # Rust dependencies
│   └── Dockerfile       # Backend container image
├── app/                 # React Native mobile app
│   ├── src/
│   │   ├── api/         # TanStack Query hooks
│   │   ├── components/  # Reusable UI components
│   │   ├── navigation/  # App routing/navigation
│   │   ├── screens/     # Screen components
│   │   ├── stores/      # Zustand state stores
│   │   └── utils/       # Utility functions
│   ├── tamagui.config.ts
│   └── package.json
├── openspec/            # Change proposals and specs
├── config.yaml          # Main configuration file
└── docker-compose.yml   # Container orchestration
```

## Configuration

The application is configured via [config.yaml](config.yaml):

```yaml
global:
  web-url: http://localhost:8080
  jwt-secret: <secret-key>

apps:
  medias:   # Photo/image galleries
    - <name>:<path>
  videos:   # Video libraries with metadata support
    - <name>:<path>
  music:    # Music collection with lyrics
    - <name>:<path>
  documents: # Document storage
    - <name>:<path>

database:
  host: localhost
  port: 5432
  name: home_nas
  user: postgres
  password: postgres
```

## Key Features

- **Multi-App Types**: Different display modes for media, videos, music, documents
- **Smart Video Recognition**: Auto-detects posters, NFO metadata, Blu-ray structures, TV episodes
- **Music Playback**: With synchronized lyrics display
- **File Upload**: To specific directories
- **File Sharing**: With expiration, burn-after-read, access limits
- **User Management**: Admin/user roles
- **Docker Deployment**: One-command startup

## Common Tasks

### Backend Development
```bash
cd backend
cargo run                    # Run dev server
cargo test                   # Run tests
cargo build --release        # Production build
```

### Frontend Development
```bash
cd app
pnpm install                 # Install dependencies
pnpm start                   # Run Expo dev server
```

### Full Stack (Docker)
```bash
docker-compose up -d         # Start all services
docker-compose down          # Stop all services
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| POST | /api/auth/login | User authentication |
| GET | /api/files/{app} | List files by app type |
| GET | /api/media/stream/{path} | Media streaming |
| POST | /api/shares | Create share link |
| GET | /s/{token} | Access shared content |