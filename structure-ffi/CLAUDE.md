# CLAUDE.md — structure-ffi

- **The FFI layer is a thin DTO-mapping shell.** For each domain type `X` there's an `XDTO` in `dto/planning.rs` annotated `#[frb]`, with `From<&X> for XDTO` and (where input is needed) `From<XDTO> for X`. `api/` functions are `#[frb(sync)]`, open the DB (`connection::init_db("structure.db")`), call into `structure-core`, and map rows/domain types to DTOs. Put no business logic here.
