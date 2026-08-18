# kivro-site

Documentation website for [kivro](../README.md). Leptos (CSR) built with
[Trunk](https://trunkrs.dev) into a static bundle — no server runtime required.

This crate is excluded from the root cargo workspace, so `cargo build` and
`cargo test` at the repo root stay native-only and are unaffected by the wasm
target.

## Prerequisites

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk
```

## Develop

```bash
cd site
trunk serve            # http://127.0.0.1:8080, rebuilds on change
```

## Build

```bash
trunk build --release              # -> site/dist, deployed at the domain root
trunk build --release --public-url /kivro/   # deployed under a subpath
```

`--public-url` matters for GitHub Pages project sites, which serve from
`/<repo>/` rather than the domain root.

Because routing is client side, the host must serve `index.html` for unknown
paths, otherwise a deep link such as `/docs/cli` 404s on reload. On GitHub Pages
that means copying `dist/index.html` to `dist/404.html` after building.

## Layout

| Path                | Contents                                                     |
| ------------------- | ------------------------------------------------------------ |
| `index.html`        | Trunk entry point; declares the wasm, CSS and asset pipelines |
| `Trunk.toml`        | Build, watch and dev-server configuration                     |
| `src/main.rs`       | Mounts the app to the document body                           |
| `src/app.rs`        | Root component: metadata context and route table              |
| `src/routes.rs`     | URL constants and the documentation navigation list           |
| `src/components/`   | Shared chrome: layout, header, sidebar, footer                |
| `src/pages/`        | One module per route                                          |
| `styles/main.css`   | Structural layout styles                                      |
| `public/`           | Static assets copied verbatim into `dist/public`              |

## Adding a page

1. Add a `Doc` entry to `DOCS_NAV` in `src/routes.rs`. The sidebar picks it up
   automatically.
2. The `/docs/:slug` route renders it; give it dedicated markup only if the
   shared `DocPage` component is not enough.

Page bodies are intentionally empty — content has not been written yet. The
source material lives in [`../docs`](../docs).
