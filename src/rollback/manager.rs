//! Менеджер отката

use anyhow::{Context, Result};
use log::{debug, info, warn};
use std::path::{Path, PathBuf};
use tokio::fs;
use walkdir::WalkDir;

use crate::config::format::{save_binary, load_binary};
use crate::rollback::checkpoint::{Checkpoint, CheckpointType};

/// Менеджер отката
pub struct RollbackManager {
    storage_path: PathBuf,
    max_checkpoints: usize,
}

impl RollbackManager {
    /// Создание нового менеджера отката
    pub fn new(storage_path: PathBuf) -> Result<Self> {
        Ok(Self {
            storage_path,
            max_checkpoints: 10, // Максимум 10 чекпоинтов на сервис
        })
    }
    
    /// Инициализация хранилища
    pub async fn init(&self) -> Result<()> {
        fs::create_dir_all(&self.storage_path).await?;
        debug!("Хранилище отката инициализировано: {:?}", self.storage_path);
        Ok(())
    }
    
    /// Создание точки отката для сервиса
    pub async fn create_checkpoint(&self, service_name: &str) -> Result<Checkpoint> {
        info!("Создание точки отката для сервиса: {}", service_name);
        
        // Создание директории для сервиса
        let service_dir = self.storage_path.join(service_name);
        fs::create_dir_all(&service_dir).await?;
        
        // Загрузка текущей конфигурации
        let config_path = self.get_config_path(service_name);
        let services = if config_path.exists() {
            match load_binary::<std::collections::HashMap<String, crate::config::service::ServiceConfig>>(
                &config_path
            ).await {
                Ok(services) => services,
                Err(_) => std::collections::HashMap::new(),
            }
        } else {
            std::collections::HashMap::new()
        };
        
        // Создание чекпоинта
        let checkpoint = Checkpoint::new(services, CheckpointType::Auto)
            .with_description(format!("Автоматический чекпоинт для {}", service_name));
        
        // Сохранение чекпоинта
        let checkpoint_path = service_dir.join(format!("{}.bin", checkpoint.id));
        save_binary(&checkpoint_path, &checkpoint).await?;
        
        // Очистка старых чекпоинтов
        self.cleanup_old_checkpoints(service_name).await?;
        
        info!("Точка отката создана: {}", checkpoint.id);
        
        Ok(checkpoint)
    }
    
    /// Создание ручной точки отката
    pub async fn create_manual_checkpoint(
        &self,
        service_name: &str,
        description: Option<String>,
    ) -> Result<Checkpoint> {
        info!("Создание ручной точки отката для сервиса: {}", service_name);
        
        let service_dir = self.storage_path.join(service_name);
        fs::create_dir_all(&service_dir).await?;
        
        // Загрузка текущей конфигурации
        let config_path = self.get_config_path(service_name);
        let services = if config_path.exists() {
            match load_binary::<std::collections::HashMap<String, crate::config::service::ServiceConfig>>(
                &config_path
            ).await {
                Ok(services) => services,
                Err(_) => std::collections::HashMap::new(),
            }
        } else {
            std::collections::HashMap::new()
        };
        
        // Создание чекпоинта
        let mut checkpoint = Checkpoint::new(services, CheckpointType::Manual);
        if let Some(desc) = description {
            checkpoint.description = Some(desc);
        }
        
        // Сохранение
        let checkpoint_path = service_dir.join(format!("{}.bin", checkpoint.id));
        save_binary(&checkpoint_path, &checkpoint).await?;
        
        info!("Ручная точка отката создана: {}", checkpoint.id);
        
        Ok(checkpoint)
    }
    
    /// Получение последней точки отката
    pub async fn get_latest_checkpoint(&self, service_name: &str) -> Result<Option<Checkpoint>> {
        let service_dir = self.storage_path.join(service_name);
        
        if !service_dir.exists() {
            return Ok(None);
        }
        
        let mut checkpoints = Vec::new();
        
        for entry in WalkDir::new(&service_dir)
            .max_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("bin") {
                match load_binary::<Checkpoint>(path).await {
                    Ok(cp) => checkpoints.push(cp),
                    Err(e) => warn!("Ошибка загрузки чекпоинта {:?}: {}", path, e),
                }
            }
        }
        
        // Сортировка по времени (новые первые)
        checkpoints.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        
        // Возврат последнего не истёкшего чекпоинта
        for cp in &checkpoints {
            if !cp.is_expired() {
                return Ok(Some(cp.clone()));
            }
        }
        
        Ok(checkpoints.into_iter().next())
    }
    
    /// Получение всех точек отката
    pub async fn list_checkpoints(&self, service_name: Option<&str>) -> Result<Vec<Checkpoint>> {
        let mut checkpoints = Vec::new();
        
        let search_dirs = if let Some(name) = service_name {
            vec![self.storage_path.join(name)]
        } else {
            // Все директории сервисов
            let mut dirs = Vec::new();
            if self.storage_path.exists() {
                for entry in fs::read_dir(&self.storage_path).await? {
                    let entry = entry?;
                    if entry.path().is_dir() {
                        dirs.push(entry.path());
                    }
                }
            }
            dirs
        };
        
        for dir in search_dirs {
            if !dir.exists() {
                continue;
            }
            
            for entry in WalkDir::new(&dir)
                .max_depth(1)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("bin") {
                    match load_binary::<Checkpoint>(path).await {
                        Ok(cp) => {
                            if !cp.is_expired() {
                                checkpoints.push(cp);
                            }
                        }
                        Err(e) => warn!("Ошибка загрузки чекпоинта {:?}: {}", path, e),
                    }
                }
            }
        }
        
        // Сортировка по времени
        checkpoints.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        
        Ok(checkpoints)
    }
    
    /// Восстановление из точки отката
    pub async fn restore_checkpoint(&self, checkpoint: &Checkpoint) -> Result<()> {
        info!("Восстановление из точки отката: {}", checkpoint.id);
        
        // Сохранение конфигураций из чекпоинта
        for (name, config) in &checkpoint.services {
            let service_dir = self.storage_path.join(name);
            fs::create_dir_all(&service_dir).await?;
            
            let config_path = self.get_config_path(name);
            
            let mut services = std::collections::HashMap::new();
            services.insert(name.clone(), config.clone());
            
            save_binary(&config_path, &services).await?;
        }
        
        info!("Восстановление завершено");
        
        Ok(())
    }
    
    /// Удаление точки отката
    pub async fn delete_checkpoint(&self, service_name: &str, checkpoint_id: &str) -> Result<()> {
        let checkpoint_path = self.storage_path
            .join(service_name)
            .join(format!("{}.bin", checkpoint_id));
        
        if checkpoint_path.exists() {
            fs::remove_file(&checkpoint_path).await?;
            info!("Точка отката удалена: {}", checkpoint_id);
        }
        
        Ok(())
    }
    
    /// Очистка старых чекпоинтов
    async fn cleanup_old_checkpoints(&self, service_name: &str) -> Result<()> {
        let service_dir = self.storage_path.join(service_name);
        
        if !service_dir.exists() {
            return Ok(());
        }
        
        let mut checkpoints = Vec::new();
        
        for entry in WalkDir::new(&service_dir)
            .max_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("bin") {
                if let Ok(cp) = load_binary::<Checkpoint>(path).await {
                    checkpoints.push((cp, path.to_path_buf()));
                }
            }
        }
        
        // Сортировка по времени (новые первые)
        checkpoints.sort_by(|a, b| b.0.timestamp.cmp(&a.0.timestamp));
        
        // Удаление старых чекпоинтов
        for (i, (cp, path)) in checkpoints.iter().enumerate() {
            if i >= self.max_checkpoints || cp.is_expired() {
                fs::remove_file(path).await?;
                debug!("Удалён старый чекпоинт: {}", cp.id);
            }
        }
        
        Ok(())
    }
    
    /// Получение пути к конфигурации сервиса
    fn get_config_path(&self, service_name: &str) -> PathBuf {
        self.storage_path
            .join(service_name)
            .join("config.bin")
    }
    
    /// Откат к конкретной версии
    pub async fn rollback_to(&self, service_name: &str, checkpoint_id: &str) -> Result<Checkpoint> {
        let checkpoint_path = self.storage_path
            .join(service_name)
            .join(format!("{}.bin", checkpoint_id));
        
        if !checkpoint_path.exists() {
            anyhow::bail!("Точка отката не найдена: {}", checkpoint_id);
        }
        
        let checkpoint = load_binary::<Checkpoint>(&checkpoint_path).await?;
        
        // Восстановление
        self.restore_checkpoint(&checkpoint).await?;
        
        info!("Откат завершён: {}", checkpoint_id);
        
        Ok(checkpoint)
    }
    
    /// Создание чекпоинта перед обновлением
    pub async fn create_pre_update_checkpoint(
        &self,
        services: std::collections::HashMap<String, crate::config::service::ServiceConfig>,
    ) -> Result<Checkpoint> {
        info!("Создание точки отката перед обновлением");
        
        let checkpoint = Checkpoint::new(services, CheckpointType::PreUpdate)
            .with_description("Перед обновлением конфигурации");
        
        // Сохранение в общую директорию
        let update_dir = self.storage_path.join("_updates");
        fs::create_dir_all(&update_dir).await?;
        
        let checkpoint_path = update_dir.join(format!("{}.bin", checkpoint.id));
        save_binary(&checkpoint_path, &checkpoint).await?;
        
        info!("Pre-update чекпоинт создан: {}", checkpoint.id);
        
        Ok(checkpoint)
    }
}
