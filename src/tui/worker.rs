use crate::tui::Message;
use anyhow::Context;
use cliscrape::FsmParser;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

#[derive(Debug, Clone)]
pub struct ParseRequest {
    pub template_path: PathBuf,
    pub input_path: PathBuf,
    pub block_idx: usize,
}

pub struct ParseWorker {
    tx: Option<mpsc::Sender<ParseRequest>>,
    join: Option<thread::JoinHandle<()>>,
}

impl ParseWorker {
    pub fn start(sender: mpsc::Sender<Message>) -> Self {
        let (tx, rx) = mpsc::channel::<ParseRequest>();

        let join = thread::spawn(move || {
            while let Ok(mut req) = rx.recv() {
                // Coalesce bursts: if multiple requests arrive quickly, only parse the latest.
                while let Ok(next) = rx.try_recv() {
                    req = next;
                }

                match parse_once(&req) {
                    Ok(report) => {
                        let _ = sender.send(Message::ParseDone(report));
                    }
                    Err(err) => {
                        let _ = sender.send(Message::ParseError(format!("{:#}", err)));
                    }
                }
            }
        });

        Self {
            tx: Some(tx),
            join: Some(join),
        }
    }

    pub fn request(&self, req: ParseRequest) {
        if let Some(tx) = &self.tx {
            let _ = tx.send(req);
        }
    }
}

impl Drop for ParseWorker {
    fn drop(&mut self) {
        // Closing the request channel ends the worker loop.
        drop(self.tx.take());

        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}

fn parse_once(req: &ParseRequest) -> anyhow::Result<cliscrape::DebugReport> {
    let parser = FsmParser::from_file(&req.template_path)
        .with_context(|| format!("Failed to load template from {:?}", req.template_path))?;

    let input_content = std::fs::read_to_string(&req.input_path)
        .with_context(|| format!("Failed to read input from {:?}", req.input_path))?;
    let blocks = crate::transcript::preprocess_ios_transcript(&input_content);

    let block = blocks
        .get(req.block_idx)
        .map(|s| s.as_str())
        .unwrap_or(&input_content);

    parser
        .debug_parse(block)
        .with_context(|| "Failed to debug-parse input")
}
