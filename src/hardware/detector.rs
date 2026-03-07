//! Детектор типа системы и оборудования

use anyhow::Result;
use log::{debug, info};
use std::path::Path;
use sysinfo::{System, CpuRefreshKind, RefreshKind};

use crate::hardware::types::{SystemType, SystemInfo, SystemProfile};

/// Детектор оборудования и типа системы
pub struct HardwareDetector {
    system: System,
}

impl HardwareDetector {
    /// Создание нового детектора
    pub fn new() -> Self {
        let system = System::new_with_specific(
            RefreshKind::new()
                .with_cpu(CpuRefreshKind::everything())
                .with_memory()
        );
        
        Self { system }
    }
    
    /// Обновление информации о системе
    pub fn refresh(&mut self) {
        self.system.refresh_cpu_specific(CpuRefreshKind::everything());
        self.system.refresh_memory();
    }
    
    /// Определение типа системы
    pub async fn detect_system_type(&mut self) -> Result<SystemType> {
        self.refresh();
        
        let has_gui = self.detect_gui();
        let has_display_manager = self.detect_display_manager();
        let is_server = self.detect_server_indicators();
        let is_embedded = self.detect_embedded();
        
        debug!("GUI: {}, Display Manager: {}, Server: {}, Embedded: {}", 
               has_gui, has_display_manager, is_server, is_embedded);
        
        let system_type = if is_embedded {
            SystemType::Embedded
        } else if is_server && !has_gui {
            SystemType::Server
        } else if has_gui || has_display_manager {
            SystemType::Workstation
        } else {
            // По умолчанию считаем сервером если нет GUI
            SystemType::Server
        };
        
        info!("Определён тип системы: {:?}", system_type);
        
        Ok(system_type)
    }
    
    /// Получение полной информации о системе
    pub fn get_system_info(&mut self) -> SystemInfo {
        self.refresh();
        
        let system_type = match self.detect_system_type_blocking() {
            Ok(t) => t,
            Err(_) => SystemType::Server,
        };
        
        let total_memory = self.system.total_memory();
        let cpu_cores = self.system.cpus().len();
        let has_gui = self.detect_gui();
        let has_display_manager = self.detect_display_manager();
        let is_server_flag = self.detect_server_indicators();
        let is_embedded = self.detect_embedded();
        
        SystemInfo {
            system_type,
            cpu_cores,
            total_memory,
            has_gui,
            has_display_manager,
            is_server_flag,
            is_embedded,
            architecture: std::env::consts::ARCH.to_string(),
            hostname: hostname::get()
                .and_then(|h| h.into_string())
                .unwrap_or_else(|_| "unknown".to_string()),
        }
    }
    
    /// Блокирующая версия detect_system_type (для sync контекста)
    fn detect_system_type_blocking(&mut self) -> Result<SystemType> {
        let has_gui = self.detect_gui();
        let has_display_manager = self.detect_display_manager();
        let is_server = self.detect_server_indicators();
        let is_embedded = self.detect_embedded();
        
        if is_embedded {
            Ok(SystemType::Embedded)
        } else if is_server && !has_gui {
            Ok(SystemType::Server)
        } else if has_gui || has_display_manager {
            Ok(SystemType::Workstation)
        } else {
            Ok(SystemType::Server)
        }
    }
    
    /// Обнаружение графического окружения
    fn detect_gui(&self) -> bool {
        // Проверка переменных окружения
        if std::env::var("DISPLAY").is_ok() || std::env::var("WAYLAND_DISPLAY").is_ok() {
            return true;
        }
        
        // Проверка наличия типичных GUI библиотек
        let gui_libs = [
            "/usr/lib/libX11.so",
            "/usr/lib/libQt5Core.so",
            "/usr/lib/libgtk-3.so",
            "/usr/lib64/libX11.so",
            "/usr/lib64/libQt5Core.so",
            "/usr/lib64/libgtk-3.so",
        ];
        
        gui_libs.iter().any(|p| Path::new(p).exists())
    }
    
    /// Обнаружение дисплей-менеджера
    fn detect_display_manager(&self) -> bool {
        let display_managers = [
            "gdm", "gdm3",
            "sddm",
            "lightdm",
            "lxdm",
            "xdm",
            "greetd",
        ];
        
        for dm in &display_managers {
            // Проверка наличия сервиса
            let service_paths = [
                format!("/etc/systemd/system/display-manager.service"),
                format!("/lib/systemd/system/{}.service", dm),
                format!("/etc/init.d/{}", dm),
            ];
            
            for path in &service_paths {
                if Path::new(path).exists() {
                    return true;
                }
            }
        }
        
        false
    }
    
    /// Обнаружение серверных индикаторов
    fn detect_server_indicators(&self) -> bool {
        // Проверка наличия серверных сервисов
        let server_services = [
            "sshd",
            "nginx", "apache2", "httpd",
            "mysql", "mysqld", "mariadb",
            "postgresql", "postgres",
            "docker", "containerd",
            "kubelet",
            "redis", "memcached",
        ];
        
        for service in &server_services {
            let paths = [
                format!("/etc/systemd/system/{}.service", service),
                format!("/lib/systemd/system/{}.service", service),
                format!("/etc/init.d/{}", service),
            ];
            
            for path in &paths {
                if Path::new(path).exists() {
                    return true;
                }
            }
        }
        
        // Проверка серверного железа (ECC память, много ядер)
        let total_memory = self.system.total_memory();
        let cpu_cores = self.system.cpus().len();
        
        // Сервер обычно имеет > 16GB RAM или > 8 ядер
        if total_memory > 16 * 1024 * 1024 * 1024 || cpu_cores > 8 {
            return true;
        }
        
        false
    }
    
    /// Обнаружение embedded-системы
    fn detect_embedded(&self) -> bool {
        // Проверка ограниченных ресурсов
        let total_memory = self.system.total_memory();
        let cpu_cores = self.system.cpus().len();
        
        // Embedded обычно < 1GB RAM и <= 2 ядра
        if total_memory < 1024 * 1024 * 1024 && cpu_cores <= 2 {
            return true;
        }
        
        // Проверка архитектуры
        let arch = std::env::consts::ARCH;
        if arch == "arm" || arch == "aarch64" || arch == "mips" {
            // Дополнительные проверки для ARM
            if total_memory < 4 * 1024 * 1024 * 1024 {
                return true;
            }
        }
        
        // Проверка наличия embedded маркеров
        if Path::new("/proc/device-tree").exists() {
            return true;
        }
        
        false
    }
    
    /// Получение рекомендуемого профиля
    pub fn get_recommended_profile(&mut self) -> SystemProfile {
        let system_type = match self.detect_system_type_blocking() {
            Ok(t) => t,
            Err(_) => SystemType::Server,
        };
        
        match system_type {
            SystemType::Workstation => SystemProfile::workstation(),
            SystemType::Server => SystemProfile::server(),
            SystemType::Embedded => SystemProfile::embedded(),
        }
    }
}

impl Default for HardwareDetector {
    fn default() -> Self {
        Self::new()
    }
}
