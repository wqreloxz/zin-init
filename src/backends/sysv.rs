//! Бэкенд для SysV init

use anyhow::{Context, Result};
use async_trait::async_trait;
use log::{debug, info, warn};
use std::path::Path;
use tokio::process::Command;
use std::io::Write;

use crate::backends::backend::{InitBackend, ServiceStatus, ServiceState};
use crate::config::service::ServiceConfig;

/// Бэкенд SysV init
pub struct SysVBackend {
    init_dir: String,
    rc_dirs: Vec<String>,
}

impl SysVBackend {
    /// Создание нового бэкенда SysV
    pub fn new() -> Result<Self> {
        let init_dir = "/etc/init.d".to_string();
        
        let rc_dirs = vec![
            "/etc/rc0.d".to_string(),
            "/etc/rc1.d".to_string(),
            "/etc/rc2.d".to_string(),
            "/etc/rc3.d".to_string(),
            "/etc/rc4.d".to_string(),
            "/etc/rc5.d".to_string(),
            "/etc/rc6.d".to_string(),
        ];
        
        Ok(Self {
            init_dir,
            rc_dirs,
        })
    }
    
    /// Проверка доступности SysV init
    pub async fn is_available() -> bool {
        Path::new("/etc/init.d").exists() && 
        !Path::new("/run/systemd/system").exists() &&
        !Path::new("/etc/init.d/rc").exists().then(|| false).unwrap_or(true)
    }
    
    /// Создание init-скрипта
    fn create_init_script(&self, name: &str, config: &ServiceConfig) -> Result<()> {
        let script_path = Path::new(&self.init_dir).join(name);
        
        let mut script = String::new();
        
        // Shebang и заголовок
        script.push_str("#!/bin/sh\n");
        script.push_str("### BEGIN INIT INFO\n");
        script.push_str(&format!("# Provides:          {}\n", name));
        script.push_str(&format!("# Required-Start:    {}\n", 
            config.depends_on.as_ref()
                .map(|deps| deps.join(" "))
                .unwrap_or_else(|| "$remote_fs $syslog".to_string())));
        script.push_str("# Required-Stop:     $remote_fs $syslog\n");
        script.push_str("# Default-Start:     2 3 4 5\n");
        script.push_str("# Default-Stop:      0 1 6\n");
        script.push_str(&format!("# Short-Description: {}\n", 
            config.description.as_deref().unwrap_or(&config.name)));
        script.push_str("### END INIT INFO\n");
        script.push_str("\n");
        
        // Переменные
        script.push_str(&format!("DAEMON={}\n", config.exec));
        script.push_str(&format!("DAEMON_ARGS=\"{}\"\n", config.args.join(" ")));
        script.push_str(&format!("NAME={}\n", name));
        script.push_str("PIDFILE=/var/run/$NAME.pid\n");
        script.push_str("\n");
        
        // Загрузка функций
        script.push_str(". /lib/lsb/init-functions\n");
        script.push_str("\n");
        
        // do_start функция
        script.push_str("do_start() {\n");
        script.push_str("    start-stop-daemon --start --quiet --pidfile $PIDFILE --exec $DAEMON --test > /dev/null \\\n");
        script.push_str("        || return 1\n");
        script.push_str("    start-stop-daemon --start --quiet --pidfile $PIDFILE --exec $DAEMON -- \\\n");
        script.push_str("        $DAEMON_ARGS \\\n");
        script.push_str("        || return 2\n");
        script.push_str("}\n");
        script.push_str("\n");
        
        // do_stop функция
        script.push_str("do_stop() {\n");
        script.push_str("    start-stop-daemon --stop --quiet --retry=TERM/30 --pidfile $PIDFILE --name $NAME\n");
        script.push_str("    RETVAL=\"$?\"\n");
        script.push_str("    [ \"$RETVAL\" = 2 ] && return 2\n");
        script.push_str("    rm -f $PIDFILE\n");
        script.push_str("    return \"$RETVAL\"\n");
        script.push_str("}\n");
        script.push_str("\n");
        
        // case statement
        script.push_str("case \"$1\" in\n");
        script.push_str("  start)\n");
        script.push_str("    [ \"$VERBOSE\" != no ] && log_daemon_msg \"Starting $DESC\" \"$NAME\"\n");
        script.push_str("    do_start\n");
        script.push_str("    case \"$?\" in\n");
        script.push_str("        0|1) [ \"$VERBOSE\" != no ] && log_end_msg 0 ;;\n");
        script.push_str("        2) [ \"$VERBOSE\" != no ] && log_end_msg 1 ;;\n");
        script.push_str("    esac\n");
        script.push_str("    ;;\n");
        script.push_str("  stop)\n");
        script.push_str("    [ \"$VERBOSE\" != no ] && log_daemon_msg \"Stopping $DESC\" \"$NAME\"\n");
        script.push_str("    do_stop\n");
        script.push_str("    case \"$?\" in\n");
        script.push_str("        0|1) [ \"$VERBOSE\" != no ] && log_end_msg 0 ;;\n");
        script.push_str("        2) [ \"$VERBOSE\" != no ] && log_end_msg 1 ;;\n");
        script.push_str("    esac\n");
        script.push_str("    ;;\n");
        script.push_str("  restart|force-reload)\n");
        script.push_str("    log_daemon_msg \"Restarting $DESC\" \"$NAME\"\n");
        script.push_str("    do_stop\n");
        script.push_str("    case \"$?\" in\n");
        script.push_str("      0|1)\n");
        script.push_str("        do_start\n");
        script.push_str("        case \"$?\" in\n");
        script.push_str("            0) log_end_msg 0 ;;\n");
        script.push_str("            1) log_end_msg 1 ;;\n");
        script.push_str("            *) log_end_msg 1 ;;\n");
        script.push_str("        esac\n");
        script.push_str("        ;;\n");
        script.push_str("      *)\n");
        script.push_str("        log_end_msg 1\n");
        script.push_str("        ;;\n");
        script.push_str("    esac\n");
        script.push_str("    ;;\n");
        script.push_str("  status)\n");
        script.push_str("    status_of_proc \"$DAEMON\" \"$NAME\" && exit 0 || exit $?\n");
        script.push_str("    ;;\n");
        script.push_str("  *)\n");
        script.push_str("    echo \"Usage: /etc/init.d/$NAME {start|stop|restart|status}\"\n");
        script.push_str("    exit 1\n");
        script.push_str("    ;;\n");
        script.push_str("esac\n");
        script.push_str("\n");
        script.push_str("exit 0\n");
        
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
    
    /// Создание ссылок для runlevel
    fn create_runlevel_links(&self, name: &str, runlevels: &[u8]) -> Result<()> {
        for &runlevel in runlevels {
            let rc_dir = Path::new(&self.init_dir).parent()
                .unwrap_or(Path::new("/"))
                .join(format!("rc{}.d", runlevel));
            
            if !rc_dir.exists() {
                continue;
            }
            
            // S - start, K - kill
            let prefix = if runlevel <= 1 || runlevel == 6 { "K" } else { "S" };
            let link_name = rc_dir.join(format!("{}02{}", prefix, name));
            
            // Удаление старой ссылки если существует
            let _ = std::fs::remove_file(&link_name);
            
            // Создание символической ссылки
            let target = Path::new(&self.init_dir).join(name);
            #[cfg(unix)]
            {
                std::os::unix::fs::symlink(&target, &link_name)?;
            }
            
            debug!("Ссылка создана: {:?} -> {:?}", link_name, target);
        }
        
        Ok(())
    }
}

impl Default for SysVBackend {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

#[async_trait]
impl InitBackend for SysVBackend {
    fn name(&self) -> &'static str {
        "sysv"
    }
    
    async fn is_available() -> bool {
        Self::is_available()
    }
    
    async fn start(&mut self, name: &str, config: &ServiceConfig) -> Result<()> {
        info!("SysV: запуск сервиса {}", name);
        
        self.create_init_script(name, config)?;
        
        let output = Command::new(&format!("/{}/{}", self.init_dir, name))
            .arg("start")
            .output()
            .await
            .context("Ошибка запуска init-скрипта")?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("init.d start failed: {}", stderr);
        }
        
        Ok(())
    }
    
    async fn stop(&mut self, name: &str, _config: &ServiceConfig) -> Result<()> {
        info!("SysV: остановка сервиса {}", name);
        
        let output = Command::new(&format!("/{}/{}", self.init_dir, name))
            .arg("stop")
            .output()
            .await
            .context("Ошибка остановки init-скрипта")?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("init.d stop failed: {}", stderr);
        }
        
        Ok(())
    }
    
    async fn restart(&mut self, name: &str, config: &ServiceConfig) -> Result<()> {
        info!("SysV: перезапуск сервиса {}", name);
        
        self.create_init_script(name, config)?;
        
        let output = Command::new(&format!("/{}/{}", self.init_dir, name))
            .arg("restart")
            .output()
            .await
            .context("Ошибка перезапуска init-скрипта")?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("init.d restart failed: {}", stderr);
        }
        
        Ok(())
    }
    
    async fn reload(&mut self, name: &str, config: &ServiceConfig) -> Result<()> {
        info!("SysV: перезагрузка конфигурации сервиса {}", name);
        
        self.create_init_script(name, config)?;
        
        let output = Command::new(&format!("/{}/{}", self.init_dir, name))
            .arg("reload")
            .output()
            .await
            .context("Ошибка перезагрузки init-скрипта")?;
        
        if !output.status.success() {
            warn!("init.d reload warning: {:?}", output.status);
        }
        
        Ok(())
    }
    
    async fn status(&self, name: &str) -> Result<ServiceStatus> {
        let output = Command::new(&format!("/{}/{}", self.init_dir, name))
            .arg("status")
            .output()
            .await;
        
        let state = match output {
            Ok(out) => {
                if out.status.success() {
                    ServiceState::Running
                } else {
                    ServiceState::Stopped
                }
            }
            Err(_) => ServiceState::Unknown,
        };
        
        Ok(ServiceStatus {
            name: name.to_string(),
            state,
            sub_state: None,
            pid: None,
            uptime_secs: None,
            exit_code: None,
        })
    }
    
    async fn enable(&mut self, name: &str, config: &ServiceConfig) -> Result<()> {
        info!("SysV: включение сервиса {}", name);
        
        self.create_init_script(name, config)?;
        
        // Включение для runlevels 2, 3, 4, 5
        self.create_runlevel_links(name, &[2, 3, 4, 5])?;
        
        Ok(())
    }
    
    async fn disable(&mut self, name: &str) -> Result<()> {
        info!("SysV: отключение сервиса {}", name);
        
        // Удаление ссылок из всех runlevel
        for runlevel in 0..=6 {
            let rc_dir = Path::new(&self.init_dir).parent()
                .unwrap_or(Path::new("/"))
                .join(format!("rc{}.d", runlevel));
            
            if !rc_dir.exists() {
                continue;
            }
            
            // Поиск и удаление ссылок
            if let Ok(entries) = std::fs::read_dir(&rc_dir) {
                for entry in entries.flatten() {
                    let file_name = entry.file_name();
                    let name_str = file_name.to_string_lossy();
                    if name_str.ends_with(name) {
                        let _ = std::fs::remove_file(entry.path());
                    }
                }
            }
        }
        
        Ok(())
    }
    
    async fn list_services(&self) -> Result<Vec<String>> {
        let init_dir = Path::new(&self.init_dir);
        
        if !init_dir.exists() {
            return Ok(Vec::new());
        }
        
        let mut services = Vec::new();
        
        for entry in std::fs::read_dir(init_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() {
                if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                    services.push(name.to_string());
                }
            }
        }
        
        Ok(services)
    }
}
