//! Типы систем

use serde::{Deserialize, Serialize};

/// Тип системы
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SystemType {
    /// Рабочая станция (Desktop/Workstation)
    Workstation,
    /// Сервер
    Server,
    /// Встраиваемая система
    Embedded,
}

impl std::fmt::Display for SystemType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SystemType::Workstation => write!(f, "workstation"),
            SystemType::Server => write!(f, "server"),
            SystemType::Embedded => write!(f, "embedded"),
        }
    }
}

/// Информация о системе
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    /// Тип системы
    pub system_type: SystemType,
    
    /// Количество ядер CPU
    pub cpu_cores: usize,
    
    /// Общий объём памяти (байты)
    pub total_memory: u64,
    
    /// Наличие графического окружения
    pub has_gui: bool,
    
    /// Наличие дисплей-менеджера
    pub has_display_manager: bool,
    
    /// Серверный флаг (наличие серверных сервисов)
    pub is_server_flag: bool,
    
    /// Embedded флаг (ограниченные ресурсы)
    pub is_embedded: bool,
    
    /// Архитектура
    pub architecture: String,
    
    /// Имя хоста
    pub hostname: String,
}

/// Конфигурация для типа системы
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemProfile {
    pub name: String,
    pub system_type: SystemType,
    pub optimized_services: Vec<String>,
    pub disabled_services: Vec<String>,
    pub boot_priority: BootPriority,
}

/// Приоритеты загрузки
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BootPriority {
    /// Быстрая загрузка DE и приложений
    FastDesktop,
    /// Быстрая загрузка сервисов
    FastServices,
    /// Минимальная загрузка
    Minimal,
    /// Полная загрузка
    Full,
}

impl SystemProfile {
    /// Профиль для рабочей станции
    pub fn workstation() -> Self {
        Self {
            name: "workstation".to_string(),
            system_type: SystemType::Workstation,
            optimized_services: vec![
                "display-manager".to_string(),
                "desktop-environment".to_string(),
                "audio".to_string(),
                "network".to_string(),
                "bluetooth".to_string(),
            ],
            disabled_services: vec![
                "mysql".to_string(),
                "postgresql".to_string(),
                "nginx".to_string(),
            ],
            boot_priority: BootPriority::FastDesktop,
        }
    }
    
    /// Профиль для сервера
    pub fn server() -> Self {
        Self {
            name: "server".to_string(),
            system_type: SystemType::Server,
            optimized_services: vec![
                "network".to_string(),
                "ssh".to_string(),
                "filesystem".to_string(),
                "database".to_string(),
                "webserver".to_string(),
            ],
            disabled_services: vec![
                "bluetooth".to_string(),
                "audio".to_string(),
                "cups".to_string(),
            ],
            boot_priority: BootPriority::FastServices,
        }
    }
    
    /// Профиль для embedded
    pub fn embedded() -> Self {
        Self {
            name: "embedded".to_string(),
            system_type: SystemType::Embedded,
            optimized_services: vec![
                "network".to_string(),
                "essential".to_string(),
            ],
            disabled_services: vec![
                "bluetooth".to_string(),
                "audio".to_string(),
                "printing".to_string(),
                "desktop-environment".to_string(),
            ],
            boot_priority: BootPriority::Minimal,
        }
    }
}
