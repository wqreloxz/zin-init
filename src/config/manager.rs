//! Менеджер конфигурации

use anyhow::{Context, Result};
use log::{debug, info, warn};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use walkdir::WalkDir;

use crate::config::format::{save_binary, load_binary, save_toml, load_toml};
use crate::config::service::ServiceConfig;
use crate::rollback::checkpoint::Checkpoint;

/// Менеджер конфигурации ZIN
pub struct ConfigManager {
    config_path: PathBuf,
    binary_path: PathBuf,
    toml_path: PathBuf,
    services_cache: HashMap<String, ServiceConfig>,
}

impl ConfigManager {
    /// Создание нового менеджера конфигурации
    pub fn new() -> Result<Self> {
        // Определение пути к конфигурации
        let config_path = if Path::new("/etc/zin").exists() {
            PathBuf::from("/etc/zin")
        } else if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".config/zin")
        } else {
            PathBuf::from("/etc/zin")
        };
        
        let binary_path = config_path.join("services.bin");
        let toml_path = config_path.join("services.toml");
        
        Ok(Self {
            config_path,
            binary_path,
            toml_path,
            services_cache: HashMap::new(),
        })
    }
    
    /// Получение пути к конфигурации
    pub fn get_config_path(&self) -> &Path {
        &self.config_path
    }
    
    /// Инициализация директории конфигурации
    pub async fn init_config_dir(&self) -> Result<()> {
        fs::create_dir_all(&self.config_path).await?;
        fs::create_dir_all(self.config_path.join("services.d")).await?;
        fs::create_dir_all(self.config_path.join("rollback")).await?;
        fs::create_dir_all(self.config_path.join("targets")).await?;
        
        info!("Директория конфигурации: {:?}", self.config_path);
        Ok(())
    }
    
    /// Загрузка всех сервисов
    pub async fn load_all_services(&mut self) -> Result<Vec<ServiceConfig>> {
        // Попытка загрузки из бинарного формата (быстрее)
        if self.binary_path.exists() {
            match load_binary::<Vec<ServiceConfig>>(&self.binary_path).await {
                Ok(configs) => {
                    debug!("Загружено {} сервисов из бинарного файла", configs.len());
                    for config in &configs {
                        self.services_cache.insert(config.name.clone(), config.clone());
                    }
                    return Ok(configs);
                }
                Err(e) => {
                    warn!("Ошибка загрузки бинарной конфигурации: {}. Пробуем TOML...", e);
                }
            }
        }
        
        // Попытка загрузки из TOML
        if self.toml_path.exists() {
            match load_toml::<Vec<ServiceConfig>>(&self.toml_path).await {
                Ok(configs) => {
                    debug!("Загружено {} сервисов из TOML файла", configs.len());
                    // Сохранение в бинарный формат для ускорения следующей загрузки
                    let _ = save_binary(&self.binary_path, &configs).await;
                    
                    for config in &configs {
                        self.services_cache.insert(config.name.clone(), config.clone());
                    }
                    return Ok(configs);
                }
                Err(e) => {
                    warn!("Ошибка загрузки TOML конфигурации: {}", e);
                }
            }
        }
        
        // Загрузка из отдельных файлов в services.d
        let services = self.load_services_from_dir().await?;
        
        if !services.is_empty() {
            // Сохранение в оба формата
            let _ = save_binary(&self.binary_path, &services).await;
            let _ = save_toml(&self.toml_path, &services).await;
        }
        
        Ok(services)
    }
    
    /// Загрузка сервисов из директории
    async fn load_services_from_dir(&self) -> Result<Vec<ServiceConfig>> {
        let mut services = Vec::new();
        let services_dir = self.config_path.join("services.d");
        
        if !services_dir.exists() {
            return Ok(services);
        }
        
        for entry in WalkDir::new(&services_dir)
            .max_depth(3)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                match load_toml::<ServiceConfig>(path).await {
                    Ok(config) => {
                        debug!("Загружен сервис: {}", config.name);
                        services.push(config);
                    }
                    Err(e) => {
                        warn!("Ошибка загрузки сервиса {:?}: {}", path, e);
                    }
                }
            }
        }
        
        Ok(services)
    }
    
    /// Загрузка конкретного сервиса
    pub async fn load_service(&mut self, name: &str) -> Result<ServiceConfig> {
        // Проверка кэша
        if let Some(config) = self.services_cache.get(name) {
            return Ok(config.clone());
        }
        
        // Поиск в services.d
        let service_file = self.config_path.join("services.d").join(format!("{}.toml", name));
        
        if service_file.exists() {
            let config = load_toml::<ServiceConfig>(&service_file).await?;
            self.services_cache.insert(name.to_string(), config.clone());
            return Ok(config);
        }
        
        anyhow::bail!("Сервис {} не найден", name);
    }
    
    /// Сохранение конфигурации в бинарном формате
    pub async fn save_binary_config(&self, services: &HashMap<String, crate::core::service::Service>) -> Result<()> {
        let configs: Vec<ServiceConfig> = services.values()
            .map(|s| s.config.clone())
            .collect();
        
        save_binary(&self.binary_path, &configs).await?;
        debug!("Конфигурация сохранена в {:?}", self.binary_path);
        
        Ok(())
    }
    
    /// Сохранение конфигурации в TOML формате
    pub async fn save_toml_config(&self, services: &HashMap<String, crate::core::service::Service>) -> Result<()> {
        let configs: Vec<ServiceConfig> = services.values()
            .map(|s| s.config.clone())
            .collect();
        
        save_toml(&self.toml_path, &configs).await?;
        debug!("Конфигурация сохранена в {:?}", self.toml_path);
        
        Ok(())
    }
    
    /// Сохранение точки отката
    pub async fn save_checkpoint(&self, checkpoint: &Checkpoint) -> Result<()> {
        let checkpoint_path = self.config_path
            .join("rollback")
            .join(format!("{}.bin", checkpoint.id));
        
        save_binary(&checkpoint_path, &checkpoint).await?;
        info!("Точка отката сохранена: {}", checkpoint.id);
        
        Ok(())
    }
    
    /// Загрузка точки отката
    pub async fn load_checkpoint(&self, id: &str) -> Result<Checkpoint> {
        let checkpoint_path = self.config_path
            .join("rollback")
            .join(format!("{}.bin", id));
        
        load_binary(&checkpoint_path).await
    }
    
    /// Восстановление из точки отката
    pub async fn restore_from_checkpoint(&mut self, checkpoint: &Checkpoint) -> Result<()> {
        info!("Восстановление из точки отката: {}", checkpoint.id);
        
        // Восстановление сервисов из чекпоинта
        for (name, config) in &checkpoint.services {
            self.services_cache.insert(name.clone(), config.clone());
        }
        
        // Пересохранение конфигурации
        let configs: Vec<ServiceConfig> = checkpoint.services.values().cloned().collect();
        save_binary(&self.binary_path, &configs).await?;
        save_toml(&self.toml_path, &configs).await?;
        
        Ok(())
    }
    
    /// Получение списка всех точек отката
    pub async fn list_checkpoints(&self) -> Result<Vec<Checkpoint>> {
        let mut checkpoints = Vec::new();
        let rollback_dir = self.config_path.join("rollback");
        
        if !rollback_dir.exists() {
            return Ok(checkpoints);
        }
        
        for entry in fs::read_dir(&rollback_dir).await? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("bin") {
                match load_binary::<Checkpoint>(&path).await {
                    Ok(checkpoint) => checkpoints.push(checkpoint),
                    Err(e) => warn!("Ошибка загрузки точки отката {:?}: {}", path, e),
                }
            }
        }
        
        // Сортировка по времени (новые первые)
        checkpoints.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        
        Ok(checkpoints)
    }
}
