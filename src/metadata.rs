use anyhow::{anyhow, Context, Result};
use std::ffi::OsStr;
use std::path::Path;

#[derive(Debug, PartialEq)]
pub struct Metadata {
    title: Option<String>,
    artist: Option<String>,
    album: Option<String>,
    album_artist: Option<String>,
    genre: Option<String>,
    disc: Option<u16>,
    total_discs: Option<u16>,
    track: Option<u16>,
    total_tracks: Option<u16>,
}

impl Metadata {
    pub fn new(
        title: Option<impl Into<String>>,
        artist: Option<impl Into<String>>,
        album: Option<impl Into<String>>,
        album_artist: Option<impl Into<String>>,
        genre: Option<impl Into<String>>,
        disc: Option<u16>,
        total_discs: Option<u16>,
        track: Option<u16>,
        total_tracks: Option<u16>,
    ) -> Self {
        Metadata {
            title: title.map(|v| v.into()),
            artist: artist.map(|v| v.into()),
            album: album.map(|v| v.into()),
            album_artist: album_artist.map(|v| v.into()),
            genre: genre.map(|v| v.into()),
            disc,
            total_discs,
            track,
            total_tracks,
        }
    }
}

pub fn write_metadata<P: AsRef<Path>>(path: P, meta: &Metadata) -> Result<()> {
    let path = path.as_ref();
    let ext = path.extension().and_then(OsStr::to_str);
    match ext {
        Some("mp3") => write_metadata_on_mp3(&path, &meta),
        Some("mp4") => write_metadata_on_m4a(&path, &meta),
        Some("m4a") => write_metadata_on_m4a(&path, &meta),
        Some(ext) => Err(anyhow!("Unknown file extension '{}' has specified", &ext,)),
        _ => Err(anyhow!(
            "The path '{}' does not have extension",
            &path.to_str().unwrap_or("failed to unwrap 'path'")
        )),
    }
}

fn write_metadata_on_mp3(path: &Path, meta: &Metadata) -> Result<()> {
    let mut tag = id3::Tag::read_from_path(path)?;
    meta.title.as_ref().map(|v| tag.set_title(v.to_owned()));
    meta.artist.as_ref().map(|v| tag.set_artist(v.to_owned()));
    meta.album.as_ref().map(|v| tag.set_album(v.to_owned()));
    meta.album_artist
        .as_ref()
        .map(|v| tag.set_album_artist(v.to_owned()));
    meta.genre.as_ref().map(|v| tag.set_genre(v.to_owned()));
    meta.disc.map(|v| tag.set_disc(v as u32));
    meta.total_discs.map(|v| tag.set_total_discs(v as u32));
    meta.track.map(|v| tag.set_track(v as u32));
    meta.total_tracks.map(|v| tag.set_total_tracks(v as u32));
    tag.write_to_path(path, id3::Version::Id3v24)
        .with_context(|| format!("Failed to write ID3v2.4 metadata"))
}

fn write_metadata_on_m4a(path: &Path, meta: &Metadata) -> Result<()> {
    let mut tag = mp4ameta::Tag::read_from_path(path)?;
    meta.title.as_ref().map(|v| tag.set_title(v.to_owned()));
    meta.artist.as_ref().map(|v| tag.set_artist(v.to_owned()));
    meta.album.as_ref().map(|v| tag.set_album(v.to_owned()));
    meta.album_artist
        .as_ref()
        .map(|v| tag.set_album_artist(v.to_owned()));
    meta.genre.as_ref().map(|v| tag.set_genre(v.to_owned()));
    meta.disc
        .zip(meta.total_discs)
        .map(|v| tag.set_disc(v.0, v.1));
    meta.track
        .zip(meta.total_tracks)
        .map(|v| tag.set_track(v.0, v.1));
    tag.write_to_path(path)
        .with_context(|| format!("Failed to write iTunes style MPEG-4 audio metadata"))
}
