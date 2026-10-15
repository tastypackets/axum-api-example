# Repository conventions

- Group API handlers by domain under `src/api/`.
- Register handlers in the domain's `mod.rs` with `OpenApiRouter::routes(routes!(handler))`. Nest domain routers under their URL prefix in `src/api/mod.rs`. Handler paths are relative to that prefix.
- Let `utoipa-axum` collect paths and schemas from route registration. Keep shared OpenAPI metadata and security declarations in `src/api/openapi.rs`.
- Require the auth extractor on protected handlers.
- Put shared dependencies in `AppState`. Use `FromRef` so handlers can extract only the dependencies they need.
- When changing an endpoint, update its OpenAPI definition and relevant auth or response tests together.
