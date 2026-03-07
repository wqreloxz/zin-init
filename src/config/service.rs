//! Конфигурация сервиса

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Конфигурация сервиса
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    /// Имя сервиса
    pub name: String,
    
    /// Описание сервиса
    pub description: Option<String>,
    
    /// Путь к исполняемому файлу
    pub exec: String,
    
    /// Аргументы командной строки
    #[serde(default)]
    pub args: Vec<String>,
    
    /// Рабочая директория
    pub working_dir: Option<String>,
    
    /// Переменные окружения
    #[serde(default)]
    pub environment: HashMap<String, String>,
    
    /// Зависимости (имена других сервисов)
    #[serde(default)]
    pub depends_on: Option<Vec<String>>,
    
    /// Тип сервиса
    #[serde(default)]
    pub service_type: ServiceType,
    
    /// Политика перезапуска
    #[serde(default)]
    pub restart_policy: RestartPolicy,
    
    /// Таймаут запуска (секунды)
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
    
    /// Приоритет загрузки (меньше = выше приоритет)
    #[serde(default)]
    pub priority: u32,
    
    /// Категория сервиса
    pub category: Option<String>,
    
    /// Включён ли сервис
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    
    /// Пользователь для запуска
    pub user: Option<String>,
    
    /// Группа для запуска
    pub group: Option<String>,
    
    /// Лимиты ресурсов
    pub limits: Option<ResourceLimits>,
    
    /// Условия запуска
    pub conditions: Option<Vec<LaunchCondition>>,
}

fn default_timeout() -> u64 {
    30
}

fn default_enabled() -> bool {
    true
}

/// Тип сервиса
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceType {
    /// Простой сервис (демон)
    Simple,
    /// Сервис с форком
    Forking,
    /// Сервис с уведомлением
    Notify,
    /// Одноразовый сервис
    Oneshot,
    /// DBus-сервис
    Dbus,
    /// Сокет-активируемый сервис
    Socket,
}

impl Default for ServiceType {
    fn default() -> Self {
        ServiceType::Simple
    }
}

/// Политика перезапуска
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RestartPolicy {
    /// Никогда не перезапускать
    No,
    /// Всегда перезапускать
    Always,
    /// Перезапускать при сбое
    OnFailure,
    /// Перезапускать при неожиданной остановке
    OnAbnormal,
}

impl Default for RestartPolicy {
    fn default() -> Self {
        RestartPolicy::OnFailure
    }
}

/// Лимиты ресурсов
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Максимум памяти (в байтах)
    pub memory_max: Option<u64>,
    /// Минимум памяти (в байтах)
    pub memory_min: Option<u64>,
    /// Лимит CPU (в процентах)
    pub cpu_limit: Option<u32>,
    /// Лимит открытых файловых дескрипторов
    pub nofile: Option<u64>,
    /// Лимит процессов
    pub nproc: Option<u64>,
}

/// Условия запуска
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LaunchCondition {
    /// Файл должен существовать
    FileExists(String),
    /// Файл не должен существовать
    FileNotExists(String),
    /// Переменная окружения должна быть установлена
    EnvironmentSet(String),
    /// Команда должна вернуть 0
    CommandSuccess(String),
    /// Система должна быть определённого типа
    SystemType(String),
}

impl ServiceConfig {
    /// Создание базовой конфигурации
    pub fn new(name: impl Into<String>, exec: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            exec: exec.into(),
            args: Vec::new(),
            working_dir: None,
            environment: HashMap::new(),
            depends_on: None,
            service_type: ServiceType::Simple,
            restart_policy: RestartPolicy::OnFailure,
            timeout_secs: default_timeout(),
            priority: 100,
            category: None,
            enabled: true,
            user: None,
            group: None,
            limits: None,
            conditions: None,
        }
    }
    
    /// Установка описания
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }
    
    /// Установка зависимостей
    pub fn with_dependencies(mut self, deps: Vec<String>) -> Self {
        self.depends_on = Some(deps);
        self
    }
    
    /// Установка приоритета
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }
    
    /// Установка категории
    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }
}
