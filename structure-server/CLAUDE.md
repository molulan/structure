# CLAUDE.md — structure-server

- **The server is a thin HTTP layer; logic stays in `structure-core`.** Handlers take `State<Store>`, run queries via `store.with_conn(|conn| …)`, and return `Result<Json<…>, ApiError>`. Request bodies are `Deserialize` structs in `dto.rs`; `error.rs` maps each persistence error to a status code via `From<…Error> for ApiError`. Add an endpoint by extending an entity's `routes()`, not by adding logic in the handler.
