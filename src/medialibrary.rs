use anyhow::Result;
use rusqlite::{Connection, NO_PARAMS};
use serde_derive::{Deserialize, Serialize};
use serde_rusqlite::*;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Entry {
    // item
    pub disc_number: u16,
    pub track_number: u16,
    // item_extra
    pub title: String,
    pub disc_count: u16,
    pub track_count: u16,
    pub location: String,
    // base_location
    pub path: String,
    // item_artist
    pub item_artist: Option<String>,
    // album
    pub album: Option<String>,
    // album_artist
    pub album_artist: Option<String>,
    // genre
    pub genre: Option<String>,
}

pub fn read_entries(conn: &Connection) -> Result<Vec<Entry>> {
    let mut stmt = conn.prepare(
        r#"SELECT * FROM item
            INNER JOIN item_extra USING(item_pid)
            INNER JOIN base_location USING(base_location_id)
            LEFT JOIN item_artist USING(item_artist_pid)
            LEFT JOIN album USING(album_pid)
            LEFT JOIN album_artist USING(album_artist_pid)
            LEFT JOIN genre USING(genre_id)
            WHERE location != "" AND path != ""
        "#,
    )?;
    let items = from_rows::<Entry>(stmt.query(NO_PARAMS)?)
        .flatten()
        .collect();
    Ok(items)
}
