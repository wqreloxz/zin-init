//! Модуль бэкендов init-систем

pub mod backend;
#[cfg(feature = "systemd-backend")]
pub mod systemd;
pub mod openrc;
pub mod sysv;
