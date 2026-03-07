# ZIN Cheat Sheet - Шпаргалка

## 📦 Установка

```bash
# Установить ZIN
sudo python3 installer.py install

# Установить приложение
sudo python3 installer.py install-app <nginx|postgresql|mysql|docker|redis>

# Создать директорию
sudo python3 installer.py create-dir /path/to/dir

# Создать файл
sudo python3 installer.py create-file /path/to/file --content "текст"
```

## 🎯 Управление сервисами

```bash
zin start              # Запустить все сервисы
zin start nginx        # Запустить nginx
zin stop nginx         # Остановить nginx
zin restart nginx      # Перезапустить nginx
zin status             # Статус всех сервисов
zin list               # Список сервисов
zin enable nginx       # Включить автозагрузку
zin disable nginx      # Отключить автозагрузку
```

## 📁 Конфигурация

```bash
zin config show nginx           # Показать конфигурацию
zin config edit nginx           # Редактировать (открывает $EDITOR)
zin config export /backup.toml  # Экспорт
zin config import /backup.toml  # Импорт
zin validate                    # Проверка валидности
```

## ↩️ Rollback (Точки отката)

```bash
zin rollback list                      # Список точек
zin rollback create nginx -d "Описание" # Создать
zin rollback restore <id> nginx -y      # Восстановить
zin rollback delete <id> nginx          # Удалить
zin rollback show <id>                  # Информация
```

## 🖥️ Система

```bash
zin info                  # Информация о системе
zin optimize              # Оптимизировать
zin optimize --dry-run    # Показать рекомендации
zin init                  # Инициализировать
zin logs                  # Просмотр логов
zin logs nginx -f         # Следить за логом
```

## 🔧 Installer.py команды

```bash
python3 installer.py install          # Установить ZIN
python3 installer.py install-app nginx # Установить приложение
python3 installer.py remove-app nginx  # Удалить приложение
python3 installer.py status            # Статус установки
python3 installer.py list-apps         # Список приложений
python3 installer.py enable nginx      # Включить сервис
python3 installer.py disable nginx     # Отключить сервис
python3 installer.py restart nginx     # Перезапустить сервис
```

## 📊 Доступные приложения

| Приложение | Команда |
|------------|---------|
| Nginx | `installer.py install-app nginx` |
| PostgreSQL | `installer.py install-app postgresql` |
| MySQL | `installer.py install-app mysql` |
| Docker | `installer.py install-app docker` |
| Redis | `installer.py install-app redis` |
| Node.js | `installer.py install-app nodejs` |
| Python3 | `installer.py install-app python3` |
| Git | `installer.py install-app git` |

## 🚀 Быстрый старт

### Веб-сервер за 5 команд

```bash
sudo python3 installer.py install          # 1. Установить ZIN
sudo python3 installer.py install-app nginx # 2. Установить nginx
zin rollback create nginx -d "Initial"     # 3. Точка отката
zin enable nginx                           # 4. Включить
zin start nginx                            # 5. Запустить
```

### База данных за 5 команд

```bash
sudo python3 installer.py install-app postgresql # 1. Установить
sudo python3 installer.py create-dir /var/lib/pgsql/data # 2. Директория
zin enable postgresql                            # 3. Включить
zin rollback create postgresql                   # 4. Точка отката
zin start postgresql                             # 5. Запустить
```

## 🎨 Типы систем

```bash
zin info  # Определит тип системы

# Workstation: приоритет DE и приложений
# Server: приоритет сервисов
# Embedded: минимальная загрузка

zin optimize --system-type server  # Применить оптимизацию
```

## ⚙️ Глобальные опции

```bash
zin -l debug start      # Уровень логирования
zin -c /path start      # Путь к конфигурации
zin --dry-run start     # Сухой запуск
zin --help              # Справка
```

## 📁 Структура конфигурации

```
/etc/zin/
├── zin.toml              # Основная конфигурация
├── services.d/           # Сервисы
│   ├── nginx.toml
│   ├── postgresql.toml
│   └── ...
├── rollback/             # Точки отката
└── targets/              # Профили
```

## 🔍 Форматы конфигурации

### TOML (читаемый)
```toml
name = "nginx"
exec = "/usr/sbin/nginx"
priority = 50
category = "webserver"
```

### Бинарный (быстрый)
- Автоматически генерируется
- Проверка SHA-256
- Магические байты `ZINCFG01`

## 🆘 Коды возврата

| Код | Значение |
|-----|----------|
| 0 | Успех |
| 1 | Общая ошибка |
| 2 | Сервис не найден |
| 3 | Конфигурация невалидна |
| 4 | Недостаточно прав |

## 📚 Документация

- `README.md` — Основная документация
- `INSTALLER_DOCS.md` — Документация установщика
- `CLI_DOCS.md` — Все команды CLI
- `examples/` — Примеры конфигураций

---

**ZIN v0.1.0** | [github.com/zin-init/zin](https://github.com/zin-init/zin)
