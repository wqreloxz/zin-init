//! Основной движок ZIN init-системы

use anyhow::{Context, Result};
use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, timeout};

use crate::backends::backend::InitBackend;
use crate::config::manager::ConfigManager;
use crate::config::service::ServiceConfig;
use crate::hardware::detector::HardwareDetector;
use crate::hardware::types::SystemType;
use crate::rollback::manager::RollbackManager;
use crate::core::state::{ServiceState, StateManager};
use crate::core::service::Service;

/// Основной движок инициализации
pub struct ZinEngine {
    config_manager: Arc<ConfigManager>,
    backend: Arc<RwLock<Box<dyn InitBackend + Send + Sync>>>,
    hardware_detector: HardwareDetector,
    rollback_manager: RollbackManager,
    state_manager: StateManager,
    services: HashMap<String, Service>,
    system_type: SystemType,
}

impl ZinEngine {
    /// Создание нового движка
    pub async fn new() -> Result<Self> {
        info!("Инициализация ZIN Engine...");
        
        let config_manager = Arc::new(ConfigManager::new()?);
        let hardware_detector = HardwareDetector::new();
        let system_type = hardware_detector.detect_system_type().await?;
        
        info!("Определён тип системы: {:?}", system_type);
        
        // Выбор бэкенда на основе доступности и конфигурации
        let backend = Self::select_backend(&system_type).await?;
        
        let rollback_manager = RollbackManager::new(
            config_manager.get_config_path().join("rollback")
        )?;
        
        let state_manager = StateManager::new()?;
        
        Ok(Self {
            config_manager,
            backend: Arc::new(RwLock::new(backend)),
            hardware_detector,
            rollback_manager,
            state_manager,
            services: HashMap::new(),
            system_type,
        })
    }
    
    /// Выбор подходящего бэкенда
    async fn select_backend(system_type: &SystemType) -> Result<Box<dyn InitBackend + Send + Sync>> {
        use crate::backends::systemd::SystemdBackend;
        use crate::backends::openrc::OpenRcBackend;
        use crate::backends::sysv::SysVBackend;
        
        // Приоритет: systemd > OpenRC > SysV
        if SystemdBackend::is_available().await {
            info!("Используется бэкенд systemd");
            Ok(Box::new(SystemdBackend::new().await?))
        } else if OpenRcBackend::is_available().await {
            info!("Используется бэкенд OpenRC");
            Ok(Box::new(OpenRcBackend::new()?))
        } else {
            info!("Используется бэкенд SysV");
            Ok(Box::new(SysVBackend::new()?))
        }
    }
    
    /// Загрузка конфигурации сервисов
    pub async fn load_services(&mut self) -> Result<()> {
        info!("Загрузка конфигурации сервисов...");
        
        let configs = self.config_manager.load_all_services().await?;
        
        for config in configs {
            let service = Service::new(config.clone());
            self.services.insert(config.name.clone(), service);
        }
        
        info!("Загружено {} сервисов", self.services.len());
        Ok(())
    }
    
    /// Оптимизация загрузки на основе типа системы
    pub async fn optimize_for_system(&mut self) -> Result<()> {
        info!("Оптимизация для типа системы: {:?}", self.system_type);
        
        match self.system_type {
            SystemType::Workstation => {
                self.optimize_workstation().await?;
            }
            SystemType::Server => {
                self.optimize_server().await?;
            }
            SystemType::Embedded => {
                self.optimize_embedded().await?;
            }
        }
        
        Ok(())
    }
    
    /// Оптимизация для рабочей станции
    async fn optimize_workstation(&mut self) -> Result<()> {
        info!("Оптимизация для workstation: приоритет DE и приложений");
        
        // Приоритеты для workstation:
        // 1. Дисплей-менеджер
        // 2. Графическая среда
        // 3. Пользовательские сервисы
        // 4. Фоновые сервисы
        
        let priority_order = vec![
            "display-manager",
            "desktop-environment",
            "audio",
            "network",
            "bluetooth",
            "printing",
        ];
        
        self.apply_priority_order(&priority_order).await?;
        
        Ok(())
    }
    
    /// Оптимизация для сервера
    async fn optimize_server(&mut self) -> Result<()> {
        info!("Оптимизация для server: приоритет критических сервисов");
        
        // Приоритеты для server:
        // 1. Сеть
        // 2. Файловые системы
        // 3. Базы данных
        // 4. Веб-сервисы
        // 5. Мониторинг
        
        let priority_order = vec![
            "network",
            "filesystem",
            "ssh",
            "database",
            "webserver",
            "monitoring",
            "backup",
        ];
        
        self.apply_priority_order(&priority_order).await?;
        
        Ok(())
    }
    
    /// Оптимизация для embedded-систем
    async fn optimize_embedded(&mut self) -> Result<()> {
        info!("Оптимизация для embedded: минимальная загрузка");
        
        // Минимальный набор сервисов
        let priority_order = vec![
            "network",
            "essential",
        ];
        
        self.apply_priority_order(&priority_order).await?;
        
        Ok(())
    }
    
    /// Применение порядка приоритетов
    async fn apply_priority_order(&mut self, priority_order: &[&str]) -> Result<()> {
        let mut priority = 0u32;
        
        for category in priority_order {
            for (name, service) in self.services.iter_mut() {
                if service.config.category.as_deref() == Some(category) {
                    service.config.priority = priority;
                    priority += 1;
                }
            }
        }
        
        // Сохранение обновлённой конфигурации
        self.config_manager.save_binary_config(&self.services).await?;
        self.config_manager.save_toml_config(&self.services).await?;
        
        Ok(())
    }
    
    /// Запуск всех сервисов
    pub async fn start_all(&mut self) -> Result<()> {
        info!("Запуск всех сервисов...");
        
        // Сортировка по приоритету
        let mut services: Vec<_> = self.services.values_mut().collect();
        services.sort_by_key(|s| s.config.priority);
        
        for service in services {
            if let Err(e) = self.start_service(&service.config.name).await {
                error!("Ошибка запуска сервиса {}: {}", service.config.name, e);
                
                // Попытка отката
                if let Err(rollback_e) = self.rollback_service(&service.config.name).await {
                    error!("Ошибка отката сервиса {}: {}", service.config.name, rollback_e);
                }
            }
        }
        
        Ok(())
    }
    
    /// Запуск конкретного сервиса
    pub async fn start_service(&mut self, name: &str) -> Result<()> {
        info!("Запуск сервиса: {}", name);
        
        let service = self.services.get_mut(name)
            .with_context(|| format!("Сервис {} не найден", name))?;
        
        // Проверка состояния
        let current_state = self.state_manager.get_state(name);
        if current_state == ServiceState::Running {
            debug!("Сервис {} уже запущен", name);
            return Ok(());
        }
        
        // Проверка зависимостей
        if let Some(deps) = &service.config.depends_on {
            for dep in deps {
                let dep_state = self.state_manager.get_state(dep);
                if dep_state != ServiceState::Running {
                    warn!("Зависимость {} не запущена, запуск...", dep);
                    self.start_service(dep).await?;
                }
            }
        }
        
        // Создание точки отката перед запуском
        self.rollback_manager.create_checkpoint(name).await?;
        
        // Запуск через бэкенд
        let mut backend = self.backend.write().await;
        backend.start(name, &service.config).await?;
        
        // Ожидание запуска с таймаутом
        let timeout_duration = Duration::from_secs(service.config.timeout_secs);
        match timeout(timeout_duration, self.wait_for_state(name, ServiceState::Running)).await {
            Ok(Ok(())) => {
                self.state_manager.set_state(name, ServiceState::Running);
                info!("Сервис {} успешно запущен", name);
                Ok(())
            }
            Ok(Err(e)) => {
                error!("Сервис {} не смог запуститься: {}", name, e);
                self.state_manager.set_state(name, ServiceState::Failed);
                Err(e)
            }
            Err(_) => {
                error!("Таймаут запуска сервиса {}", name);
                self.state_manager.set_state(name, ServiceState::Failed);
                anyhow::bail!("Таймаут запуска сервиса {}", name);
            }
        }
    }
    
    /// Остановка сервиса
    pub async fn stop_service(&mut self, name: &str) -> Result<()> {
        info!("Остановка сервиса: {}", name);
        
        let service = self.services.get_mut(name)
            .with_context(|| format!("Сервис {} не найден", name))?;
        
        let mut backend = self.backend.write().await;
        backend.stop(name, &service.config).await?;
        
        self.state_manager.set_state(name, ServiceState::Stopped);
        info!("Сервис {} остановлен", name);
        
        Ok(())
    }
    
    /// Перезапуск сервиса
    pub async fn restart_service(&mut self, name: &str) -> Result<()> {
        info!("Перезапуск сервиса: {}", name);
        
        self.stop_service(name).await?;
        self.start_service(name).await?;
        
        Ok(())
    }
    
    /// Откат сервиса к последней рабочей версии
    async fn rollback_service(&mut self, name: &str) -> Result<()> {
        warn!("Выполнение отката для сервиса: {}", name);
        
        if let Some(checkpoint) = self.rollback_manager.get_latest_checkpoint(name).await? {
            info!("Восстановление из точки отката: {:?}", checkpoint.id);
            
            // Восстановление конфигурации
            self.config_manager.restore_from_checkpoint(&checkpoint).await?;
            
            // Перезапуск с восстановленной конфигурацией
            self.reload_service(name).await?;
            
            info!("Откат сервиса {} завершён успешно", name);
            Ok(())
        } else {
            warn!("Точки отката для сервиса {} не найдены", name);
            anyhow::bail!("Нет точек отката для сервиса {}", name);
        }
    }
    
    /// Перезагрузка конфигурации сервиса
    pub async fn reload_service(&mut self, name: &str) -> Result<()> {
        info!("Перезагрузка конфигурации сервиса: {}", name);
        
        let service = self.services.get_mut(name)
            .with_context(|| format!("Сервис {} не найден", name))?;
        
        // Перечитывание конфигурации
        let new_config = self.config_manager.load_service(name).await?;
        service.config = new_config;
        
        let mut backend = self.backend.write().await;
        backend.reload(name, &service.config).await?;
        
        info!("Конфигурация сервиса {} перезапущена", name);
        
        Ok(())
    }
    
    /// Ожидание достижения состояния
    async fn wait_for_state(&self, name: &str, target_state: ServiceState) -> Result<()> {
        loop {
            let state = self.state_manager.get_state(name);
            if state == target_state {
                return Ok(());
            }
            if state == ServiceState::Failed {
                anyhow::bail!("Сервис {} перешёл в состояние Failed", name);
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
    
    /// Получение статуса всех сервисов
    pub fn status(&self) -> HashMap<String, ServiceState> {
        self.services
            .keys()
            .map(|name| (name.clone(), self.state_manager.get_state(name)))
            .collect()
    }
    
    /// Получение типа системы
    pub fn system_type(&self) -> SystemType {
        self.system_type.clone()
    }
    
    /// Получение используемого бэкенда
    pub fn backend_name(&self) -> &str {
        self.backend.try_read()
            .map(|b| b.name())
            .unwrap_or("unknown")
    }
}

impl Drop for ZinEngine {
    fn drop(&mut self) {
        info!("ZIN Engine завершает работу");
    }
}
