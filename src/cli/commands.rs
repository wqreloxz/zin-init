//! CLI команды

use clap::{Parser, Subcommand};

/// ZIN - легковесная декларативная init-система
#[derive(Parser, Debug)]
#[command(name = "zin")]
#[command(author = "ZIN Project Contributors")]
#[command(version = "0.1.0")]
#[command(about = "Легковесная декларативная init-система для Linux", long_about = None)]
pub struct Cli {
    /// Уровень логирования (trace, debug, info, warn, error)
    #[arg(short, long, default_value = "info")]
    pub log_level: String,
    
    /// Путь к конфигурации
    #[arg(short, long)]
    pub config: Option<String>,
    
    /// Сухой запуск (без реальных изменений)
    #[arg(long)]
    pub dry_run: bool,
    
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Запуск init-системы
    Start {
        /// Имя сервиса (все если не указано)
        service: Option<String>,
        
        /// Не ждать завершения запуска
        #[arg(long)]
        no_wait: bool,
    },
    
    /// Остановка сервисов
    Stop {
        /// Имя сервиса
        service: String,
        
        /// Таймаут остановки (сек)
        #[arg(short, long, default_value = "30")]
        timeout: u64,
    },
    
    /// Перезапуск сервисов
    Restart {
        /// Имя сервиса
        service: String,
    },
    
    /// Перезагрузка конфигурации
    Reload {
        /// Имя сервиса (все если не указано)
        service: Option<String>,
    },
    
    /// Статус сервисов
    Status {
        /// Имя сервиса (все если не указано)
        service: Option<String>,
        
        /// Подробный вывод
        #[arg(short, long)]
        verbose: bool,
        
        /// JSON вывод
        #[arg(long)]
        json: bool,
    },
    
    /// Включение сервиса (автозагрузка)
    Enable {
        /// Имя сервиса
        service: String,
    },
    
    /// Отключение сервиса
    Disable {
        /// Имя сервиса
        service: String,
    },
    
    /// Список сервисов
    List {
        /// Только включённые
        #[arg(short, long)]
        enabled: bool,
        
        /// Только запущенные
        #[arg(short, long)]
        running: bool,
        
        /// Фильтр по категории
        #[arg(short, long)]
        category: Option<String>,
    },
    
    /// Управление конфигурацией
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    
    /// Управление точками отката
    Rollback {
        #[command(subcommand)]
        action: RollbackAction,
    },
    
    /// Информация о системе
    Info {
        /// JSON вывод
        #[arg(long)]
        json: bool,
    },
    
    /// Оптимизация под тип системы
    Optimize {
        /// Тип системы (workstation, server, embedded)
        #[arg(short, long)]
        system_type: Option<String>,
        
        /// Показать рекомендации без применения
        #[arg(long)]
        dry_run: bool,
    },
    
    /// Валидация конфигурации
    Validate {
        /// Путь к файлу конфигурации
        file: Option<String>,
    },
    
    /// Генерация конфигурации по умолчанию
    Init {
        /// Тип системы (workstation, server, embedded)
        #[arg(short, long)]
        system_type: Option<String>,
        
        /// Путь для инициализации
        #[arg(short, long)]
        path: Option<String>,
    },
    
    /// Логирование
    Logs {
        /// Имя сервиса
        service: Option<String>,
        
        /// Количество строк
        #[arg(short, long, default_value = "50")]
        lines: usize,
        
        /// Следить за логом
        #[arg(short, long)]
        follow: bool,
    },
    
    /// Демон (фоновый режим)
    Daemon {
        /// Запуск в foreground
        #[arg(long)]
        foreground: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum ConfigAction {
    /// Показать конфигурацию
    Show {
        /// Имя сервиса
        service: String,
        
        /// Формат вывода (toml, json, binary)
        #[arg(short, long, default_value = "toml")]
        format: String,
    },
    
    /// Редактировать конфигурацию
    Edit {
        /// Имя сервиса
        service: String,
    },
    
    /// Проверить конфигурацию
    Check {
        /// Путь к файлу
        file: String,
    },
    
    /// Экспорт конфигурации
    Export {
        /// Путь для экспорта
        output: String,
        
        /// Формат (toml, json)
        #[arg(short, long, default_value = "toml")]
        format: String,
    },
    
    /// Импорт конфигурации
    Import {
        /// Путь к файлу
        file: String,
        
        /// Перезаписать существующие
        #[arg(long)]
        overwrite: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum RollbackAction {
    /// Список точек отката
    List {
        /// Имя сервиса
        service: Option<String>,
    },
    
    /// Создать точку отката
    Create {
        /// Имя сервиса
        service: String,
        
        /// Описание
        #[arg(short, long)]
        description: Option<String>,
    },
    
    /// Восстановить из точки отката
    Restore {
        /// ID точки отката
        checkpoint_id: String,
        
        /// Имя сервиса
        service: String,
        
        /// Подтверждение
        #[arg(short, long)]
        yes: bool,
    },
    
    /// Удалить точку отката
    Delete {
        /// ID точки отката
        checkpoint_id: String,
        
        /// Имя сервиса
        service: String,
    },
    
    /// Показать информацию о точке отката
    Show {
        /// ID точки отката
        checkpoint_id: String,
    },
}
