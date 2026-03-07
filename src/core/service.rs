//! Модуль управления сервисами

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

use crate::config::service::ServiceConfig;

/// Сервис в системе
#[derive(Debug, Clone)]
pub struct Service {
    pub config: ServiceConfig,
    pub state: ServiceState,
    pub started_at: Option<DateTime<Utc>>,
    pub stopped_at: Option<DateTime<Utc>>,
    pub pid: Option<u32>,
    pub restart_count: u32,
    pub last_error: Option<String>,
}

impl Service {
    pub fn new(config: ServiceConfig) -> Self {
        Self {
            config,
            state: ServiceState::Stopped,
            started_at: None,
            stopped_at: None,
            pid: None,
            restart_count: 0,
            last_error: None,
        }
    }
    
    pub fn is_running(&self) -> bool {
        self.state == ServiceState::Running
    }
    
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }
}

/// Состояние сервиса
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceState {
    /// Сервис остановлен
    Stopped,
    /// Сервис запускается
    Starting,
    /// Сервис запущен
    Running,
    /// Сервис останавливается
    Stopping,
    /// Сервис перезапущается
    Restarting,
    /// Сервис не смог запуститься
    Failed,
    /// Сервис не найден
    NotFound,
}

impl std::fmt::Display for ServiceState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceState::Stopped => write!(f, "stopped"),
            ServiceState::Starting => write!(f, "starting"),
            ServiceState::Running => write!(f, "running"),
            ServiceState::Stopping => write!(f, "stopping"),
            ServiceState::Restarting => write!(f, "restarting"),
            ServiceState::Failed => write!(f, "failed"),
            ServiceState::NotFound => write!(f, "not-found"),
        }
    }
}

/// Менеджер состояний сервисов
#[derive(Debug, Default)]
pub struct StateManager {
    states: HashMap<String, ServiceState>,
}

impl StateManager {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self::default())
    }
    
    pub fn get_state(&self, name: &str) -> ServiceState {
        *self.states.get(name).unwrap_or(&ServiceState::NotFound)
    }
    
    pub fn set_state(&mut self, name: &str, state: ServiceState) {
        self.states.insert(name.to_string(), state);
    }
    
    pub fn list_states(&self) -> &HashMap<String, ServiceState> {
        &self.states
    }
    
    pub fn running_count(&self) -> usize {
        self.states.values()
            .filter(|&&s| s == ServiceState::Running)
            .count()
    }
    
    pub fn failed_count(&self) -> usize {
        self.states.values()
            .filter(|&&s| s == ServiceState::Failed)
            .count()
    }
}
