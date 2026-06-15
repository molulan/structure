use std::sync::{Arc, Mutex};

use rusqlite::Connection;

use crate::persistence::connection;

/// A cheaply-cloneable handle to the database.
#[derive(Clone)]
pub struct Store {
    conn: Arc<Mutex<Connection>>,
}

impl Store {
    pub fn open(path: &str) -> rusqlite::Result<Store> {
        let conn = connection::init_db(path)?;
        Ok(Store {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Runs `f` with exclusive access to the connection and returns its result.
    pub fn with_conn<T>(&self, f: impl FnOnce(&mut Connection) -> T) -> T {
        // A panic while holding the lock poisons the mutex. A Rust panic in our
        // query code does not leave the SQLite connection in an unsafe state, so
        // we recover the guard rather than propagate the poison and bring the
        // whole process down.
        let mut guard = self
            .conn
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        f(&mut guard)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::planning::MesocycleMode;
    use crate::persistence::mesocycles;

    #[test]
    fn with_conn_reuses_one_connection_across_calls() {
        let store = Store::open(":memory:").expect("opening in-memory store should succeed");

        store
            .with_conn(|conn| mesocycles::create(conn, "hypertrophy", MesocycleMode::Manual))
            .expect("creating mesocycle should succeed");

        // An in-memory database lives only as long as its connection, so finding
        // the row in a second call proves both calls hit the same connection.
        let mesocycles = store
            .with_conn(|conn| mesocycles::list(conn))
            .expect("listing mesocycles should succeed");

        assert_eq!(mesocycles.len(), 1);
    }

    #[test]
    fn cloned_store_shares_the_same_database() {
        let store = Store::open(":memory:").expect("opening in-memory store should succeed");
        store
            .with_conn(|conn| mesocycles::create(conn, "arms", MesocycleMode::Manual))
            .expect("creating mesocycle should succeed");

        let clone = store.clone();

        let mesocycles = clone
            .with_conn(|conn| mesocycles::list(conn))
            .expect("listing mesocycles should succeed");

        assert_eq!(mesocycles.len(), 1);
    }
}
