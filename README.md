# Spritedit

A sprite editor that works in the browser. Built with Rust + WebAssembly + WebGPU.

## Features

- **Isometric grid view** — edit sprites in flat or isometric projection
- **Drawing tools** — pencil, eraser, flood fill, color picker with full alpha support
- **Smooth painting** — Bresenham line interpolation for continuous strokes
- **Load sprites** — from local files (PNG, JPEG) or from a URL
- **Save sprites** — export as PNG
- **Configurable resolution** — set pixels-per-grid-box for tile-based workflows
- **Command palette** — VSCode-style `Cmd+Shift+P` to quickly access any command
- **GenAI generation** — UI for AI-powered sprite creation (backend integration ready)
- **Zoom & pan** — scroll wheel to zoom, middle-mouse to pan

## Keyboard Shortcuts

| Shortcut | Action |
|---|---|
| `Cmd+Shift+P` | Command palette |
| `Cmd+N` | New sprite |
| `Cmd+O` | Open file |
| `Cmd+S` | Save file |
| `P` | Pencil tool |
| `E` | Eraser tool |
| `F` | Fill tool |
| `I` | Color picker tool |
| `G` | Toggle grid |
| `V` | Toggle isometric view |
| Right-click | Pick color from canvas |
| Middle-mouse drag | Pan |
| Scroll wheel | Zoom |

## Prerequisites

- [Rust](https://rustup.rs/) (1.75+)
- For WASM: [trunk](https://trunkrs.dev/) (`cargo install trunk`)
- WASM target: `rustup target add wasm32-unknown-unknown`

## Running

### Native

```sh
cargo run
```

### Browser (WASM)

```sh
trunk serve
```

Then open [http://localhost:8080](http://localhost:8080) in a WebGPU-capable browser (Chrome, Edge, Firefox).

### Release build

```sh
# Native
cargo build --release

# WASM (outputs to dist/)
trunk build --release
```

## Tech Stack

- **Rust** — all application logic
- **egui / eframe** — immediate mode GUI
- **wgpu** — WebGPU rendering backend
- **trunk** — WASM bundler
- **image** — PNG/JPEG encoding and decoding
