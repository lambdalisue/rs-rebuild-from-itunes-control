use anyhow::Result;
use clap::Clap;

use rebuild_from_itunes_control::medialibrary;
use rebuild_from_itunes_control::metadata;
use rusqlite::Connection;

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clap)]
#[clap(version = "1.0")]
struct Opts {
    input: String,
    output: String,
}

fn main() -> Result<()> {
    let opts: Opts = Opts::parse();

    let root = Path::new(&opts.input);
    let dist = Path::new(&opts.output);
    let path = root.join("iTunes_Control/iTunes/MediaLibrary.sqlitedb");
    let conn = Connection::open(path)?;
    let entries = medialibrary::read_entries(&conn)?;
    for entry in &entries {
        // Build src/dst
        let src = build_src(&entry, &root);
        let dst = build_dst(&entry, &dist);

        // Copy and update tags
        if !&src.exists() {
            println!("{:?} does not exist", &src);
            continue;
        }
        fs::create_dir_all(&dst.parent().unwrap())?;
        fs::copy(&src, &dst)?;

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
        metadata::write_metadata(&dst, &meta)?;
        println!("{:?} -> {:?}", &src, &dst);
    }

    Ok(())
}

fn build_src(entry: &medialibrary::Entry, root: &Path) -> PathBuf {
    root.join(&entry.path).join(&entry.location)
}

fn build_dst(entry: &medialibrary::Entry, root: &Path) -> PathBuf {
    // XXX: Is there any way to write the code below more simple?
    let art = entry
        .item_artist
        .as_ref()
        .map(String::to_owned)
        .unwrap_or("Unknown artist".to_string());
    let art = entry
        .album_artist
        .as_ref()
        .map(String::to_owned)
        .unwrap_or(art);
    let alb = entry
        .album
        .as_ref()
        .map(String::to_owned)
        .unwrap_or("Unknown album".to_string());
    let ext = Path::new(&entry.location)
        .extension()
        .and_then(OsStr::to_str)
        .unwrap_or("");
    root.join(art)
        .join(alb)
        .join(format!("{}.{}", &entry.title, ext))
}
