# ZIN CLI - Полная документация по командам

## Обзор

ZIN CLI предоставляет единый интерфейс для управления init-системой, сервисами и конфигурацией.

```bash
zin <команда> [опции] [аргументы]
```

## Глобальные опции

| Опция | Краткая | Описание | Пример |
|-------|---------|----------|--------|
| `--log-level` | `-l` | Уровень логирования | `zin -l debug start` |
| `--config` | `-c` | Путь к конфигурации | `zin -c /etc/zin status` |
| `--dry-run` | - | Сухой запуск (без изменений) | `zin --dry-run start` |
| `--help` | `-h` | Показать справку | `zin --help` |
| `--version` | `-v` | Показать версию | `zin --version` |

---

## Управление сервисами

### `zin start` - Запуск сервисов

Запускает один или все сервисы.

```bash
# Запустить все сервисы
zin start

# Запустить конкретный сервис
zin start nginx

# Запустить без ожидания
zin start --no-wait

# Запустить с отладочным логом
zin -l debug start nginx
```

**Опции:**
- `[service]` - Имя сервиса (опционально)
- `--no-wait` - Не ждать завершения запуска

---

### `zin stop` - Остановка сервиса

Останавливает указанный сервис.

```bash
# Остановить сервис
zin stop nginx

# Остановить с таймаутом
zin stop nginx --timeout 60
```

**Аргументы:**
- `<service>` - Имя сервиса (обязательно)

**Опции:**
- `--timeout`, `-t` - Таймаут остановки в секундах (по умолчанию: 30)

---

### `zin restart` - Перезапуск сервиса

Перезапускает указанный сервис.

```bash
# Перезапустить сервис
zin restart nginx

# Перезапустить с пересозданием
zin restart postgresql
```

**Аргументы:**
- `<service>` - Имя сервиса (обязательно)

---

### `zin reload` - Перезагрузка конфигурации

Перезагружает конфигурацию сервисов.

```bash
# Перезагрузить все сервисы
zin reload

# Перезагрузить конкретный сервис
zin reload nginx
```

**Аргументы:**
- `[service]` - Имя сервиса (опционально)

---

### `zin status` - Статус сервисов

Показывает статус сервисов.

```bash
# Статус всех сервисов
zin status

# Статус конкретного сервиса
zin status nginx

# Подробный статус
zin status --verbose

# JSON вывод
zin status --json
```

**Аргументы:**
- `[service]` - Имя сервиса (опционально)

**Опции:**
- `--verbose`, `-v` - Подробный вывод
- `--json` - JSON формат

**Пример вывода:**
```
SERVICE                        STATE           PID
-------------------------------------------------------
nginx                          running         1234
postgresql                     running         5678
redis                          stopped         -
docker                         failed          -
```

---

### `zin list` - Список сервисов

Показывает список всех сервисов с фильтрами.

```bash
# Все сервисы
zin list

# Только включённые
zin list --enabled

# Только запущенные
zin list --running

# Фильтр по категории
zin list --category network
```

**Опции:**
- `--enabled`, `-e` - Только включённые
- `--running`, `-r` - Только запущенные
- `--category`, `-c` - Фильтр по категории

---

### `zin enable` - Включение сервиса

Включает автозагрузку сервиса.

```bash
# Включить сервис
zin enable nginx

# Включить несколько сервисов
zin enable nginx postgresql redis
```

**Аргументы:**
- `<service>` - Имя сервиса (обязательно)

---

### `zin disable` - Отключение сервиса

Отключает автозагрузку сервиса.

```bash
# Отключить сервис
zin disable nginx
```

**Аргументы:**
- `<service>` - Имя сервиса (обязательно)

---

## Конфигурация

### `zin config show` - Показать конфигурацию

Показывает конфигурацию сервиса.

```bash
# Показать в TOML формате
zin config show nginx

# Показать в JSON формате
zin config show nginx --format json

# Показать в бинарном формате
zin config show nginx --format binary
```

**Аргументы:**
- `<service>` - Имя сервиса (обязательно)

**Опции:**
- `--format`, `-f` - Формат вывода (toml, json, binary)

---

### `zin config edit` - Редактировать конфигурацию

Открывает конфигурацию в редакторе.

```bash
# Редактировать сервис
zin config edit nginx

# С использованием конкретного редактора
EDITOR=vim zin config edit nginx
```

**Аргументы:**
- `<service>` - Имя сервиса (обязательно)

---

### `zin config check` - Проверить конфигурацию

Проверяет валидность конфигурации.

```bash
# Проверить файл
zin config check /etc/zin/services.d/nginx.toml
```

**Аргументы:**
- `<file>` - Путь к файлу (обязательно)

---

### `zin config export` - Экспорт конфигурации

Экспортирует конфигурацию в файл.

```bash
# Экспорт в TOML
zin config export /backup/config.toml

# Экспорт в JSON
zin config export /backup/config.json --format json
```

**Аргументы:**
- `<output>` - Путь для экспорта (обязательно)

**Опции:**
- `--format`, `-f` - Формат (toml, json)

---

### `zin config import` - Импорт конфигурации

Импортирует конфигурацию из файла.

```bash
# Импортировать конфигурацию
zin config import /backup/config.toml

# Импортировать с перезаписью
zin config import /backup/config.toml --overwrite
```

**Аргументы:**
- `<file>` - Путь к файлу (обязательно)

**Опции:**
- `--overwrite` - Перезаписать существующие

---

## Точки отката (Rollback)

### `zin rollback list` - Список точек отката

Показывает все точки отката.

```bash
# Все точки отката
zin rollback list

# Точки отката конкретного сервиса
zin rollback list nginx
```

**Аргументы:**
- `[service]` - Имя сервиса (опционально)

---

### `zin rollback create` - Создать точку отката

Создаёт новую точку отката.

```bash
# Создать точку отката
zin rollback create nginx

# Создать с описанием
zin rollback create nginx --description "Перед обновлением"
```

**Аргументы:**
- `<service>` - Имя сервиса (обязательно)

**Опции:**
- `--description`, `-d` - Описание точки отката

---

### `zin rollback restore` - Восстановить из точки отката

Восстанавливает конфигурацию из точки отката.

```bash
# Восстановить с подтверждением
zin rollback restore abc123 nginx

# Восстановить без подтверждения
zin rollback restore abc123 nginx --yes
```

**Аргументы:**
- `<checkpoint_id>` - ID точки отката (обязательно)
- `<service>` - Имя сервиса (обязательно)

**Опции:**
- `--yes`, `-y` - Автоматическое подтверждение

---

### `zin rollback delete` - Удалить точку отката

Удаляет точку отката.

```bash
# Удалить точку отката
zin rollback delete abc123 nginx
```

**Аргументы:**
- `<checkpoint_id>` - ID точки отката (обязательно)
- `<service>` - Имя сервиса (обязательно)

---

### `zin rollback show` - Показать точку отката

Показывает информацию о точке отката.

```bash
# Показать информацию
zin rollback show abc123
```

**Аргументы:**
- `<checkpoint_id>` - ID точки отката (обязательно)

---

## Система

### `zin info` - Информация о системе

Показывает информацию о системе и типе загрузки.

```bash
# Основная информация
zin info

# JSON вывод
zin info --json
```

**Опции:**
- `--json` - JSON формат

**Пример вывода:**
```
System Information
----------------------------------------
  Type:           Workstation (DE optimized)
  CPU Cores:      8
  Memory:         16.00 GB
  GUI:            yes
  Display Mgr:    yes
  Architecture:   x86_64
  Hostname:       myhost
  Backend:        systemd
```

---

### `zin optimize` - Оптимизация системы

Оптимизирует загрузку под тип системы.

```bash
# Автоматическая оптимизация
zin optimize

# Оптимизация для workstation
zin optimize --system-type workstation

# Показать рекомендации без применения
zin optimize --dry-run
```

**Опции:**
- `--system-type`, `-s` - Тип системы (workstation, server, embedded)
- `--dry-run` - Показать рекомендации без применения

---

### `zin validate` - Валидация конфигурации

Проверяет валидность конфигурации.

```bash
# Проверить все конфигурации
zin validate

# Проверить конкретный файл
zin validate /etc/zin/services.d/nginx.toml
```

**Аргументы:**
- `[file]` - Путь к файлу (опционально)

---

### `zin init` - Инициализация конфигурации

Создаёт начальную конфигурацию.

```bash
# Инициализировать с автоопределением
zin init

# Инициализировать для server
zin init --system-type server

# Инициализировать в другую директорию
zin init --path /custom/config
```

**Опции:**
- `--system-type`, `-s` - Тип системы
- `--path`, `-p` - Путь для инициализации

---

## Логи

### `zin logs` - Просмотр логов

Показывает логи сервисов.

```bash
# Логи всех сервисов
zin logs

# Логи конкретного сервиса
zin logs nginx

# Последние 100 строк
zin logs nginx --lines 100

# Следить за логом
zin logs nginx --follow
```

**Аргументы:**
- `[service]` - Имя сервиса (опционально)

**Опции:**
- `--lines`, `-n` - Количество строк (по умолчанию: 50)
- `--follow`, `-f` - Следить за логом

---

## Демон

### `zin daemon` - Запуск демона

Запускает ZIN в фоновом режиме.

```bash
# Запуск в фоне
zin daemon

# Запуск в foreground режиме
zin daemon --foreground
```

**Опции:**
- `--foreground` - Запуск в foreground режиме

---

## Сводная таблица команд

| Категория | Команда | Описание |
|-----------|---------|----------|
| **Запуск** | `start [service]` | Запуск сервисов |
| **Остановка** | `stop <service>` | Остановка сервиса |
| **Перезапуск** | `restart <service>` | Перезапуск сервиса |
| **Reload** | `reload [service]` | Перезагрузка конфигурации |
| **Статус** | `status [service]` | Статус сервисов |
| **Список** | `list` | Список сервисов |
| **Включение** | `enable <service>` | Включение автозагрузки |
| **Отключение** | `disable <service>` | Отключение автозагрузки |
| **Конфиг** | `config show <service>` | Показать конфигурацию |
| **Конфиг** | `config edit <service>` | Редактировать конфигурацию |
| **Конфиг** | `config check <file>` | Проверить конфигурацию |
| **Конфиг** | `config export <out>` | Экспорт конфигурации |
| **Конфиг** | `config import <file>` | Импорт конфигурации |
| **Rollback** | `rollback list [service]` | Список точек отката |
| **Rollback** | `rollback create <service>` | Создать точку отката |
| **Rollback** | `rollback restore <id> <svc>` | Восстановить из точки |
| **Rollback** | `rollback delete <id> <svc>` | Удалить точку отката |
| **Rollback** | `rollback show <id>` | Показать точку отката |
| **Система** | `info` | Информация о системе |
| **Система** | `optimize` | Оптимизация системы |
| **Система** | `validate [file]` | Валидация конфигурации |
| **Система** | `init` | Инициализация конфигурации |
| **Логи** | `logs [service]` | Просмотр логов |
| **Демон** | `daemon` | Запуск демона |

---

## Примеры использования

### 1. Установка и запуск веб-сервера

```bash
# Установить ZIN
sudo python3 installer.py install

# Установить nginx
sudo python3 installer.py install-app nginx

# Создать точку отката перед изменениями
zin rollback create nginx --description "Перед настройкой"

# Включить автозагрузку
zin enable nginx

# Запустить
zin start nginx

# Проверить статус
zin status nginx
```

### 2. Развёртывание базы данных

```bash
# Установить PostgreSQL
sudo python3 installer.py install-app postgresql

# Создать директорию для данных
sudo python3 installer.py create-dir /var/lib/postgresql/data

# Создать конфигурацию
sudo python3 installer.py create-file /etc/zin/services.d/postgresql.toml --content """
name = "postgresql"
description = "PostgreSQL"
exec = "/usr/lib/postgresql/15/bin/postgres"
priority = 40
category = "database"
enabled = true
user = "postgres"
"""

# Включить и запустить
zin enable postgresql
zin start postgresql

# Проверить статус
zin status postgresql
```

### 3. Оптимизация сервера

```bash
# Показать рекомендации
zin optimize --dry-run

# Применить оптимизацию для server
zin optimize --system-type server

# Проверить статус
zin status

# Посмотреть логи
zin logs --lines 100
```

### 4. Управление конфигурацией

```bash
# Экспорт текущей конфигурации
zin config export /backup/zin-config.toml

# Внести изменения в бэкап
vim /backup/zin-config.toml

# Проверить валидность
zin config check /backup/zin-config.toml

# Импортировать обратно
zin config import /backup/zin-config.toml --overwrite

# Перезагрузить конфигурацию
zin reload
```

### 5. Аварийное восстановление

```bash
# Список точек отката
zin rollback list nginx

# Показать информацию о точке
zin rollback show abc-123-def

# Восстановить из точки
zin rollback restore abc-123-def nginx --yes

# Перезапустить сервис
zin restart nginx

# Проверить статус
zin status nginx
```

---

## Переменные окружения

| Переменная | Описание | По умолчанию |
|------------|----------|--------------|
| `ZIN_CONFIG` | Путь к конфигурации | `/etc/zin` |
| `ZIN_LOG_LEVEL` | Уровень логирования | `info` |
| `ZIN_BACKEND` | Предпочитаемый бэкенд | `auto` |
| `EDITOR` | Редактор для `config edit` | `nano` |

---

## Коды возврата

| Код | Описание |
|-----|----------|
| `0` | Успех |
| `1` | Общая ошибка |
| `2` | Сервис не найден |
| `3` | Конфигурация невалидна |
| `4` | Недостаточно прав |
| `5` | Бэкенд недоступен |

---

## См. также

- [README.md](README.md) - Основная документация
- [INSTALLER_DOCS.md](INSTALLER_DOCS.md) - Документация установщика
- [examples/](examples/) - Примеры конфигураций
