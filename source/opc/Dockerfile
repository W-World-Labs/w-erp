# Frontend tooling requires the Node range declared by Vite 8.
FROM node:24-alpine AS frontend-builder
WORKDIR /app
COPY vendor/ vendor/
COPY source/opc/frontend/ source/opc/frontend/
WORKDIR /app/source/opc/frontend
RUN npm ci
RUN npm run build

FROM rust:1.96-slim-bookworm AS backend-builder
WORKDIR /app/source/opc/backend
COPY source/opc/backend/ ./
COPY --from=frontend-builder /app/source/opc/frontend/dist ../frontend/dist
RUN cargo build --locked --release --bin opc-server --bin opc

FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=backend-builder /app/source/opc/backend/target/release/opc-server /app/opc-server
COPY --from=backend-builder /app/source/opc/backend/target/release/opc /app/opc
RUN mkdir -p /app/data /app/data/uploads
ENV PORT=8987 BIND_HOST=0.0.0.0 DATABASE_URL=sqlite:data/database.sqlite?mode=rwc RUST_LOG=info
EXPOSE 8987
VOLUME ["/app/data"]
CMD ["/app/opc-server"]
