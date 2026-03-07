//! Модуль вывода CLI

use anyhow::Result;
use std::collections::HashMap;

use crate::core::service::ServiceState;
use crate::hardware::types::{SystemInfo, SystemType};

/// Форматтер вывода
pub struct OutputFormatter {
    json: bool,
    verbose: bool,
}

impl OutputFormatter {
    pub fn new(json: bool, verbose: bool) -> Self {
        Self { json, verbose }
    }
    
    /// Вывод статуса сервиса
    pub fn print_service_status(&self, name: &str, state: ServiceState, pid: Option<u32>) {
        if self.json {
            let json = serde_json::json!({
                "name": name,
                "state": state.to_string(),
                "pid": pid,
            });
            println!("{}", json);
        } else {
            let state_str = match state {
                ServiceState::Running => "● running",
                ServiceState::Stopped => "○ stopped",
                ServiceState::Starting => "◍ starting",
                ServiceState::Stopping => "◍ stopping",
                ServiceState::Failed => "✗ failed",
                ServiceState::Restarting => "◍ restarting",
                ServiceState::NotFound => "? not-found",
            };
            
            let pid_str = pid.map(|p| format!(" (PID: {})", p)).unwrap_or_default();
            println!("{} {}{}", state_str, name, pid_str);
            
            if self.verbose {
                self.print_verbose_info(name, state);
            }
        }
    }
    
    /// Вывод списка сервисов
    pub fn print_services_list(&self, services: &[(String, ServiceState, Option<u32>)]) {
        if self.json {
            let list: Vec<_> = services.iter().map(|(n, s, p)| {
                serde_json::json!({
                    "name": n,
                    "state": s.to_string(),
                    "pid": p,
                })
            }).collect();
            println!("{}", serde_json::to_string_pretty(&list).unwrap());
        } else {
            println!("{:<30} {:<15} {}", "SERVICE", "STATE", "PID");
            println!("{:-<55}", "");
            
            for (name, state, pid) in services {
                let state_str = match state {
                    ServiceState::Running => "\x1b[32mrunning\x1b[0m",
                    ServiceState::Stopped => "\x1b[90mstopped\x1b[0m",
                    ServiceState::Failed => "\x1b[31mfailed\x1b[0m",
                    ServiceState::Starting => "\x1b[33mstarting\x1b[0m",
                    ServiceState::Stopping => "\x1b[33mstopping\x1b[0m",
                    _ => state.to_string().as_str(),
                };
                
                let pid_str = pid.map(|p| p.to_string()).unwrap_or_else(|| "-".to_string());
                println!("{:<30} {:<15} {}", name, state_str, pid_str);
            }
        }
    }
    
    /// Вывод информации о системе
    pub fn print_system_info(&self, info: &SystemInfo, backend: &str) {
        if self.json {
            let json = serde_json::json!({
                "system_type": info.system_type.to_string(),
                "cpu_cores": info.cpu_cores,
                "total_memory_bytes": info.total_memory,
                "total_memory_human": Self::format_bytes(info.total_memory),
                "has_gui": info.has_gui,
                "has_display_manager": info.has_display_manager,
                "architecture": info.architecture,
                "hostname": info.hostname,
                "backend": backend,
            });
            println!("{}", serde_json::to_string_pretty(&json).unwrap());
        } else {
            println!("\n\x1b[1mSystem Information\x1b[0m");
            println!("{:-<40}", "");
            println!("  Type:           {}", self.format_system_type(info.system_type));
            println!("  CPU Cores:      {}", info.cpu_cores);
            println!("  Memory:         {}", Self::format_bytes(info.total_memory));
            println!("  GUI:            {}", yes_no(info.has_gui));
            println!("  Display Mgr:    {}", yes_no(info.has_display_manager));
            println!("  Architecture:   {}", info.architecture);
            println!("  Hostname:       {}", info.hostname);
            println!("  Backend:        {}", backend);
            println!();
        }
    }
    
    /// Вывод списка точек отката
    pub fn print_checkpoints(&self, checkpoints: &[crate::rollback::checkpoint::Checkpoint]) {
        if self.json {
            let list: Vec<_> = checkpoints.iter().map(|cp| {
                serde_json::json!({
                    "id": cp.id,
                    "timestamp": cp.timestamp,
                    "description": cp.description,
                    "type": format!("{:?}", cp.checkpoint_type),
                    "services_count": cp.services.len(),
                    "expired": cp.is_expired(),
                })
            }).collect();
            println!("{}", serde_json::to_string_pretty(&list).unwrap());
        } else {
            if checkpoints.is_empty() {
                println!("Нет точек отката");
                return;
            }
            
            println!("{:<36} {:<20} {:<15} {}", "ID", "TIMESTAMP", "TYPE", "DESCRIPTION");
            println!("{:-<90}", "");
            
            for cp in checkpoints {
                let type_str = format!("{:?}", cp.checkpoint_type);
                let desc = cp.description.as_deref().unwrap_or("-");
                println!("{:<36} {:<20} {:<15} {}", cp.id, cp.timestamp.format("%Y-%m-%d %H:%M:%S"), type_str, desc);
            }
        }
    }
    
    /// Вывод конфигурации сервиса
    pub fn print_service_config(&self, name: &str, config: &crate::config::service::ServiceConfig, format: &str) {
        match format {
            "json" => {
                println!("{}", serde_json::to_string_pretty(&config).unwrap());
            }
            "toml" => {
                println!("{}", toml::to_string_pretty(&config).unwrap());
            }
            _ => {
                println!("\x1b[1mService: {}\x1b[0m", name);
                println!("{:-<40}", "");
                println!("  Description:    {}", config.description.as_deref().unwrap_or("-"));
                println!("  Exec:           {}", config.exec);
                println!("  Type:           {:?}", config.service_type);
                println!("  Restart Policy: {:?}", config.restart_policy);
                println!("  Enabled:        {}", yes_no(config.enabled));
                println!("  Priority:       {}", config.priority);
                println!("  Category:       {}", config.category.as_deref().unwrap_or("-"));
                
                if let Some(deps) = &config.depends_on {
                    println!("  Dependencies:   {}", deps.join(", "));
                }
                
                if !config.environment.is_empty() {
                    println!("  Environment:");
                    for (k, v) in &config.environment {
                        println!("    {}={}", k, v);
                    }
                }
            }
        }
    }
    
    /// Вывод ошибки
    pub fn print_error(&self, message: &str) {
        if self.json {
            let json = serde_json::json!({
                "error": message,
            });
            eprintln!("{}", serde_json::to_string(&json).unwrap());
        } else {
            eprintln!("\x1b[31mError:\x1b[0m {}", message);
        }
    }
    
    /// Вывод успеха
    pub fn print_success(&self, message: &str) {
        if self.json {
            let json = serde_json::json!({
                "success": true,
                "message": message,
            });
            println!("{}", serde_json::to_string(&json).unwrap());
        } else {
            println!("\x1b[32m✓\x1b[0m {}", message);
        }
    }
    
    /// Вывод предупреждения
    pub fn print_warning(&self, message: &str) {
        if self.json {
            let json = serde_json::json!({
                "warning": message,
            });
            eprintln!("{}", serde_json::to_string(&json).unwrap());
        } else {
            eprintln!("\x1b[33mWarning:\x1b[0m {}", message);
        }
    }
    
    fn print_verbose_info(&self, name: &str, state: ServiceState) {
        // Дополнительная информация для verbose режима
        println!("  Service: {}", name);
        println!("  State: {:?}", state);
    }
    
    fn format_system_type(&self, t: SystemType) -> String {
        match t {
            SystemType::Workstation => "Workstation \x1b[32m(DE optimized)\x1b[0m".to_string(),
            SystemType::Server => "Server \x1b[34m(services optimized)\x1b[0m".to_string(),
            SystemType::Embedded => "Embedded \x1b[33m(minimal)\x1b[0m".to_string(),
        }
    }
    
    fn format_bytes(bytes: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;
        const TB: u64 = GB * 1024;
        
        if bytes >= TB {
            format!("{:.2} TB", bytes as f64 / TB as f64)
        } else if bytes >= GB {
            format!("{:.2} GB", bytes as f64 / GB as f64)
        } else if bytes >= MB {
            format!("{:.2} MB", bytes as f64 / MB as f64)
        } else if bytes >= KB {
            format!("{:.2} KB", bytes as f64 / KB as f64)
        } else {
            format!("{} B", bytes)
        }
    }
}

fn yes_no(b: bool) -> &'static str {
    if b { "yes" } else { "no" }
}

/// Прогресс бар
pub struct ProgressBar {
    total: usize,
    current: usize,
    width: usize,
}

impl ProgressBar {
    pub fn new(total: usize, width: usize) -> Self {
        Self { total, current: 0, width }
    }
    
    pub fn update(&mut self, current: usize) {
        self.current = current;
        self.render();
    }
    
    pub fn increment(&mut self) {
        self.current += 1;
        self.render();
    }
    
    pub fn finish(&mut self) {
        self.current = self.total;
        self.render();
        println!();
    }
    
    fn render(&self) {
        let percent = (self.current as f64 / self.total as f64) * 100.0;
        let filled = ((self.current as f64 / self.total as f64) * self.width as f64) as usize;
        let empty = self.width - filled;
        
        let bar = "█".repeat(filled) + &"░".repeat(empty);
        
        print!("\r[{}] {:5.1}% ({}/{})", bar, percent, self.current, self.total);
        use std::io::Write;
        std::io::stderr().flush().ok();
    }
}
