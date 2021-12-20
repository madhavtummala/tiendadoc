use std::path::Path;
use rusqlite::{Result, Connection};

pub fn get_connection(db_file: &str) -> Result<Connection> {
    let db_file_exists = Path::new(db_file).exists();
    let conn= Connection::open(db_file)?;
    if !db_file_exists { init_tables(&conn); }
    Ok(conn)
}

pub fn init_tables(conn: &Connection) -> Result<()> {
    conn.execute(
        "create table keys (
                  keyword            TEXT NOT NULL,
                  file_id            TEXT NOT NULL
                  )", [])?;
    Ok(())
}