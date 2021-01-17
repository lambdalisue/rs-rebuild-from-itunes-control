use anyhow::Result;
use clap::Clap;

use rebuild_from_itunes_control::medialibrary;
use rebuild_from_itunes_control::metadata;
use rusqlite::Connection;

use std::ffi::OsStr;
use std::fs;
use std::path::Path;

#[derive(Clap)]
#[clap(version = "1.0")]
struct Opts {
    input: String,
    output: String,
}

fn main() -> Result<()> {
    let opts: Opts = Opts::parse();

    let path = Path::new(&opts.input);
    let path = path.join("iTunes_Control/iTunes/MediaLibrary.sqlitedb");
    let conn = Connection::open(path)?;
    let entries = medialibrary::read_entries(&conn)?;
    for entry in &entries {
        // Build metadata from entry
        let meta = metadata::Metadata::new(
            Some(entry.title.to_owned()),
            // XXX: Is there any way to write the code below more simple?
            entry
                .item_artist
                .as_ref()
                .map(String::to_owned)
                .or(Some("Unknown artist".into())),
            entry
                .album
                .as_ref()
                .map(String::to_owned)
                .or(Some("Unknown album".into())),
            entry
                .album_artist
                .as_ref()
                .map(String::to_owned)
                .or(Some("Unknown artist".into())),
            entry
                .genre
                .as_ref()
                .map(String::to_owned)
                .or(Some("".into())),
            Some(entry.disc_number),
            Some(entry.disc_count),
            Some(entry.track_number),
            Some(entry.track_count),
        );

        // Build src/dst
        let src = Path::new(&opts.input)
            .join(&entry.path)
            .join(&entry.location);
        let dst = Path::new(&opts.output)
            .join(
                // XXX: Is there any way to write the code below more simple?
                &entry.album_artist.as_ref().unwrap_or(
                    entry
                        .item_artist
                        .as_ref()
                        .unwrap_or(&"Unknown artist".to_string()),
                ),
            )
            .join(&entry.album.as_ref().unwrap_or(&"Unknown album".to_string()))
            .join(format!(
                "{}.{}",
                &entry.title,
                &src.extension().and_then(OsStr::to_str).unwrap_or("")
            ));

        // Copy and update tags
        if !&src.exists() {
            println!("{:?} does not exist", &src);
            continue;
        }
        fs::create_dir_all(&dst.parent().unwrap())?;
        fs::copy(&src, &dst)?;
        metadata::write_metadata(&dst, &meta)?;
        println!("{:?} -> {:?}", &src, &dst);
    }

    Ok(())
}
