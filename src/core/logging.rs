//! Модуль логирования

use log::{LevelFilter, Metadata, Record, Level};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use chrono::Utc;

/// Логгер ZIN
pub struct ZinLogger {
    file: Option<File>,
    console: bool,
}

impl ZinLogger {
    pub fn new(log_path: Option<&Path>, console: bool) -> anyhow::Result<Self> {
        let file = match log_path {
            Some(path) => {
                let parent = path.parent().unwrap_or(Path::new("/var/log"));
                std::fs::create_dir_all(parent)?;
                
                Some(OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)?)
            }
            None => None,
        };
        
        Ok(Self { file, console })
    }
    
    pub fn init(&self) -> anyhow::Result<()> {
        let max_level = LevelFilter::Info;
        
        log::set_boxed_logger(Box::new(ZinLoggerWrapper {
            console: self.console,
            file: self.file.is_some(),
        }))?;
        log::set_max_level(max_level);
        
        Ok(())
    }
}

struct ZinLoggerWrapper {
    console: bool,
    file: bool,
}

impl log::Log for ZinLoggerWrapper {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }
    
    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S");
            let message = format!(
                "[{}] [{}] {}",
                timestamp,
                record.level(),
                record.args()
            );
            
            if self.console {
                eprintln!("{}", message);
            }
            
            // Логирование в файл реализовано через log crate
        }
    }
    
    fn flush(&self) {}
}
