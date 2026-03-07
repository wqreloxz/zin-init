# ZIN - Легковесная декларативная init-система

**ZIN** (Zen Init) — современная, легковесная и декларативная init-система для Linux с поддержкой различных бэкендов (systemd, OpenRC, SysV), аппаратной ориентацией и встроенным механизмом отката.

## Особенности

-  **Легковесность** — написана на Rust, минимальное потребление ресурсов
-  **Декларативная конфигурация** — TOML и бинарный форматы с проверкой целостности
- **Мультибэкенд** — автоматический выбор между systemd, OpenRC, SysV
-  **Аппаратная ориентация** — оптимизация для workstation/server/embedded
- **Механизм отката** — автоматическое восстановление при сбоях
- **Философия UNIX** — модульность, простота, композируемость
-  **Универсальный установщик** — установка приложений, создание файлов и папок

## Быстрый старт

### Установка через установщик (рекомендуется)

```bash
# Клонирование репозитория
git clone https://github.com/zin-init/zin.git
cd zin

# Установка ZIN
sudo python3 installer.py install

# Установка приложения (например, nginx)
sudo python3 installer.py install-app nginx

# Создать директорию
sudo python3 installer.py create-dir /var/www/myapp

# Создать файл
sudo python3 installer.py create-file /var/www/myapp/config.toml --content "key=value"
```

### Ручная установка

```bash
# Сборка
cargo build --release

# Установка
sudo cp target/release/zin /usr/local/bin/
sudo cp target/release/zind /usr/local/bin/
```

### Инициализация

```bash
# Создание конфигурации
zin init

# Проверка информации о системе
zin info

# Оптимизация под ваш тип системы
zin optimize
```

### Управление сервисами

```bash
# Запуск всех сервисов
zin start

# Запуск конкретного сервиса
zin start nginx

# Статус сервисов
zin status

# Список сервисов
zin list

# Включение сервиса
zin enable sshd

# Перезапуск сервиса
zin restart nginx
```

## Конфигурация

### Формат конфигурации

Конфигурация сервисов хранится в `/etc/zin/services.d/` в формате TOML:

```toml
name = "nginx"
description = "Веб-сервер Nginx"
exec = "/usr/sbin/nginx"
args = ["-g", "daemon off;"]
working_dir = "/var/www"
priority = 50
category = "webserver"
enabled = true
timeout_secs = 30

# Зависимости
depends_on = ["network", "filesystem"]

# Тип сервиса
service_type = "Simple"

# Политика перезапуска
restart_policy = "OnFailure"

# Пользователь и группа
user = "www-data"
group = "www-data"

# Переменные окружения
[environment]
NGINX_HOST = "localhost"
NGINX_PORT = "80"

# Лимиты ресурсов
[limits]
memory_max = 536870912  # 512MB
nofile = 65536
nproc = 4096
```

### Бинарный формат

Для ускорения загрузки конфигурация также сохраняется в бинарном формате с:
- Магическими байтами `ZINCFG01`
- Контрольной суммой SHA-256
- Метками времени

## Типы систем

ZIN автоматически определяет тип системы и применяет оптимизации:

### Workstation
- Приоритет: Display Manager → Desktop Environment → Приложения
- Быстрая загрузка графического интерфейса

### Server
- Приоритет: Сеть → Файловые системы → Сервисы
- Оптимизация для серверных нагрузок

### Embedded
- Минимальный набор сервисов
- Экономия ресурсов

## Механизм отката

### Создание точки отката

```bash
# Автоматически (перед изменениями)
zin rollback create nginx --description "Перед обновлением"

# Просмотр точек отката
zin rollback list

# Восстановление
zin rollback restore <checkpoint-id> nginx
```

### Автоматический откат

При сбое запуска сервиса ZIN автоматически:
1. Создаёт точку отката перед запуском
2. Пытается запустить сервис
3. При неудаче — восстанавливает предыдущую конфигурацию

## CLI команды

| Команда | Описание |
|---------|----------|
| `zin start [service]` | Запуск сервисов |
| `zin stop <service>` | Остановка сервиса |
| `zin restart <service>` | Перезапуск сервиса |
| `zin reload [service]` | Перезагрузка конфигурации |
| `zin status [service]` | Статус сервисов |
| `zin list` | Список всех сервисов |
| `zin enable <service>` | Включение автозагрузки |
| `zin disable <service>` | Отключение автозагрузки |
| `zin info` | Информация о системе |
| `zin optimize` | Оптимизация под систему |
| `zin config show <service>` | Показать конфигурацию |
| `zin rollback list` | Точки отката |
| `zin logs [service]` | Просмотр логов |

## Установщик (installer.py)

| Команда | Описание |
|---------|----------|
| `install` | Установить ZIN |
| `install-app <name>` | Установить приложение |
| `remove-app <name>` | Удалить приложение |
| `create-dir <path>` | Создать директорию |
| `create-file <path>` | Создать файл |
| `status` | Показать статус |
| `list-apps` | Список приложений |
| `enable <service>` | Включить сервис |
| `disable <service>` | Отключить сервис |
| `restart <service>` | Перезапустить сервис |

## Документация

- **[INSTALLER_DOCS.md](INSTALLER_DOCS.md)** — Полная документация по установщику
- **[CLI_DOCS.md](CLI_DOCS.md)** — Все команды ZIN CLI
- **[examples/](examples/)** — Примеры конфигураций

## Архитектура

```
┌─────────────────────────────────────────────────────────┐
│                      ZIN CLI                            │
├─────────────────────────────────────────────────────────┤
│                   ZIN Engine                            │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐     │
│  │   Config    │  │  Hardware   │  │  Rollback   │     │
│  │   Manager   │  │   Detector  │  │   Manager   │     │
│  └─────────────┘  └─────────────┘  └─────────────┘     │
├─────────────────────────────────────────────────────────┤
│                    Backend Layer                        │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐     │
│  │  systemd    │  │   OpenRC    │  │    SysV     │     │
│  └─────────────┘  └─────────────┘  └─────────────┘     │
└─────────────────────────────────────────────────────────┘
```

## Примеры конфигураций

Смотрите директорию `examples/` для готовых конфигураций популярных сервисов.

## Сборка

```bash
# Debug сборка
cargo build

# Release сборка
cargo build --release

# Сборка с конкретным бэкендом
cargo build --release --no-default-features --features systemd-backend

# Тесты
cargo test

# Линтер
cargo clippy
```

## Требования

- Rust 1.70+
- Linux (любой дистрибутив)
- Один из: systemd, OpenRC, или SysV init

## Лицензия

MIT License — см. файл [LICENSE](LICENSE)

## Вклад в проект

Приветствуются PR и issue reports! Пожалуйста, прочитайте [CONTRIBUTING.md](CONTRIBUTING.md) перед началом работы.

## Связь

- GitHub Issues: для багов и предложений
- Discussions: для вопросов и обсуждений

---

**ZIN** — создаётся с любовью к UNIX-философии и Rust.
