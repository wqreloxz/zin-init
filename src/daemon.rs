//! ZIN Daemon - фоновый сервис управления

use anyhow::Result;
use log::{error, info, warn};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::signal;
use tokio::time::{Duration, interval};

use zin_core::core::engine::ZinEngine;
use zin_core::core::service::ServiceState;

/// Основной демон
pub struct ZinDaemon {
    engine: Arc<RwLock<ZinEngine>>,
    health_check_interval: Duration,
}

impl ZinDaemon {
    /// Создание нового демона
    pub async fn new() -> Result<Self> {
        info!("Инициализация ZIN Daemon...");
        
        let mut engine = ZinEngine::new().await?;
        engine.load_services().await?;
        engine.optimize_for_system().await?;
        
        Ok(Self {
            engine: Arc::new(RwLock::new(engine)),
            health_check_interval: Duration::from_secs(30),
        })
    }
    
    /// Запуск демона
    pub async fn run(&self) -> Result<()> {
        info!("ZIN Daemon запущен");
        
        // Запуск всех сервисов
        {
            let mut engine = self.engine.write().await;
            engine.start_all().await?;
        }
        
        // Создание задачи мониторинга
        let engine_clone = Arc::clone(&self.engine);
        let monitor_handle = tokio::spawn(async move {
            run_health_monitor(engine_clone).await;
        });
        
        // Ожидание сигналов завершения
        tokio::select! {
            _ = signal::ctrl_c() => {
                info!("Получен сигнал завершения");
            }
            _ = signal::unix::signal(signal::unix::SignalKind::terminate()) => {
                info!("Получен SIGTERM");
            }
            _ = signal::unix::signal(signal::unix::SignalKind::interrupt()) => {
                info!("Получен SIGINT");
            }
        }
        
        // Остановка мониторинга
        monitor_handle.abort();
        
        // Грациозная остановка сервисов
        info!("Остановка сервисов...");
        {
            let mut engine = self.engine.write().await;
            for (name, _) in engine.status() {
                if let Err(e) = engine.stop_service(&name).await {
                    error!("Ошибка остановки {}: {}", name, e);
                }
            }
        }
        
        info!("ZIN Daemon остановлен");
        
        Ok(())
    }
    
    /// Перезагрузка конфигурации
    pub async fn reload_config(&self) -> Result<()> {
        info!("Перезагрузка конфигурации...");
        
        let mut engine = self.engine.write().await;
        engine.load_services().await?;
        
        info!("Конфигурация перезапущена");
        
        Ok(())
    }
    
    /// Получение статуса
    pub async fn status(&self) -> std::collections::HashMap<String, ServiceState> {
        let engine = self.engine.read().await;
        engine.status()
    }
}

/// Монитор здоровья сервисов
async fn run_health_monitor(engine: Arc<RwLock<ZinEngine>>) {
    let mut interval = interval(Duration::from_secs(30));
    
    loop {
        interval.tick().await;
        
        let mut engine = engine.write().await;
        let status = engine.status();
        
        for (name, state) in &status {
            match state {
                ServiceState::Failed => {
                    warn!("Сервис {} в состоянии Failed, попытка восстановления...", name);
                    
                    // Попытка отката
                    if let Err(e) = engine.start_service(name).await {
                        error!("Не удалось восстановить {}: {}", name, e);
                    }
                }
                ServiceState::Stopped => {
                    // Проверка должен ли сервис быть запущен
                    if let Some(service) = engine.services.get(name) {
                        if service.config.enabled {
                            info!("Сервис {} остановлен, запуск...", name);
                            if let Err(e) = engine.start_service(name).await {
                                error!("Ошибка запуска {}: {}", name, e);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Инициализация логирования
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info")
    ).init();
    
    let daemon = ZinDaemon::new().await?;
    daemon.run().await?;
    
    Ok(())
}
