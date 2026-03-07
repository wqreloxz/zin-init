//! Бэкенд для OpenRC

use anyhow::{Context, Result};
use async_trait::async_trait;
use log::{debug, info, warn};
use std::path::Path;
use tokio::process::Command;
use std::io::Write;

use crate::backends::backend::{InitBackend, ServiceStatus, ServiceState};
use crate::config::service::ServiceConfig;

/// Бэкенд OpenRC
pub struct OpenRcBackend {
    init_dir: String,
    conf_dir: String,
}

impl OpenRcBackend {
    /// Создание нового бэкенда OpenRC
    pub fn new() -> Result<Self> {
        let init_dir = if Path::new("/etc/init.d").exists() {
            "/etc/init.d".to_string()
        } else {
            "/lib/rc/sh".to_string()
        };
        
        let conf_dir = "/etc/conf.d".to_string();
        
        Ok(Self {
            init_dir,
            conf_dir,
        })
    }
    
    /// Проверка доступности OpenRC
    pub async fn is_available() -> bool {
        Path::new("/etc/init.d").exists() && 
        (Path::new("/sbin/openrc-run").exists() || Path::new("/etc/init.d/rc").exists())
    }
    
    /// Выполнение rc-команды
    async fn rc_command(&self, service: &str, action: &str) -> Result<std::process::Output> {
        Command::new("/etc/init.d")
            .arg(service)
            .arg(action)
            .output()
            .await
            .context("Ошибка выполнения rc-команды")
    }
    
    /// Создание init-скрипта
    fn create_init_script(&self, name: &str, config: &ServiceConfig) -> Result<()> {
        let script_path = Path::new(&self.init_dir).join(name);
        
        let mut script = String::new();
        
        // Shebang
        script.push_str("#!/sbin/openrc-run\n");
        
        // Зависимости
        script.push_str("\n");
        if let Some(deps) = &config.depends_on {
            script.push_str("depend() {\n");
            for dep in deps {
                script.push_str(&format!("    need {}\n", dep));
            }
            script.push_str("}\n");
        }
        
        // Start функция
        script.push_str("\nstart() {\n");
        script.push_str("    ebegin \"Starting ");
        script.push_str(name);
        script.push_str("\"\n");
        
        script.push_str("    start-stop-daemon --start --background");
        
        if let Some(user) = &config.user {
            script.push_str(&format!(" --chuid {}", user));
        }
        
        if let Some(working_dir) = &config.working_dir {
            script.push_str(&format!(" --chdir {}", working_dir));
        }
        
        script.push_str(&format!(" --exec {}", config.exec));
        
        if !config.args.is_empty() {
            script.push_str(" --");
            for arg in &config.args {
                script.push(' ');
                script.push_str(arg);
            }
        }
        
        script.push_str("\n");
        script.push_str("    eend $?\n");
        script.push_str("}\n");
        
        // Stop функция
        script.push_str("\nstop() {\n");
        script.push_str("    ebegin \"Stopping ");
        script.push_str(name);
        script.push_str("\"\n");
        script.push_str(&format!("    start-stop-daemon --stop --exec {}", config.exec));
        script.push_str("\n");
        script.push_str("    eend $?\n");
        script.push_str("}\n");
        
        // Reload функция
        if config.service_type != crate::config::service::ServiceType::Oneshot {
            script.push_str("\nreload() {\n");
            script.push_str(&format!("    start-stop-daemon --signal HUP --exec {}\n", config.exec));
            script.push_str("}\n");
        }
        
        // Запись файла
        let mut file = std::fs::File::create(&script_path)
            .context("Ошибка создания init-скрипта")?;
        file.write_all(script.as_bytes())
            .context("Ошибка записи init-скрипта")?;
        
        // Установка прав на выполнение
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&script_path, std::fs::Permissions::from_mode(0o755))?;
        }
        
        debug!("Init-скрипт создан: {:?}", script_path);
        
        Ok(())
    }
    
    /// Создание конфига
    fn create_config(&self, name: &str, config: &ServiceConfig) -> Result<()> {
        let conf_path = Path::new(&self.conf_dir).join(name);
        
        let mut conf = String::new();
        
        // Переменные окружения
        if !config.environment.is_empty() {
            conf.push_str("# Environment variables\n");
            for (key, value) in &config.environment {
                conf.push_str(&format!("{}=\"{}\"\n", key, value));
            }
            conf.push_str("\n");
        }
        
        // Дополнительные опции
        conf.push_str("# Command arguments\n");
        if !config.args.is_empty() {
            conf.push_str(&format!("command_args=\"{}\"\n", config.args.join(" ")));
        }
        
        // Запись файла
        let mut file = std::fs::File::create(&conf_path)
            .context("Ошибка создания конфига")?;
        file.write_all(conf.as_bytes())
            .context("Ошибка записи конфига")?;
        
        debug!("Конфиг создан: {:?}", conf_path);
        
        Ok(())
    }
}

impl Default for OpenRcBackend {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

#[async_trait]
impl InitBackend for OpenRcBackend {
    fn name(&self) -> &'static str {
        "openrc"
    }
    
    async fn is_available() -> bool {
        Self::is_available()
    }
    
    async fn start(&mut self, name: &str, config: &ServiceConfig) -> Result<()> {
        info!("OpenRC: запуск сервиса {}", name);
        
        // Создание init-скрипта и конфига
        self.create_init_script(name, config)?;
        self.create_config(name, config)?;
        
        // Запуск сервиса
        let output = self.rc_command(name, "start").await?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("rc-service start failed: {}", stderr);
        }
        
        Ok(())
    }
    
    async fn stop(&mut self, name: &str, _config: &ServiceConfig) -> Result<()> {
        info!("OpenRC: остановка сервиса {}", name);
        
        let output = self.rc_command(name, "stop").await?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("rc-service stop failed: {}", stderr);
        }
        
        Ok(())
    }
    
    async fn restart(&mut self, name: &str, config: &ServiceConfig) -> Result<()> {
        info!("OpenRC: перезапуск сервиса {}", name);
        
        self.create_init_script(name, config)?;
        
        let output = self.rc_command(name, "restart").await?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("rc-service restart failed: {}", stderr);
        }
        
        Ok(())
    }
    
    async fn reload(&mut self, name: &str, config: &ServiceConfig) -> Result<()> {
        info!("OpenRC: перезагрузка конфигурации сервиса {}", name);
        
        self.create_init_script(name, config)?;
        self.create_config(name, config)?;
        
        let output = self.rc_command(name, "reload").await?;
        
        if !output.status.success() {
            warn!("rc-service reload warning: {:?}", output.status);
        }
        
        Ok(())
    }
    
    async fn status(&self, name: &str) -> Result<ServiceStatus> {
        let output = self.rc_command(name, "status").await;
        
        let (state, pid) = match output {
            Ok(out) => {
                if out.status.success() {
                    // Попытка извлечь PID из вывода
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    let pid = stdout
                        .lines()
                        .find_map(|line| {
                            if line.contains("pid") {
                                line.split_whitespace()
                                    .find(|w| w.chars().all(|c| c.is_numeric()))
                                    .and_then(|s| s.parse().ok())
                            } else {
                                None
                            }
                        });
                    (ServiceState::Running, pid)
                } else {
                    (ServiceState::Stopped, None)
                }
            }
            Err(_) => (ServiceState::Unknown, None),
        };
        
        Ok(ServiceStatus {
            name: name.to_string(),
            state,
            sub_state: None,
            pid,
            uptime_secs: None,
            exit_code: None,
        })
    }
    
    async fn enable(&mut self, name: &str, config: &ServiceConfig) -> Result<()> {
        info!("OpenRC: включение сервиса {}", name);
        
        self.create_init_script(name, config)?;
        self.create_config(name, config)?;
        
        // Добавление в default runlevel
        let output = Command::new("rc-update")
            .arg("add")
            .arg(name)
            .arg("default")
            .output()
            .await
            .context("Ошибка выполнения rc-update")?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("rc-update warning: {}", stderr);
        }
        
        Ok(())
    }
    
    async fn disable(&mut self, name: &str) -> Result<()> {
        info!("OpenRC: отключение сервиса {}", name);
        
        let output = Command::new("rc-update")
            .arg("del")
            .arg(name)
            .arg("default")
            .output()
            .await
            .context("Ошибка выполнения rc-update")?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("rc-update warning: {}", stderr);
        }
        
        Ok(())
    }
    
    async fn list_services(&self) -> Result<Vec<String>> {
        let output = Command::new("rc-status")
            .arg("--all")
            .output()
            .await
            .context("Ошибка выполнения rc-status")?;
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        let services = stdout
            .lines()
            .skip(1) // Пропуск заголовка
            .filter_map(|line| {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if let Some(name) = parts.first() {
                    if !name.is_empty() && !name.starts_with('*') {
                        return Some(name.to_string());
                    }
                }
                None
            })
            .collect();
        
        Ok(services)
    }
}
