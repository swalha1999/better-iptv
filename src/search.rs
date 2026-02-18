use crate::model::Playlist;
use nucleo::pattern::{CaseMatching, Normalization};
use nucleo::{Config, Nucleo};
use std::sync::{mpsc, Arc};
use std::thread;

const MAX_RESULTS: usize = 500;

pub struct SearchEngine {
    query_tx: mpsc::SyncSender<String>,
    result_rx: mpsc::Receiver<SearchResult>,
}

pub struct SearchResult {
    pub query: String,
    pub indices: Vec<usize>,
}

impl SearchEngine {
    pub fn new(playlist: &Playlist) -> Self {
        let channel_names: Vec<Arc<str>> = playlist.channels.iter().map(|c| c.name.clone()).collect();

        let (query_tx, query_rx) = mpsc::sync_channel::<String>(4);
        let (result_tx, result_rx) = mpsc::channel::<SearchResult>();

        thread::Builder::new()
            .name("search".into())
            .spawn(move || {
                let mut nucleo: Nucleo<u32> = Nucleo::new(Config::DEFAULT, Arc::new(|| {}), None, 1);

                {
                    let injector = nucleo.injector();
                    for (i, name) in channel_names.iter().enumerate() {
                        let name = name.clone();
                        injector.push(i as u32, move |_data, cols| {
                            cols[0] = name.as_ref().into();
                        });
                    }
                }

                // Tick until injection is complete
                loop {
                    let status = nucleo.tick(10);
                    if !status.running {
                        break;
                    }
                }

                let mut last_query = String::new();

                while let Ok(mut query) = query_rx.recv() {
                    // Drain to latest
                    while let Ok(newer) = query_rx.try_recv() {
                        query = newer;
                    }

                    if query == last_query {
                        continue;
                    }
                    last_query = query.clone();

                    if query.is_empty() {
                        let _ = result_tx.send(SearchResult {
                            query,
                            indices: vec![],
                        });
                        continue;
                    }

                    nucleo.pattern.reparse(
                        0,
                        &query,
                        CaseMatching::Ignore,
                        Normalization::Smart,
                        false,
                    );

                    loop {
                        let status = nucleo.tick(10);
                        if !status.running {
                            break;
                        }
                    }

                    let snapshot = nucleo.snapshot();
                    let matched = snapshot.matched_item_count() as usize;
                    let take = MAX_RESULTS.min(matched) as u32;

                    let indices: Vec<usize> = snapshot
                        .matched_items(..take)
                        .map(|item| *item.data as usize)
                        .collect();

                    let _ = result_tx.send(SearchResult { query, indices });
                }
            })
            .expect("failed to spawn search thread");

        Self { query_tx, result_rx }
    }

    pub fn send_query(&self, query: &str) {
        let _ = self.query_tx.try_send(query.to_string());
    }

    pub fn try_recv(&self) -> Option<SearchResult> {
        self.result_rx.try_recv().ok()
    }
}
