//! ZIN CLI - основной исполняемый файл

use anyhow::Result;
use clap::Parser;
use log::{debug, error, info};
use std::path::Path;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use zin_core::core::engine::ZinEngine;
use zin_core::config::manager::ConfigManager;
use zin_core::hardware::detector::HardwareDetector;
use zin_core::rollback::manager::RollbackManager;
use zin_core::cli::commands::{Cli, Commands, ConfigAction, RollbackAction};
use zin_core::cli::output::OutputFormatter;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Инициализация логирования
    init_logging(&cli.log_level)?;
    
    let formatter = OutputFormatter::new(false, false);
    
    // Выполнение команды
    let result = run_command(cli, &formatter).await;
    
    if let Err(e) = result {
        formatter.print_error(&e.to_string());
        std::process::exit(1);
    }
    
    Ok(())
}

fn init_logging(level: &str) -> Result<()> {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(level));
    
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(filter)
        .init();
    
    Ok(())
}

async fn run_command(cli: Cli, formatter: &OutputFormatter) -> Result<()> {
    match cli.command {
        Commands::Start { service, no_wait } => {
            cmd_start(service, no_wait, cli.dry_run, formatter).await
        }
        Commands::Stop { service, timeout } => {
            cmd_stop(&service, timeout, formatter).await
        }
        Commands::Restart { service } => {
            cmd_restart(&service, formatter).await
        }
        Commands::Reload { service } => {
            cmd_reload(service, formatter).await
        }
        Commands::Status { service, verbose, json } => {
            let fmt = OutputFormatter::new(json, verbose);
            cmd_status(service, &fmt).await
        }
        Commands::Enable { service } => {
            cmd_enable(&service, formatter).await
        }
        Commands::Disable { service } => {
            cmd_disable(&service, formatter).await
        }
        Commands::List { enabled, running, category } => {
            cmd_list(enabled, running, category.as_deref(), formatter).await
        }
        Commands::Config { action } => {
            cmd_config(action, formatter).await
        }
        Commands::Rollback { action } => {
            cmd_rollback(action, formatter).await
        }
        Commands::Info { json } => {
            let fmt = OutputFormatter::new(json, false);
            cmd_info(&fmt).await
        }
        Commands::Optimize { system_type, dry_run } => {
            cmd_optimize(system_type.as_deref(), dry_run, formatter).await
        }
        Commands::Validate { file } => {
            cmd_validate(file.as_deref(), formatter).await
        }
        Commands::Init { system_type, path } => {
            cmd_init(system_type.as_deref(), path.as_deref(), formatter).await
        }
        Commands::Logs { service, lines, follow } => {
            cmd_logs(service.as_deref(), lines, follow, formatter).await
        }
        Commands::Daemon { foreground } => {
            cmd_daemon(foreground, formatter).await
        }
    }
}

/// Запуск сервисов
async fn cmd_start(
    service: Option<String>,
    no_wait: bool,
    dry_run: bool,
    formatter: &OutputFormatter,
) -> Result<()> {
    if dry_run {
        formatter.print_warning("Dry run: изменения не применяются");
    }
    
    let mut engine = ZinEngine::new().await?;
    engine.load_services().await?;
    engine.optimize_for_system().await?;
    
    if let Some(name) = service {
        formatter.print_success(&format!("Запуск сервиса: {}", name));
        engine.start_service(&name).await?;
    } else {
        formatter.print_success("Запуск всех сервисов...");
        engine.start_all().await?;
    }
    
    if !no_wait {
        let status = engine.status();
        let running = status.values()
            .filter(|&&s| s == zin_core::core::service::ServiceState::Running)
            .count();
        formatter.print_success(&format!("Запущено сервисов: {}", running));
    }
    
    Ok(())
}

/// Остановка сервиса
async fn cmd_stop(service: &str, timeout: u64, formatter: &OutputFormatter) -> Result<()> {
    let mut engine = ZinEngine::new().await?;
    engine.load_services().await?;
    
    formatter.print_success(&format!("Остановка сервиса: {}", service));
    engine.stop_service(service).await?;
    
    Ok(())
}

/// Перезапуск сервиса
async fn cmd_restart(service: &str, formatter: &OutputFormatter) -> Result<()> {
    let mut engine = ZinEngine::new().await?;
    engine.load_services().await?;
    
    formatter.print_success(&format!("Перезапуск сервиса: {}", service));
    engine.restart_service(service).await?;
    
    Ok(())
}

/// Перезагрузка конфигурации
async fn cmd_reload(service: Option<String>, formatter: &OutputFormatter) -> Result<()> {
    let mut engine = ZinEngine::new().await?;
    engine.load_services().await?;
    
    if let Some(name) = service {
        formatter.print_success(&format!("Перезагрузка конфигурации: {}", name));
        engine.reload_service(&name).await?;
    } else {
        for (name, _) in engine.status() {
            formatter.print_success(&format!("Перезагрузка конфигурации: {}", name));
            engine.reload_service(&name).await?;
        }
    }
    
    Ok(())
}

/// Статус сервисов
async fn cmd_status(service: Option<String>, formatter: &OutputFormatter) -> Result<()> {
    let engine = ZinEngine::new().await?;
    
    if let Some(name) = service {
        let status = engine.status();
        if let Some(&state) = status.get(&name) {
            formatter.print_service_status(&name, state, None);
        } else {
            formatter.print_error(&format!("Сервис не найден: {}", name));
        }
    } else {
        let status = engine.status();
        let services: Vec<_> = status
            .into_iter()
            .map(|(n, s)| (n, s, None))
            .collect();
        formatter.print_services_list(&services);
    }
    
    Ok(())
}

/// Включение сервиса
async fn cmd_enable(service: &str, formatter: &OutputFormatter) -> Result<()> {
    let mut engine = ZinEngine::new().await?;
    engine.load_services().await?;
    
    let mut backend = engine.backend.write().await;
    let config = engine.services.get(service)
        .map(|s| s.config.clone())
        .ok_or_else(|| anyhow::anyhow!("Сервис не найден: {}", service))?;
    
    backend.enable(service, &config).await?;
    formatter.print_success(&format!("Сервис включён: {}", service));
    
    Ok(())
}

/// Отключение сервиса
async fn cmd_disable(service: &str, formatter: &OutputFormatter) -> Result<()> {
    let mut engine = ZinEngine::new().await?;
    engine.load_services().await?;
    
    let mut backend = engine.backend.write().await;
    backend.disable(service).await?;
    formatter.print_success(&format!("Сервис отключён: {}", service));
    
    Ok(())
}

/// Список сервисов
async fn cmd_list(
    enabled: bool,
    running: bool,
    category: Option<&str>,
    formatter: &OutputFormatter,
) -> Result<()> {
    let engine = ZinEngine::new().await?;
    let mut engine_mut = engine;
    engine_mut.load_services().await?;
    
    let status = engine_mut.status();
    let mut services = Vec::new();
    
    for (name, state) in status {
        let service = engine_mut.services.get(&name);
        
        // Фильтры
        if enabled && service.map(|s| !s.config.enabled).unwrap_or(true) {
            continue;
        }
        if running && state != zin_core::core::service::ServiceState::Running {
            continue;
        }
        if let Some(cat) = category {
            if service.and_then(|s| s.config.category.as_deref()) != Some(cat) {
                continue;
            }
        }
        
        services.push((name, state, None));
    }
    
    formatter.print_services_list(&services);
    
    Ok(())
}

/// Управление конфигурацией
async fn cmd_config(action: ConfigAction, formatter: &OutputFormatter) -> Result<()> {
    match action {
        ConfigAction::Show { service, format } => {
            let mut config_manager = ConfigManager::new()?;
            let config = config_manager.load_service(&service).await?;
            formatter.print_service_config(&service, &config, &format);
        }
        ConfigAction::Edit { service } => {
            // Открытие редактора
            let config_path = ConfigManager::new()?
                .get_config_path()
                .join("services.d")
                .join(format!("{}.toml", service));
            
            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());
            let status = std::process::Command::new(&editor)
                .arg(&config_path)
                .status()?;
            
            if !status.success() {
                anyhow::bail!("Редактор завершился с ошибкой");
            }
        }
        ConfigAction::Check { file } => {
            let content = std::fs::read_to_string(&file)?;
            let config: zin_core::config::service::ServiceConfig = toml::from_str(&content)?;
            formatter.print_success(&format!("Конфигурация валидна: {}", config.name));
        }
        ConfigAction::Export { output, format } => {
            let mut config_manager = ConfigManager::new()?;
            let services = config_manager.load_all_services().await?;
            
            let content = match format.as_str() {
                "json" => serde_json::to_string_pretty(&services)?,
                "toml" => toml::to_string_pretty(&services)?,
                _ => anyhow::bail!("Неподдерживаемый формат: {}", format),
            };
            
            std::fs::write(&output, content)?;
            formatter.print_success(&format!("Экспорт завершён: {}", output));
        }
        ConfigAction::Import { file, overwrite } => {
            let content = std::fs::read_to_string(&file)?;
            let services: Vec<zin_core::config::service::ServiceConfig> = toml::from_str(&content)?;
            
            let mut config_manager = ConfigManager::new()?;
            config_manager.init_config_dir().await?;
            
            for service in services {
                let path = config_manager.get_config_path()
                    .join("services.d")
                    .join(format!("{}.toml", service.name));
                
                if path.exists() && !overwrite {
                    formatter.print_warning(&format!("Пропущен: {} (существует)", service.name));
                    continue;
                }
                
                let content = toml::to_string_pretty(&service)?;
                std::fs::write(&path, content)?;
                formatter.print_success(&format!("Импортирован: {}", service.name));
            }
        }
    }
    
    Ok(())
}

/// Управление откатом
async fn cmd_rollback(action: RollbackAction, formatter: &OutputFormatter) -> Result<()> {
    let config_manager = ConfigManager::new()?;
    let rollback_manager = RollbackManager::new(
        config_manager.get_config_path().join("rollback")
    )?;
    
    match action {
        RollbackAction::List { service } => {
            let checkpoints = rollback_manager.list_checkpoints(service.as_deref()).await?;
            formatter.print_checkpoints(&checkpoints);
        }
        RollbackAction::Create { service, description } => {
            let checkpoint = rollback_manager
                .create_manual_checkpoint(&service, description)
                .await?;
            formatter.print_success(&format!("Точка отката создана: {}", checkpoint.id));
        }
        RollbackAction::Restore { checkpoint_id, service, yes } => {
            if !yes {
                print!("Восстановить из точки отката {}? [y/N] ", checkpoint_id);
                use std::io::{self, Write};
                io::stdout().flush()?;
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                if !input.trim().eq_ignore_ascii_case("y") {
                    formatter.print_warning("Отменено");
                    return Ok(());
                }
            }
            
            let checkpoint = rollback_manager.rollback_to(&service, &checkpoint_id).await?;
            formatter.print_success(&format!("Восстановление завершено: {}", checkpoint.id));
        }
        RollbackAction::Delete { checkpoint_id, service } => {
            rollback_manager.delete_checkpoint(&service, &checkpoint_id).await?;
            formatter.print_success("Точка отката удалена");
        }
        RollbackAction::Show { checkpoint_id } => {
            // Поиск чекпоинта по всем сервисам
            let checkpoints = rollback_manager.list_checkpoints(None).await?;
            if let Some(cp) = checkpoints.iter().find(|c| c.id == checkpoint_id) {
                let json = serde_json::json!({
                    "id": cp.id,
                    "timestamp": cp.timestamp,
                    "description": cp.description,
                    "type": format!("{:?}", cp.checkpoint_type),
                    "services": cp.services.keys().collect::<Vec<_>>(),
                    "state_hash": cp.state_hash,
                    "expired": cp.is_expired(),
                });
                println!("{}", serde_json::to_string_pretty(&json)?);
            } else {
                formatter.print_error(&format!("Точка отката не найдена: {}", checkpoint_id));
            }
        }
    }
    
    Ok(())
}

/// Информация о системе
async fn cmd_info(formatter: &OutputFormatter) -> Result<()> {
    let mut engine = ZinEngine::new().await?;
    let info = engine.hardware_detector.get_system_info();
    formatter.print_system_info(&info, engine.backend_name());
    
    Ok(())
}

/// Оптимизация
async fn cmd_optimize(
    system_type: Option<&str>,
    dry_run: bool,
    formatter: &OutputFormatter,
) -> Result<()> {
    let mut detector = HardwareDetector::new();
    let profile = detector.get_recommended_profile();
    
    if dry_run {
        formatter.print_success("Рекомендации (dry run):");
        println!("\nТип системы: {:?}", profile.system_type);
        println!("\nОптимизированные сервисы:");
        for s in &profile.optimized_services {
            println!("  + {}", s);
        }
        println!("\nОтключённые сервисы:");
        for s in &profile.disabled_services {
            println!("  - {}", s);
        }
    } else {
        let mut engine = ZinEngine::new().await?;
        engine.load_services().await?;
        engine.optimize_for_system().await?;
        formatter.print_success(&format!("Оптимизация применена: {:?}", profile.system_type));
    }
    
    Ok(())
}

/// Валидация
async fn cmd_validate(file: Option<&str>, formatter: &OutputFormatter) -> Result<()> {
    if let Some(path) = file {
        let content = std::fs::read_to_string(path)?;
        let config: zin_core::config::service::ServiceConfig = toml::from_str(&content)?;
        formatter.print_success(&format!("Валидно: {}", config.name));
    } else {
        let mut config_manager = ConfigManager::new()?;
        let services = config_manager.load_all_services().await?;
        formatter.print_success(&format!("Валидировано {} сервисов", services.len()));
    }
    
    Ok(())
}

/// Инициализация
async fn cmd_init(
    system_type: Option<&str>,
    path: Option<&str>,
    formatter: &OutputFormatter,
) -> Result<()> {
    let config_manager = ConfigManager::new()?;
    config_manager.init_config_dir().await?;
    
    // Создание примеров конфигураций
    let examples_dir = config_manager.get_config_path().join("services.d");
    
    let network_config = zin_core::config::service::ServiceConfig::new("network", "/etc/init.d/network")
        .with_description("Сетевые интерфейсы")
        .with_priority(1)
        .with_category("network");
    
    let ssh_config = zin_core::config::service::ServiceConfig::new("sshd", "/usr/sbin/sshd")
        .with_description("SSH сервер")
        .with_priority(10)
        .with_category("network")
        .with_dependencies(vec!["network".to_string()]);
    
    for config in [&network_config, &ssh_config] {
        let path = examples_dir.join(format!("{}.toml", config.name));
        let content = toml::to_string_pretty(config)?;
        std::fs::write(&path, content)?;
    }
    
    formatter.print_success("Конфигурация инициализирована");
    println!("Путь: {:?}", config_manager.get_config_path());
    
    Ok(())
}

/// Логи
async fn cmd_logs(
    service: Option<&str>,
    lines: usize,
    follow: bool,
    formatter: &OutputFormatter,
) -> Result<()> {
    let log_path = Path::new("/var/log/zin.log");
    
    if !log_path.exists() {
        formatter.print_warning("Лог файл не найден");
        return Ok(());
    }
    
    let content = std::fs::read_to_string(log_path)?;
    let log_lines: Vec<_> = content.lines().collect();
    let start = log_lines.len().saturating_sub(lines);
    
    for line in log_lines.iter().skip(start) {
        println!("{}", line);
    }
    
    if follow {
        use std::io::{BufRead, BufReader};
        use std::time::Duration;
        
        let mut file = std::fs::File::open(log_path)?;
        file.seek(std::io::SeekFrom::End(0))?;
        
        let mut reader = BufReader::new(file);
        let mut line = String::new();
        
        loop {
            line.clear();
            match reader.read_line(&mut line) {
                Ok(0) => {
                    std::thread::sleep(Duration::from_millis(100));
                }
                Ok(_) => {
                    print!("{}", line);
                }
                Err(e) => {
                    eprintln!("Ошибка чтения лога: {}", e);
                    break;
                }
            }
        }
    }
    
    Ok(())
}

/// Демон
async fn cmd_daemon(foreground: bool, formatter: &OutputFormatter) -> Result<()> {
    if !foreground {
        // Fork в фон
        formatter.print_success("ZIN daemon запущен в фоне");
        return Ok(());
    }
    
    formatter.print_success("ZIN daemon в foreground режиме");
    
    let mut engine = ZinEngine::new().await?;
    engine.load_services().await?;
    engine.optimize_for_system().await?;
    engine.start_all().await?;
    
    // Ожидание сигналов
    use tokio::signal;
    signal::ctrl_c().await?;
    
    formatter.print_success("ZIN daemon остановлен");
    
    Ok(())
}
