use std::collections::VecDeque;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Arc;

use chrono::Local;
use parking_lot::Mutex;
use tauri::{AppHandle, Emitter};

use crate::models::LogEntry;
use crate::paths::Paths;

const MAX_MEMORY_LOGS: usize = 800;

/// Logger simples: guarda em memoria (UI) e anexa em arquivo.
#[derive(Clone)]
pub struct Logger {
    buffer: Arc<Mutex<VecDeque<LogEntry>>>,
    file: Arc<Mutex<Option<std::fs::File>>>,
    app: Arc<Mutex<Option<AppHandle>>>,
}

impl Logger {
    pub fn new(paths: &Paths) -> Self {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(paths.log_file())
            .ok();
        Self {
            buffer: Arc::new(Mutex::new(VecDeque::with_capacity(MAX_MEMORY_LOGS))),
            file: Arc::new(Mutex::new(file)),
            app: Arc::new(Mutex::new(None)),
        }
    }

    pub fn attach(&self, app: AppHandle) {
        *self.app.lock() = Some(app);
    }

    fn push(&self, level: &str, message: impl Into<String>) {
        let entry = LogEntry {
            time: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            level: level.to_string(),
            message: message.into(),
        };

        {
            let mut buf = self.buffer.lock();
            if buf.len() >= MAX_MEMORY_LOGS {
                buf.pop_front();
            }
            buf.push_back(entry.clone());
        }

        if let Some(f) = self.file.lock().as_mut() {
            let _ = writeln!(f, "[{}] [{}] {}", entry.time, entry.level, entry.message);
            let _ = f.flush();
        }

        if let Some(app) = self.app.lock().as_ref() {
            let _ = app.emit("nr-log", entry);
        }
    }

    pub fn info(&self, m: impl Into<String>) {
        self.push("INFO", m);
    }
    pub fn warn(&self, m: impl Into<String>) {
        self.push("WARN", m);
    }
    pub fn error(&self, m: impl Into<String>) {
        self.push("ERROR", m);
    }

    pub fn snapshot(&self) -> Vec<LogEntry> {
        self.buffer.lock().iter().cloned().collect()
    }

    pub fn clear(&self) {
        self.buffer.lock().clear();
    }
}

// Mensagens de log curtas em portugues.
impl Logger {
    pub fn pt_info(&self, m: impl Into<String>) {
        self.info(m);
    }
}
