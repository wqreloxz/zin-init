//! Точка отката (checkpoint)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::service::ServiceConfig;

/// Точка отката
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    /// Уникальный идентификатор
    pub id: String,
    
    /// Время создания
    pub timestamp: DateTime<Utc>,
    
    /// Описание
    pub description: Option<String>,
    
    /// Снимок конфигураций сервисов
    pub services: std::collections::HashMap<String, ServiceConfig>,
    
    /// Хеш состояния
    pub state_hash: String,
    
    /// Тип чекпоинта
    pub checkpoint_type: CheckpointType,
    
    /// Срок хранения (дней)
    pub ttl_days: Option<u32>,
}

/// Тип точки отката
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CheckpointType {
    /// Автоматический (перед изменением)
    Auto,
    /// Ручной (по запросу пользователя)
    Manual,
    /// Перед обновлением
    PreUpdate,
    /// После успешной загрузки
    PostBoot,
    /// Аварийный (при ошибке)
    Emergency,
}

impl Checkpoint {
    /// Создание новой точки отката
    pub fn new(
        services: std::collections::HashMap<String, ServiceConfig>,
        checkpoint_type: CheckpointType,
    ) -> Self {
        use sha2::{Sha256, Digest};
        
        let id = Uuid::new_v4().to_string();
        let timestamp = Utc::now();
        
        // Вычисление хеша состояния
        let mut hasher = Sha256::new();
        for (name, config) in &services {
            hasher.update(name.as_bytes());
            if let Ok(bytes) = bincode::serialize(config) {
                hasher.update(&bytes);
            }
        }
        let state_hash = hex::encode(hasher.finalize());
        
        Self {
            id,
            timestamp,
            description: None,
            services,
            state_hash,
            checkpoint_type,
            ttl_days: Some(30), // По умолчанию 30 дней
        }
    }
    
    /// Установка описания
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
    
    /// Проверка истечения срока хранения
    pub fn is_expired(&self) -> bool {
        if let Some(ttl) = self.ttl_days {
            let age = Utc::now().signed_duration_since(self.timestamp)
                .num_days() as u32;
            age > ttl
        } else {
            false
        }
    }
    
    /// Получение возраста в днях
    pub fn age_days(&self) -> i64 {
        Utc::now().signed_duration_since(self.timestamp).num_days()
    }
}

/// Менеджер хранения чекпоинтов
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointStore {
    /// Путь к хранилищу
    pub path: String,
    
    /// Список чекпоинтов
    pub checkpoints: Vec<String>,
    
    /// Максимальное количество чекпоинтов
    pub max_checkpoints: usize,
}

impl CheckpointStore {
    pub fn new(path: String, max_checkpoints: usize) -> Self {
        Self {
            path,
            checkpoints: Vec::new(),
            max_checkpoints,
        }
    }
}
