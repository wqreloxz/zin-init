//! Бэкенд для systemd

use anyhow::{Context, Result};
use async_trait::async_trait;
use log::{debug, info, warn};
use std::path::Path;
use tokio::process::Command;

#[cfg(feature = "systemd-backend")]
use zbus::Connection;

use crate::backends::backend::{InitBackend, ServiceStatus, ServiceState};
use crate::config::service::ServiceConfig;

/// Бэкенд systemd
pub struct SystemdBackend {
    #[cfg(feature = "systemd-backend")]
    connection: Option<Connection>,
}

impl SystemdBackend {
    /// Создание нового бэкенда systemd
    pub async fn new() -> Result<Self> {
        #[cfg(feature = "systemd-backend")]
        let connection = Connection::system().await.ok();
        
        Ok(Self {
            #[cfg(feature = "systemd-backend")]
            connection,
        })
    }
    
    /// Проверка доступности systemd
    pub async fn is_available() -> bool {
        // Проверка наличия systemd
        Path::new("/run/systemd/system").exists()
    }
    
    /// Выполнение systemctl команды
    async fn systemctl(&self, args: &[&str]) -> Result<std::process::Output> {
        Command::new("systemctl")
            .args(args)
            .output()
            .await
            .context("Ошибка выполнения systemctl")
    }
}

#[async_trait]
impl InitBackend for SystemdBackend {
    fn name(&self) -> &'static str {
        "systemd"
    }
    
    async fn is_available() -> bool {
        Self::is_available()
    }
    
    async fn start(&mut self, name: &str, config: &ServiceConfig) -> Result<()> {
        info!("systemd: запуск сервиса {}", name);
        
        // Создание unit-файла если не существует
        self.create_unit_file(name, config).await?;
        
        // Перезагрузка daemon
        self.systemctl(&["daemon-reload"]).await?;
        
        // Запуск сервиса
        let output = self.systemctl(&["start", &format!("{}.service", name)]).await?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("systemctl start failed: {}", stderr);
        }
        
        Ok(())
    }
    
    async fn stop(&mut self, name: &str, _config: &ServiceConfig) -> Result<()> {
        info!("systemd: остановка сервиса {}", name);
        
        let output = self.systemctl(&["stop", &format!("{}.service", name)]).await?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("systemctl stop failed: {}", stderr);
        }
        
        Ok(())
    }
    
    async fn restart(&mut self, name: &str, config: &ServiceConfig) -> Result<()> {
        info!("systemd: перезапуск сервиса {}", name);
        
        let output = self.systemctl(&["restart", &format!("{}.service", name)]).await?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("systemctl restart failed: {}", stderr);
        }
        
        Ok(())
    }
    
    async fn reload(&mut self, name: &str, config: &ServiceConfig) -> Result<()> {
        info!("systemd: перезагрузка конфигурации сервиса {}", name);
        
        // Обновление unit-файла
        self.create_unit_file(name, config).await?;
        
        // Перезагрузка daemon
        self.systemctl(&["daemon-reload"]).await?;
        
        // Перезагрузка сервиса
        self.systemctl(&["reload", &format!("{}.service", name)]).await?;
        
        Ok(())
    }
    
    async fn status(&self, name: &str) -> Result<ServiceStatus> {
        let output = self.systemctl(&[
            "show",
            &format!("{}.service", name),
            "--property=ActiveState,SubState,MainPID,ExecMainStatus"
        ]).await?;
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        let mut active_state = "unknown";
        let mut sub_state = "unknown";
        let mut pid = None;
        let mut exit_code = None;
        
        for line in stdout.lines() {
            let parts: Vec<&str> = line.splitn(2, '=').collect();
            if parts.len() == 2 {
                match parts[0] {
                    "ActiveState" => active_state = parts[1],
                    "SubState" => sub_state = parts[1],
                    "MainPID" => pid = parts[1].parse().ok(),
                    "ExecMainStatus" => exit_code = parts[1].parse().ok(),
                    _ => {}
                }
            }
        }
        
        let state = match active_state {
            "active" => ServiceState::Running,
            "inactive" => ServiceState::Stopped,
            "activating" => ServiceState::Starting,
            "deactivating" => ServiceState::Stopping,
            "failed" => ServiceState::Failed,
            _ => ServiceState::Unknown,
        };
        
        Ok(ServiceStatus {
            name: name.to_string(),
            state,
            sub_state: Some(sub_state.to_string()),
            pid,
            uptime_secs: None,
            exit_code,
        })
    }
    
    async fn enable(&mut self, name: &str, config: &ServiceConfig) -> Result<()> {
        info!("systemd: включение сервиса {}", name);
        
        self.create_unit_file(name, config).await?;
        self.systemctl(&["daemon-reload"]).await?;
        
        let output = self.systemctl(&["enable", &format!("{}.service", name)]).await?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("systemctl enable warning: {}", stderr);
        }
        
        Ok(())
    }
    
    async fn disable(&mut self, name: &str) -> Result<()> {
        info!("systemd: отключение сервиса {}", name);
        
        let output = self.systemctl(&["disable", &format!("{}.service", name)]).await?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("systemctl disable warning: {}", stderr);
        }
        
        Ok(())
    }
    
    async fn list_services(&self) -> Result<Vec<String>> {
        let output = self.systemctl(&["list-units", "--type=service", "--all", "--no-legend"]).await?;
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        let services = stdout
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if let Some(name) = parts.first() {
                    Some(name.trim_end_matches(".service").to_string())
                } else {
                    None
                }
            })
            .collect();
        
        Ok(services)
    }
}

impl SystemdBackend {
    /// Создание unit-файла для сервиса
    async fn create_unit_file(&self, name: &str, config: &ServiceConfig) -> Result<()> {
        use std::io::Write;
        
        let unit_dir = Path::new("/etc/systemd/system");
        let unit_path = unit_dir.join(format!("{}.service", name));
        
        let mut content = String::new();
        
        // Unit секция
        content.push_str("[Unit]\n");
        content.push_str(&format!("Description={}\n", 
            config.description.as_deref().unwrap_or(&config.name)));
        
        if let Some(deps) = &config.depends_on {
            for dep in deps {
                content.push_str(&format!("After={}.service\n", dep));
                content.push_str(&format!("Requires={}.service\n", dep));
            }
        }
        
        // Service секция
        content.push_str("\n[Service]\n");
        content.push_str(&format!("Type={}\n", match config.service_type {
            crate::config::service::ServiceType::Simple => "simple",
            crate::config::service::ServiceType::Forking => "forking",
            crate::config::service::ServiceType::Notify => "notify",
            crate::config::service::ServiceType::Oneshot => "oneshot",
            crate::config::service::ServiceType::Dbus => "dbus",
            crate::config::service::ServiceType::Socket => "socket",
        }));
        
        content.push_str(&format!("ExecStart={}", config.exec));
        if !config.args.is_empty() {
            content.push(' ');
            content.push_str(&config.args.join(" "));
        }
        content.push('\n');
        
        if let Some(working_dir) = &config.working_dir {
            content.push_str(&format!("WorkingDirectory={}\n", working_dir));
        }
        
        if let Some(user) = &config.user {
            content.push_str(&format!("User={}\n", user));
        }
        
        if let Some(group) = &config.group {
            content.push_str(&format!("Group={}\n", group));
        }
        
        // Environment
        for (key, value) in &config.environment {
            content.push_str(&format!("Environment=\"{}={}\"\\n", key, value));
        }
        
        // Restart policy
        content.push_str(&format!("Restart={}\n", match config.restart_policy {
            crate::config::service::RestartPolicy::No => "no",
            crate::config::service::RestartPolicy::Always => "always",
            crate::config::service::RestartPolicy::OnFailure => "on-failure",
            crate::config::service::RestartPolicy::OnAbnormal => "on-abnormal",
        }));
        
        // Limits
        if let Some(limits) = &config.limits {
            if let Some(memory) = limits.memory_max {
                content.push_str(&format!("MemoryLimit={}\n", memory));
            }
            if let Some(nofile) = limits.nofile {
                content.push_str(&format!("LimitNOFILE={}\n", nofile));
            }
            if let Some(nproc) = limits.nproc {
                content.push_str(&format!("LimitNPROC={}\n", nproc));
            }
        }
        
        // Install секция
        content.push_str("\n[Install]\n");
        content.push_str("WantedBy=multi-user.target\n");
        
        // Запись файла
        let mut file = std::fs::File::create(&unit_path)
            .context("Ошибка создания unit-файла")?;
        file.write_all(content.as_bytes())
            .context("Ошибка записи unit-файла")?;
        
        debug!("Unit-файл создан: {:?}", unit_path);
        
        Ok(())
    }
}
