//! Трейт для бэкендов init-систем

use anyhow::Result;
use async_trait::async_trait;

use crate::config::service::ServiceConfig;

/// Трейт для всех бэкендов
#[async_trait]
pub trait InitBackend {
    /// Название бэкенда
    fn name(&self) -> &'static str;
    
    /// Проверка доступности бэкенда
    async fn is_available() -> bool where Self: Sized;
    
    /// Запуск сервиса
    async fn start(&mut self, name: &str, config: &ServiceConfig) -> Result<()>;
    
    /// Остановка сервиса
    async fn stop(&mut self, name: &str, config: &ServiceConfig) -> Result<()>;
    
    /// Перезапуск сервиса
    async fn restart(&mut self, name: &str, config: &ServiceConfig) -> Result<()>;
    
    /// Перезагрузка конфигурации сервиса
    async fn reload(&mut self, name: &str, config: &ServiceConfig) -> Result<()>;
    
    /// Получение статуса сервиса
    async fn status(&self, name: &str) -> Result<ServiceStatus>;
    
    /// Включение сервиса (автозагрузка)
    async fn enable(&mut self, name: &str, config: &ServiceConfig) -> Result<()>;
    
    /// Отключение сервиса
    async fn disable(&mut self, name: &str) -> Result<()>;
    
    /// Получение списка всех сервисов
    async fn list_services(&self) -> Result<Vec<String>>;
}

/// Статус сервиса
#[derive(Debug, Clone)]
pub struct ServiceStatus {
    pub name: String,
    pub state: ServiceState,
    pub sub_state: Option<String>,
    pub pid: Option<u32>,
    pub uptime_secs: Option<u64>,
    pub exit_code: Option<i32>,
}

/// Состояние сервиса
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceState {
    Running,
    Stopped,
    Starting,
    Stopping,
    Failed,
    Unknown,
}

impl std::fmt::Display for ServiceState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceState::Running => write!(f, "running"),
            ServiceState::Stopped => write!(f, "stopped"),
            ServiceState::Starting => write!(f, "starting"),
            ServiceState::Stopping => write!(f, "stopping"),
            ServiceState::Failed => write!(f, "failed"),
            ServiceState::Unknown => write!(f, "unknown"),
        }
    }
}
