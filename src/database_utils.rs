use std::path::Path;
use rusqlite::{Result, Connection};

pub fn get_connection(db_file: &str) -> Result<Connection> {
    let db_file_exists = Path::new("tiendadoc.db").exists();
    let conn= Connection::open("tiendadoc.db")?;
    if !db_file_exists { init_tables(&conn); }
    Ok(conn)
}

pub fn init_tables(conn: &Connection) -> Result<()> {
    println!("came to init tables");
    conn.execute(
        "create table keys (
                  keyword            TEXT NOT NULL,
                  file_id            TEXT NOT NULL
                  )", [])?;
    Ok(())
}