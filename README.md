# MediaCleaner Pro

[![CI](https://github.com/xcoder-es/media-cleaner-pro/actions/workflows/ci.yml/badge.svg)](https://github.com/xcoder-es/media-cleaner-pro/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/xcoder-es/media-cleaner-pro)](https://github.com/xcoder-es/media-cleaner-pro/releases/latest)
[![License](https://img.shields.io/github/license/xcoder-es/media-cleaner-pro)](LICENSE)

Advanced perceptual duplicate image removal with a 10-stage pipeline. Processes hundreds of thousands of local images without uploading to any server. Single native binary — no Docker or cloud dependencies.

## Features

- **10-stage pipeline**: exact duplicate (SHA-256), perceptual duplicate (dHash), tiny image, icon, thumbnail, screenshot, wallpaper, document detection, AI classification, and quality ranking
- **File organization**: automatically routes duplicates, rejected files, and categorized images to configurable output directories
- **Embedded frontend**: modern browser UI served directly from the binary — no separate server or npm install needed
- **OpenAPI spec**: full auto-generated API documentation at `/api/openapi.json`
- **Cross-platform**: Windows, macOS (Intel + Apple Silicon), Linux
- **Private**: all processing happens locally — zero data leaves your machine

## Download

Get the latest binary from the [Releases](https://github.com/xcoder-es/media-cleaner-pro/releases) page.

| Platform | File | Signed |
|----------|------|--------|
| Windows x86_64 | `mediacleaner-pro-windows-x86_64.exe` | ✅ Signed via SignPath |
| macOS Apple Silicon | `mediacleaner-pro-macos-aarch64` | Notarization pending |
| macOS Intel | `mediacleaner-pro-macos-x86_64` | Notarization pending |
| Linux x86_64 | `mediacleaner-pro-linux-x86_64` | GPG signing pending |

## Quick Start

1. **Run the binary**
   ```bash
   ./mediacleaner-pro
   ```
   On first run, it auto-creates `.env` and `data/source/`, `data/output/` directories.

2. **Place your images** in `data/source/` (or edit `.env` to change the path).

3. **Open the web UI** at [http://127.0.0.1:8080](http://127.0.0.1:8080).

4. **Configure and start**: set your source/output directories, adjust the sensitivity threshold, and click Start.

## Configuration

Edit `.env` in the same directory as the binary:

| Variable | Default | Description |
|----------|---------|-------------|
| `SERVER_HOST` | `127.0.0.1` | Bind address |
| `SERVER_PORT` | `8080` | HTTP port |
| `SOURCE_DIR` | `./data/source` | Input image directory |
| `DEST_DIR` | `./data/output` | Output organized files |
| `HAMMING_THRESHOLD` | `4` | Perceptual similarity threshold (0-64, lower = stricter) |
| `MIN_WIDTH` | `100` | Minimum image width in pixels |
| `MIN_HEIGHT` | `100` | Minimum image height in pixels |

## File Organization

After processing, files are organized under `DEST_DIR`:

```
output/
├── duplicates/
│   ├── exact/
│   └── perceptual/
├── rejected/
│   └── tiny/
├── categories/
│   ├── icons/
│   ├── thumbnails/
│   ├── screenshots/
│   ├── wallpapers/
│   └── documents/
└── quality/
    ├── excellent/
    ├── good/
    ├── average/
    ├── below_average/
    └── poor/
```

Unique kept files with no classification stay in the source directory.

## API

The API is fully documented at `/api/openapi.json` (OpenAPI 3.1). Key endpoints:

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/health` | Health check |
| `GET` | `/api/status` | Current pipeline state |
| `POST` | `/api/start` | Start a processing job |
| `POST` | `/api/control` | Pause/resume/cancel |
| `GET` | `/api/progress` | SSE progress stream |
| `GET` | `/api/logs` | Log messages |
| `GET` | `/api/browse` | Browse local directories |

## Building from Source

Requires [Rust](https://rustup.rs/) 1.75+.

```bash
git clone https://github.com/xcoder-es/media-cleaner-pro.git
cd media-cleaner-pro
cargo build --release
```

The compiled binary is at `target/release/mediacleaner-pro`.

### Optional: Build with Frontend

If Node.js is available, the frontend can be rebuilt:

```bash
cd frontend
npm install
npm run build
cd ..
cargo build --release
```

Otherwise, a minimal placeholder frontend is built in automatically.

## Project Status

Desktop MVP v1.0 — all core pipeline stages, file organization, and web UI are functional. Cloud/SaaS features (team accounts, sync protocol, Stripe billing) are under development.

## License

MIT
