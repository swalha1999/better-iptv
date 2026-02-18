use crate::cache;
use crate::model::Playlist;
use crate::parser;
use crate::xtream::XtreamProvider;
use anyhow::{Context, Result};
use std::io::{BufReader, Read};
use std::path::PathBuf;

/// Unified provider for loading playlists from different sources.
pub enum Provider {
    M3u {
        path: Option<PathBuf>,
        url: Option<String>,
    },
    Xtream(XtreamProvider),
}

impl Provider {
    /// Load a playlist from the configured provider.
    /// If fetching from a URL, caches the raw data to disk.
    pub fn load(&self) -> Result<Playlist> {
        match self {
            Provider::M3u { path, url } => {
                if let Some(p) = path {
                    parser::parse_m3u_file(p)
                } else if let Some(u) = url {
                    let response = ureq::get(u)
                        .timeout(std::time::Duration::from_secs(120))
                        .call()
                        .with_context(|| format!("Failed to fetch M3U from {u}"))?;
                    // Read into memory so we can both cache and parse
                    let mut data = Vec::new();
                    response.into_reader().read_to_end(&mut data)
                        .context("Failed to read M3U response")?;
                    // Cache to disk
                    cache::save_playlist(&data, Some(u));
                    let cursor = std::io::Cursor::new(data);
                    parser::parse_m3u(BufReader::new(cursor))
                } else {
                    anyhow::bail!("M3U provider requires either a path or URL")
                }
            }
            Provider::Xtream(xtream) => xtream.fetch_playlist(),
        }
    }

    /// Try loading from cache first. Returns None if no cache exists
    /// or if the source URL has changed since the cache was written.
    pub fn load_cached(&self) -> Option<Playlist> {
        match self {
            Provider::M3u { url: Some(u), .. } => {
                if cache::playlist_source_changed(u) {
                    return None; // URL changed — force fresh fetch
                }
                let path = cache::load_playlist_path()?;
                parser::parse_m3u_file(&path).ok()
            }
            Provider::Xtream(_) => {
                let path = cache::load_playlist_path()?;
                parser::parse_m3u_file(&path).ok()
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_m3u_file_provider_load() {
        let dir = tempfile::tempdir().unwrap();
        let m3u_path = dir.path().join("test.m3u");
        std::fs::write(&m3u_path, "#EXTM3U\n#EXTINF:-1,Channel1\nhttp://stream1.tv\n").unwrap();

        let provider = Provider::M3u {
            path: Some(m3u_path),
            url: None,
        };

        let playlist = provider.load().unwrap();
        assert_eq!(playlist.channels.len(), 1);
        assert_eq!(&*playlist.channels[0].name, "Channel1");
    }

    #[test]
    fn test_m3u_no_path_no_url_errors() {
        let provider = Provider::M3u { path: None, url: None };
        assert!(provider.load().is_err());
    }

    #[test]
    fn test_m3u_file_provider_missing_file() {
        let provider = Provider::M3u {
            path: Some(PathBuf::from("/nonexistent/playlist.m3u")),
            url: None,
        };
        assert!(provider.load().is_err());
    }

    #[test]
    fn test_m3u_file_provider_empty_playlist() {
        let dir = tempfile::tempdir().unwrap();
        let m3u_path = dir.path().join("empty.m3u");
        std::fs::write(&m3u_path, "#EXTM3U\n").unwrap();

        let provider = Provider::M3u { path: Some(m3u_path), url: None };
        let playlist = provider.load().unwrap();
        assert!(playlist.channels.is_empty());
    }

    #[test]
    fn test_file_provider_no_cache() {
        // File-based provider should return None for load_cached
        let provider = Provider::M3u { path: Some(PathBuf::from("/tmp/test.m3u")), url: None };
        assert!(provider.load_cached().is_none());
    }
}
