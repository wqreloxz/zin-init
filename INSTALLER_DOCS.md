# ZIN Installer & Manager - Документация

## Обзор

`installer.py` — универсальный установщик и менеджер для ZIN Init System с поддержкой:
- Установки/удаления ZIN
- Управления приложениями (nginx, postgresql, docker, etc.)
- Создания файлов и директорий
- Управления сервисами

## Быстрый старт

```bash
# Установка ZIN
sudo python3 installer.py install

# Установка приложения
sudo python3 installer.py install-app nginx

# Показать статус
python3 installer.py status

# Список доступных приложений
python3 installer.py list-apps
```

## Команды

### Основные команды

| Команда | Описание |
|---------|----------|
| `install` | Установить ZIN Init System |
| `uninstall` | Удалить ZIN (в разработке) |
| `status` | Показать статус установки |
| `list-apps` | Список доступных приложений |

### Управление приложениями

| Команда | Описание | Пример |
|---------|----------|--------|
| `install-app <name>` | Установить приложение | `sudo python3 installer.py install-app nginx` |
| `remove-app <name>` | Удалить приложение | `sudo python3 installer.py remove-app nginx` |

### Работа с файлами и папками

| Команда | Описание | Пример |
|---------|----------|--------|
| `create-dir <path>` | Создать директорию | `sudo python3 installer.py create-dir /var/www/myapp` |
| `create-file <path>` | Создать файл | `sudo python3 installer.py create-file /etc/myapp/config.toml --content "key=value"` |

### Управление сервисами

| Команда | Описание | Пример |
|---------|----------|--------|
| `enable <service>` | Включить сервис | `sudo python3 installer.py enable nginx` |
| `disable <service>` | Отключить сервис | `sudo python3 installer.py disable nginx` |
| `restart <service>` | Перезапустить сервис | `sudo python3 installer.py restart nginx` |

## Опции

| Опция | Описание | Пример |
|-------|----------|--------|
| `--prefix <path>` | Префикс установки | `--prefix /usr/local` |
| `--dry-run` | Пробный запуск без изменений | `--dry-run install` |
| `--content <text>` | Содержимое для файла | `--content "hello world"` |
| `--mode <octal>` | Режим доступа для файла | `--mode 755` |

## Доступные приложения

| Приложение | Пакет | Сервис | Описание |
|------------|-------|--------|----------|
| `nginx` | nginx | nginx | Веб-сервер |
| `postgresql` | postgresql | postgresql | База данных PostgreSQL |
| `mysql` | mysql-server | mysql | База данных MySQL |
| `docker` | docker.io | docker | Контейнеризация |
| `redis` | redis-server | redis | In-memory хранилище |
| `nodejs` | nodejs | - | JavaScript runtime |
| `python3` | python3 | - | Python 3 |
| `git` | git | - | Система контроля версий |

## Примеры использования

### 1. Полная установка веб-сервера

```bash
# Установить ZIN
sudo python3 installer.py install

# Установить nginx
sudo python3 installer.py install-app nginx

# Включить автозагрузку
sudo python3 installer.py enable nginx

# Запустить
zin start nginx
```

### 2. Установка базы данных

```bash
# Установить PostgreSQL
sudo python3 installer.py install-app postgresql

# Создать директорию для данных
sudo python3 installer.py create-dir /var/lib/postgresql/data

# Включить сервис
sudo python3 installer.py enable postgresql

# Запустить
zin start postgresql
```

### 3. Установка Docker

```bash
# Установить Docker
sudo python3 installer.py install-app docker

# Создать конфигурацию
sudo python3 installer.py create-file /etc/docker/daemon.json --content '{"registry-mirrors": []}'

# Включить сервис
sudo python3 installer.py enable docker

# Запустить
zin start docker
```

### 4. Создание структуры проекта

```bash
# Создать директорию проекта
sudo python3 installer.py create-dir /var/www/myapp

# Создать поддиректории
sudo python3 installer.py create-dir /var/www/myapp/static
sudo python3 installer.py create-dir /var/www/myapp/templates
sudo python3 installer.py create-dir /var/www/myapp/logs

# Создать файл конфигурации
sudo python3 installer.py create-file /var/www/myapp/config.toml --content """
[server]
host = "0.0.0.0"
port = 8080

[database]
url = "postgresql://localhost/myapp"
"""

# Создать файл приложения
sudo python3 installer.py create-file /var/www/myapp/main.py --content """
print("Hello from myapp!")
""" --mode 755
```

### 5. Пробный запуск (dry-run)

```bash
# Проверка что будет сделано без реальных изменений
python3 installer.py --dry-run install
python3 installer.py --dry-run install-app nginx
python3 installer.py --dry-run create-dir /test/dir
```

### 6. Проверка статуса

```bash
# Общий статус
python3 installer.py status

# Список приложений
python3 installer.py list-apps
```

## Интеграция с ZIN CLI

После установки приложений через installer.py, вы можете управлять ими через ZIN CLI:

```bash
# Статус сервисов
zin status

# Запуск всех сервисов
zin start

# Перезапуск конкретного сервиса
zin restart nginx

# Создание точки отката
zin rollback create nginx --description "Перед обновлением"

# Просмотр логов
zin logs nginx --lines 100
```

## Структура конфигурации

После установки, конфигурация располагается в `/etc/zin/`:

```
/etc/zin/
├── zin.toml              # Основная конфигурация
├── services.d/           # Конфигурации сервисов
│   ├── nginx.toml
│   ├── postgresql.toml
│   └── docker.toml
├── rollback/             # Точки отката
└── targets/              # Целевые профили
```

## Переменные окружения

| Переменная | Описание | По умолчанию |
|------------|----------|--------------|
| `ZIN_PREFIX` | Префикс установки | `/usr/local` |
| `ZIN_CONFIG` | Путь к конфигурации | `/etc/zin` |
| `ZIN_LOG` | Путь к логам | `/var/log/zin` |

## Логирование

Все операции записываются в лог-файл:

```bash
# Просмотр лога установки
cat /var/log/zin_install.log

# Следить за логом
tail -f /var/log/zin_install.log
```

## Устранение проблем

### Ошибка: "Требуется запуск от root"

```bash
# Используйте sudo
sudo python3 installer.py install
```

### Ошибка: "Менеджер пакетов не найден"

Установщик поддерживает:
- apt (Debian/Ubuntu)
- yum (RHEL/CentOS)
- pacman (Arch Linux)
- apk (Alpine Linux)

### Ошибка: "Модуль не найден"

Установите зависимости Python:

```bash
pip install requests
```

## Скрипты для автоматизации

### Пример: Автоматическая установка веб-сервера

```bash
#!/bin/bash
# install-webserver.sh

set -e

echo "=== Установка веб-сервера ==="

# Установка ZIN
python3 installer.py install

# Установка nginx
python3 installer.py install-app nginx

# Создание структуры проекта
python3 installer.py create-dir /var/www/myapp
python3 installer.py create-file /var/www/myapp/index.html --content "<h1>Hello!</h1>"

# Включение сервиса
python3 installer.py enable nginx

# Запуск
zin start nginx

echo "Готово! Веб-сервер запущен."
```

### Пример: Установка Docker окружения

```bash
#!/bin/bash
# install-docker.sh

set -e

echo "=== Установка Docker ==="

# Установка Docker
python3 installer.py install-app docker

# Создание конфигурации
python3 installer.py create-file /etc/docker/daemon.json --content '{
    "registry-mirrors": ["https://mirror.gcr.io"],
    "log-driver": "json-file",
    "log-opts": {
        "max-size": "10m",
        "max-file": "3"
    }
}'

# Включение сервиса
python3 installer.py enable docker

# Запуск
zin start docker

echo "Docker установлен!"
```

## См. также

- [README.md](README.md) - Основная документация ZIN
- [examples/](examples/) - Примеры конфигураций
- [ZIN CLI Documentation](#zin-cli-команды) - Команды ZIN CLI

## Поддержка

Для сообщений об ошибках и предложений используйте GitHub Issues.
