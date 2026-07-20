# MediaCleaner Pro — Architecture & Conventions

## Mission
Process hundreds of thousands of local images through a 10-stage perceptual duplicate removal pipeline without uploading files to any server. Deliver as a single native binary (desktop) or a SaaS control plane (cloud) from the same codebase.

## Architecture: Hexagonal Modular Monolith

```
┌─────────────────────────────────────────────────────────────┐
│                      API Layer (Axum)                        │
│  REST (/api/*)  │  SSE (/api/progress)  │  utoipa OpenAPI   │
└────────────────────┬──────────────────────────────────────────┘
                     │
┌────────────────────▼──────────────────────────────────────────┐
│              Application Services (Event-driven)              │
│  PipelineService  │  JobService  │  SyncService  │  Team...   │
│  Commands → Domain Events → Projections → Query Responses    │
└────────────────────┬──────────────────────────────────────────┘
                     │
┌────────────────────▼──────────────────────────────────────────┐
│              Domain Core (Pure Rust, no I/O)                  │
│  ImageData  │  Job  │  Stage  │  DuplicateSet  │  PipelineConfig
│  Ports: FileSystem, JobRepo, ImageHasher, ImageDecoder,       │
│         PipelineStage, NotificationBus, UserRepo, SyncService │
└────────────────────┬──────────────────────────────────────────┘
                     │
┌────────────────────▼──────────────────────────────────────────┐
│              Adapters (Swappable behind feature flags)        │
│  fs/sftp FileSystem  │  sqlx/SQLite/Postgres  │  sha256/dhash│
│  image-rs Decoder    │  SSE Notification    │  JWT Auth       │
└─────────────────────────────────────────────────────────────┘
```

## Workspace Layout

```
Cargo.toml                    # Workspace root (mediacleaner-pro package)
AGENTS.md
sqlx-data.json                # Compile-time query cache for sqlx
packages/
├── mc-core/                  # Domain models, ports (traits), events, utils
├── mc-infra/                 # All adapters (behind feature flags)
└── mc-api/                   # Axum routes, utoipa derives, middleware
apps/
└── desktop/                  # Binary entrypoint (future)
src/                          # Current codebase (being migrated)
  ├── api/                    # → mc-api
  ├── processing/             # → mc-core (domain) + mc-infra (impls)
  ├── state/                  # → mc-core (domain) + leftover AppState
  ├── temporal/               # → mc-infra (temporal adapter)
  ├── config.rs               # → stays in app layer
  ├── main.rs                 # → apps/desktop
  └── lib.rs                  # → removed (workspace members)
frontend/                     # Astro + React (shared across apps)
docs/
├── enterprise-spec.md        # Full enterprise/paid tier specification
├── sync-protocol.md          # Desktop ↔ Cloud sync protocol
└── billing-model.md          # Free/paid tiers, Stripe integration
```

## Build Modes (Cargo features)

| Feature     | Binary | Frontend    | Database        | Sync   | Target       |
|-------------|--------|-------------|-----------------|--------|--------------|
| `desktop`   | Native | `rust-embed` | SQLite           | none   | Free users   |
| `cloud`     | Docker | Separate    | PostgreSQL + Redis | Stripe | SaaS/Enterprise |

`desktop` is the default. `cloud` adds cloud-only code (sync endpoints, auth, team management, billing).

## Coding Principles

### Single Responsibility
One port, one adapter, one service. If a trait has "and" in its name, split it.
```rust
// GOOD
pub trait FileSystem { fn read(&self); fn write(&self); }

// BAD
pub trait FileSystemAndHashing { fn read(&self); fn hash(&self); }
```

### DRY
Shared logic lives in `mc-core::domain` or `mc-core::services`. Pipeline stages use the same `ImageHasher` and `ImageDecoder` ports — never duplicate hash or decode logic.

### KISS
- Desktop mode stores state in SQLite + in-memory cache, not Redis
- Binary distribution over package manager complexity
- SSE over WebSockets for progress streaming
- Tasks over threads (tokio), rayon for CPU-bound work

### Strongly Typed
- `newtype` wrappers for IDs: `JobId(String)`, `UserId(String)`, `TeamId(String)`
- Where possible, use `enum` over stringly-typed fields
- All port methods return `Result<T, DomainError>` — stringly-typed errors are forbidden
- `sqlx-data.json` ensures every SQL query is validated at compile time

### BDD
Tests live in `packages/mc-core/tests/features/` using the `cucumber` crate. Each scenario tests a service through its ports with mock adapters:

```gherkin
Feature: Exact Duplicate Detection
  Scenario: Two identical images are detected
    Given an image "photo.jpg" with sha256 "abc123"
    When I process "photo.jpg" through the pipeline
    Then it should be marked as "unique"
    Given an image "copy.jpg" with sha256 "abc123"
    When I process "copy.jpg" through the pipeline
    Then it should be marked as "exact_duplicate"
```

## Domain Models (mc-core)

```rust
pub struct ImageMetadata {
    pub path: String,
    pub filename: String,
    pub size_bytes: u64,
    pub width: u32,
    pub height: u32,
    pub sha256: String,
    pub dhash: Option<u64>,
    pub format: String,
}

pub struct Job {
    pub id: JobId,
    pub user_id: Option<UserId>,
    pub team_id: Option<TeamId>,
    pub source_dir: PathBuf,
    pub dest_dir: PathBuf,
    pub config: PipelineConfig,
    pub stages: Vec<StageState>,
    pub stats: ProcessingStats,
    pub status: JobStatus,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub sync_status: SyncStatus,
}

pub struct PipelineConfig {
    pub hamming_threshold: u32,
    pub min_width: u32,
    pub min_height: u32,
    pub detect_icons: bool,
    pub detect_thumbnails: bool,
    pub detect_screenshots: bool,
    pub detect_wallpapers: bool,
    pub detect_documents: bool,
    pub classification_enabled: bool,
    pub quality_ranking_enabled: bool,
}

pub enum PipelineEvent {
    StageStarted { stage: usize, name: String },
    StageProgress { stage: usize, processed: usize, total: usize },
    StageCompleted { stage: usize, results: StageResult },
    JobCompleted { job_id: JobId },
    JobPaused, JobResumed, JobCancelled,
    Error { stage: usize, message: String, path: Option<PathBuf> },
    SyncStatusChanged(SyncStatus),
}
```

## Ports (Traits in mc-core)

| Port | Primary Methods | Adapter(s) |
|------|----------------|------------|
| `FileSystem` | `list_dir`, `read_file`, `write_file`, `create_dir`, `delete_file`, `copy_file`, `move_file`, `canonicalize` | `fs` (local), `sftp` (remote) |
| `JobRepository` | `create_job`, `get_job`, `update_job`, `list_jobs`, `delete_job`, `query_by_team` | `sqlx-sqlite` (desktop), `sqlx-postgres` (cloud) |
| `ImageHasher` | `compute_dhash`, `hamming_distance` | `dhash` adapter |
| `ExactHasher` | `compute_sha256` | `sha256` adapter |
| `ImageDecoder` | `decode` | `image-rs` adapter |
| `PipelineStage` | `name`, `description`, `process(image, context) -> StageResult` | 10 impls |
| `NotificationBus` | `broadcast(event)`, `subscribe() -> Receiver` | `sse` (desktop), `ws` (cloud) |
| `FileScanner` | `scan(path, extensions) -> Stream<ImageData>` | `fs`, `sftp` |
| `UserRepository` | `register`, `authenticate`, `get_user`, `get_team_members` | `sqlx-postgres` (cloud only) |
| `SyncService` | `push_results`, `pull_config`, `resolve_conflicts` | `http` (cloud only) |

## Pipeline Backpressure

```
FileScanner       (16 concurrent reads, async)
     │
     │ tokio::sync::mpsc::channel(1000)
     ▼
Hash Workers      (8 rayon threads, CPU-bound)
     │
     │ tokio::sync::mpsc::channel(1000)
     ▼
Stage Pipeline    (tokio tasks, sequential per image)
     │
     ▼
Progress Writer   (batched writes to DB every 100ms)
```

Scanner blocks when hash channel is full. Hash workers block when stage channel is full. Memory stays bounded.

## Migration Path (6 Phases)

| Phase | What | Outcome |
|-------|------|---------|
| **1** | Extract `mc-core` crate from existing `src/` | Pure domain + ports + services; all tests passing |
| **2** | Extract `mc-infra` crate | All adapters behind trait boundaries |
| **3** | Build `apps/desktop` binary | Embedded frontend + SQLite + native API; single-file distribution |
| **4** | Add `utoipa` + `sqlx-data.json` + `openapi-typescript` | Zero type drift between Rust and frontend |
| **5** | Add YAML config + magic byte validation + security hardening | Production-ready desktop mode |
| **6** | Build cloud mode: auth, sync protocol, team API, Stripe | Enterprise SaaS tier |

## Build & Distribution

```bash
# Build (current platform)
cargo build --release

# Test
cargo test --workspace

# CI: GitHub Actions release.yml
# - targets: windows-2022, macos-14 (ARM+Intel), ubuntu-22.04
# - Creates GitHub Release with compressed binaries
```

## Conventions

- **Commits**: Conventional Commits (`feat:`, `fix:`, `refactor:`, `docs:`)
- **Branch**: `main` always green, feature branches for Phases
- **Errors**: `DomainError` enum in `mc-core`, never `anyhow` in domain code
- **Logging**: `tracing` events with structured fields
- **No comments in code** — intent expressed through types, names, and tests
- **Frontend types are generated from OpenAPI**, never hand-written
