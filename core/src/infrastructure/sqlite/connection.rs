use rusqlite::{Connection, Result};
use crate::infrastructure::sqlite::schema;

pub fn open(path: &str) -> Result<Connection> {
    let conn = Connection::open(path)?;
    conn.execute_batch(schema::CREATE_SCHEMA)?;
    Ok(conn)
}

pub fn open_in_memory() -> Result<Connection> {
    let conn = Connection::open_in_memory()?;
    conn.execute_batch(schema::CREATE_SCHEMA)?;
    Ok(conn)
}
