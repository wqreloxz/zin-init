//! ZIN - Легковесная декларативная init-система для Linux
//! 
//! # Особенности
//! - Декларативная конфигурация в бинарном и TOML форматах
//! - Поддержка различных бэкендов (systemd, OpenRC, SysV)
//! - Аппаратная ориентация (workstation/server)
//! - Механизм отката при сбоях
//! - Соблюдение философии UNIX

pub mod core;
pub mod config;
pub mod backends;
pub mod hardware;
pub mod rollback;

pub use core::engine::ZinEngine;
pub use config::manager::ConfigManager;
pub use backends::backend::InitBackend;
pub use hardware::detector::HardwareDetector;
pub use rollback::manager::RollbackManager;
