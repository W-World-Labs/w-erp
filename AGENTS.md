# W-ERP public source

W-ERP is a Rust/Axum + React/Vite ERP, with HTTP server `opc-server` and thin CLI `opc`.
Business rules, authentication and company isolation belong in the HTTP API.
Read `README.md`, `source/opc/backend/Cargo.toml` and `source/opc/frontend/package.json`.

Use the smallest direct change and preserve existing source contracts. Do not add unnecessary tests.
Keep secrets, tenant data, deployment environments and production configuration outside Git.
Build the frontend before the release backend; `build.rs` embeds `frontend/dist`.
The existing Rust integration suite is aggregated through `tests/integration.rs`; preserve `autotests = false`.
Public dependency pins and third-party notices are part of this distribution.
No private CI or production deployment workflow is shipped here.
Public CI uses a standard GitHub-hosted Linux runner to typecheck, build and run
the existing isolated tests. It uploads no compiled artifacts or deployment data.
