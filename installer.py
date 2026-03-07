#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
ZIN Installer & Manager
Установщик и менеджер приложений для ZIN Init System

Поддерживает:
- Установка/удаление пакетов
- Создание файлов и папок
- Управление сервисами
- Конфигурация системы
"""

import os
import sys
import json
import shutil
import subprocess
import argparse
from pathlib import Path
from datetime import datetime
from typing import Optional, List, Dict

# Цвета для вывода
class Colors:
    GREEN = '\033[92m'
    YELLOW = '\033[93m'
    RED = '\033[91m'
    BLUE = '\033[94m'
    CYAN = '\033[96m'
    RESET = '\033[0m'
    BOLD = '\033[1m'

def print_colored(text: str, color: str = Colors.RESET, bold: bool = False):
    """Вывод цветного текста"""
    prefix = Colors.BOLD if bold else ''
    print(f"{prefix}{color}{text}{Colors.RESET}")

def print_success(text: str):
    print_colored(f"✓ {text}", Colors.GREEN)

def print_error(text: str):
    print_colored(f"✗ {text}", Colors.RED)

def print_warning(text: str):
    print_colored(f"⚠ {text}", Colors.YELLOW)

def print_info(text: str):
    print_colored(f"ℹ {text}", Colors.BLUE)

def print_step(text: str):
    print_colored(f"▶ {text}", Colors.CYAN, bold=True)

class ZinInstaller:
    """Основной класс установщика ZIN"""
    
    def __init__(self, prefix: str = "/usr/local", dry_run: bool = False):
        self.prefix = Path(prefix)
        self.dry_run = dry_run
        self.bin_dir = self.prefix / "bin"
        self.config_dir = Path("/etc/zin")
        self.systemd_dir = Path("/lib/systemd/system")
        self.openrc_dir = Path("/etc/init.d")
        self.log_file = Path("/var/log/zin_install.log")
        
        # Пакеты для установки
        self.packages = {
            "core": {
                "description": "Основные компоненты ZIN",
                "files": ["zin", "zind"],
                "required": True
            },
            "systemd-backend": {
                "description": "Бэкенд для systemd",
                "files": [],
                "required": False
            },
            "openrc-backend": {
                "description": "Бэкенд для OpenRC",
                "files": [],
                "required": False
            },
            "examples": {
                "description": "Примеры конфигураций",
                "files": ["examples/services.d/*.toml"],
                "required": False
            }
        }
        
        # Приложения для управления
        self.applications = {
            "nginx": {
                "package": "nginx",
                "service": "nginx",
                "config_dir": "/etc/nginx",
                "description": "Веб-сервер"
            },
            "postgresql": {
                "package": "postgresql",
                "service": "postgresql",
                "config_dir": "/etc/postgresql",
                "description": "База данных PostgreSQL"
            },
            "mysql": {
                "package": "mysql-server",
                "service": "mysql",
                "config_dir": "/etc/mysql",
                "description": "База данных MySQL"
            },
            "docker": {
                "package": "docker.io",
                "service": "docker",
                "config_dir": "/etc/docker",
                "description": "Контейнеризация"
            },
            "redis": {
                "package": "redis-server",
                "service": "redis",
                "config_dir": "/etc/redis",
                "description": "In-memory хранилище"
            },
            "nodejs": {
                "package": "nodejs",
                "service": None,
                "config_dir": None,
                "description": "JavaScript runtime"
            },
            "python3": {
                "package": "python3",
                "service": None,
                "config_dir": None,
                "description": "Python 3"
            },
            "git": {
                "package": "git",
                "service": None,
                "config_dir": None,
                "description": "Система контроля версий"
            }
        }
    
    def log(self, message: str):
        """Запись в лог"""
        timestamp = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
        log_entry = f"[{timestamp}] {message}\n"
        
        if not self.dry_run:
            try:
                with open(self.log_file, 'a') as f:
                    f.write(log_entry)
            except:
                pass
        
        print_info(message)
    
    def check_root(self) -> bool:
        """Проверка прав root"""
        if os.geteuid() != 0:
            print_error("Требуется запуск от root (sudo)")
            return False
        return True
    
    def detect_init_system(self) -> str:
        """Определение init-системы"""
        if Path("/run/systemd/system").exists():
            return "systemd"
        elif Path("/etc/init.d").exists() and Path("/sbin/openrc-run").exists():
            return "openrc"
        elif Path("/etc/init.d").exists():
            return "sysv"
        else:
            return "unknown"
    
    def create_directories(self):
        """Создание необходимых директорий"""
        dirs = [
            self.bin_dir,
            self.config_dir,
            self.config_dir / "services.d",
            self.config_dir / "rollback",
            self.config_dir / "targets",
            Path("/var/log/zin")
        ]
        
        for dir_path in dirs:
            if self.dry_run:
                print_step(f"[DRY] Создать директорию: {dir_path}")
            else:
                try:
                    dir_path.mkdir(parents=True, exist_ok=True)
                    print_success(f"Создана директория: {dir_path}")
                except Exception as e:
                    print_error(f"Ошибка создания {dir_path}: {e}")
    
    def create_file(self, path: str, content: str, mode: int = 0o644):
        """Создание файла с содержимым"""
        file_path = Path(path)
        
        if self.dry_run:
            print_step(f"[DRY] Создать файл: {file_path}")
            return
        
        try:
            file_path.parent.mkdir(parents=True, exist_ok=True)
            with open(file_path, 'w') as f:
                f.write(content)
            os.chmod(file_path, mode)
            print_success(f"Создан файл: {file_path}")
        except Exception as e:
            print_error(f"Ошибка создания файла {file_path}: {e}")
    
    def copy_file(self, src: str, dst: str, mode: int = 0o755):
        """Копирование файла"""
        src_path = Path(src)
        dst_path = Path(dst)
        
        if not src_path.exists():
            print_warning(f"Исходный файл не найден: {src_path}")
            return
        
        if self.dry_run:
            print_step(f"[DRY] Копировать: {src_path} -> {dst_path}")
            return
        
        try:
            shutil.copy2(src_path, dst_path)
            os.chmod(dst_path, mode)
            print_success(f"Скопировано: {dst_path}")
        except Exception as e:
            print_error(f"Ошибка копирования: {e}")
    
    def install_package(self, package_name: str) -> bool:
        """Установка системного пакета"""
        print_step(f"Установка пакета: {package_name}")
        
        if self.dry_run:
            print_step(f"[DRY] apt install -y {package_name}")
            return True
        
        # Определение менеджера пакетов
        if shutil.which("apt"):
            cmd = ["apt", "install", "-y", package_name]
        elif shutil.which("yum"):
            cmd = ["yum", "install", "-y", package_name]
        elif shutil.which("pacman"):
            cmd = ["pacman", "-S", "--noconfirm", package_name]
        elif shutil.which("apk"):
            cmd = ["apk", "add", package_name]
        else:
            print_error("Менеджер пакетов не найден")
            return False
        
        try:
            result = subprocess.run(cmd, capture_output=True, text=True)
            if result.returncode == 0:
                print_success(f"Пакет установлен: {package_name}")
                return True
            else:
                print_error(f"Ошибка установки: {result.stderr}")
                return False
        except Exception as e:
            print_error(f"Ошибка: {e}")
            return False
    
    def install_zin(self) -> bool:
        """Установка ZIN"""
        print_step("Установка ZIN...")
        
        # Проверка прав
        if not self.check_root():
            return False
        
        # Создание директорий
        self.create_directories()
        
        # Сборка проекта
        print_step("Сборка проекта...")
        if self.dry_run:
            print_step("[DRY] cargo build --release")
        else:
            try:
                result = subprocess.run(
                    ["cargo", "build", "--release"],
                    capture_output=True,
                    text=True,
                    cwd=Path(__file__).parent
                )
                if result.returncode != 0:
                    print_error(f"Ошибка сборки: {result.stderr}")
                    return False
                print_success("Сборка завершена")
            except Exception as e:
                print_error(f"Ошибка сборки: {e}")
                return False
        
        # Копирование бинарников
        target_dir = Path(__file__).parent / "target" / "release"
        self.copy_file(target_dir / "zin", self.bin_dir / "zin", 0o755)
        self.copy_file(target_dir / "zind", self.bin_dir / "zind", 0o755)
        
        # Копирование примеров конфигурации
        examples_src = Path(__file__).parent / "examples" / "services.d"
        if examples_src.exists():
            for toml_file in examples_src.glob("*.toml"):
                self.copy_file(
                    toml_file,
                    self.config_dir / "services.d" / toml_file.name,
                    0o644
                )
        
        # Создание основной конфигурации
        self.create_main_config()
        
        # Установка systemd unit
        init_system = self.detect_init_system()
        if init_system == "systemd":
            self.install_systemd_unit()
        elif init_system == "openrc":
            self.install_openrc_script()
        
        print_success("ZIN успешно установлен!")
        print_info(f"Путь к конфигурации: {self.config_dir}")
        print_info("Команды для начала работы:")
        print("  zin info      - информация о системе")
        print("  zin optimize  - оптимизация под систему")
        print("  zin start     - запуск сервисов")
        
        return True
    
    def create_main_config(self):
        """Создание основной конфигурации"""
        config_content = """# ZIN Main Configuration
# Автоматически сгенерировано установщиком

[general]
log_level = "info"
config_dir = "/etc/zin"
rollback_dir = "/etc/zin/rollback"
max_checkpoints = 10

[optimization]
auto_optimize = true
detect_hardware = true

[backend]
auto_detect = true
preferred = "systemd"
"""
        self.create_file(
            self.config_dir / "zin.toml",
            config_content,
            0o644
        )
    
    def install_systemd_unit(self):
        """Установка systemd unit файла"""
        unit_content = """[Unit]
Description=ZIN Init System Daemon
Documentation=https://github.com/zin-init/zin
DefaultDependencies=no
Before=basic.target
After=local-fs.target

[Service]
Type=oneshot
ExecStart=/usr/local/bin/zin start
RemainAfterExit=yes
ExecStop=/usr/local/bin/zin stop
TimeoutSec=300

[Install]
WantedBy=multi-user.target
"""
        self.create_file(
            self.systemd_dir / "zin.service",
            unit_content,
            0o644
        )
        
        if not self.dry_run:
            subprocess.run(["systemctl", "daemon-reload"], capture_output=True)
    
    def install_openrc_script(self):
        """Установка OpenRC init скрипта"""
        script_content = """#!/sbin/openrc-run

description="ZIN Init System"
command="/usr/local/bin/zin"
command_args="start"
pidfile="/var/run/zin.pid"

depend() {
    need local-fs
    use logger
}

start() {
    ebegin "Starting ZIN"
    $command $command_args
    eend $?
}

stop() {
    ebegin "Stopping ZIN"
    /usr/local/bin/zin stop
    eend $?
}
"""
        self.create_file(
            self.openrc_dir / "zin",
            script_content,
            0o755
        )
        
        if not self.dry_run:
            subprocess.run(["rc-update", "add", "zin", "default"], capture_output=True)
    
    def install_application(self, app_name: str) -> bool:
        """Установка приложения"""
        if app_name not in self.applications:
            print_error(f"Приложение не найдено: {app_name}")
            return False
        
        app = self.applications[app_name]
        print_step(f"Установка приложения: {app_name} ({app['description']})")
        
        # Установка пакета
        if not self.install_package(app["package"]):
            return False
        
        # Создание конфигурации
        if app["config_dir"]:
            config_dir = Path(app["config_dir"])
            if self.dry_run:
                print_step(f"[DRY] Создать директорию: {config_dir}")
            else:
                config_dir.mkdir(parents=True, exist_ok=True)
                print_success(f"Создана директория: {config_dir}")
        
        # Создание сервиса ZIN
        self.create_application_service(app_name, app)
        
        return True
    
    def create_application_service(self, app_name: str, app: dict):
        """Создание сервиса ZIN для приложения"""
        service_templates = {
            "nginx": """name = "nginx"
description = "Веб-сервер Nginx"
exec = "/usr/sbin/nginx"
args = ["-g", "daemon off;"]
priority = 50
category = "webserver"
enabled = true
depends_on = ["network"]
restart_policy = "OnFailure"
user = "www-data"
group = "www-data"
""",
            "postgresql": """name = "postgresql"
description = "PostgreSQL база данных"
exec = "/usr/lib/postgresql/15/bin/postgres"
args = ["-D", "/var/lib/postgresql/15/main"]
priority = 40
category = "database"
enabled = true
depends_on = ["network", "filesystem"]
restart_policy = "OnFailure"
user = "postgres"
group = "postgres"
""",
            "mysql": """name = "mysql"
description = "MySQL база данных"
exec = "/usr/sbin/mysqld"
priority = 40
category = "database"
enabled = true
depends_on = ["filesystem"]
restart_policy = "OnFailure"
user = "mysql"
group = "mysql"
""",
            "docker": """name = "docker"
description = "Docker контейнеризация"
exec = "/usr/bin/dockerd"
args = ["--config-file", "/etc/docker/daemon.json"]
priority = 35
category = "container"
enabled = true
depends_on = ["network", "filesystem"]
restart_policy = "Always"
""",
            "redis": """name = "redis"
description = "Redis in-memory хранилище"
exec = "/usr/bin/redis-server"
args = ["/etc/redis/redis.conf"]
priority = 45
category = "database"
enabled = true
depends_on = ["network"]
restart_policy = "OnFailure"
user = "redis"
group = "redis"
"""
        }
        
        if app_name in service_templates:
            service_content = service_templates[app_name]
            service_path = self.config_dir / "services.d" / f"{app_name}.toml"
            self.create_file(service_path, service_content, 0o644)
    
    def uninstall_application(self, app_name: str) -> bool:
        """Удаление приложения"""
        if app_name not in self.applications:
            print_error(f"Приложение не найдено: {app_name}")
            return False
        
        app = self.applications[app_name]
        print_step(f"Удаление приложения: {app_name}")
        
        # Удаление пакета
        if self.dry_run:
            print_step(f"[DRY] apt remove -y {app['package']}")
        else:
            if shutil.which("apt"):
                cmd = ["apt", "remove", "-y", app["package"]]
            elif shutil.which("yum"):
                cmd = ["yum", "remove", "-y", app["package"]]
            elif shutil.which("pacman"):
                cmd = ["pacman", "-R", "--noconfirm", app["package"]]
            elif shutil.which("apk"):
                cmd = ["apk", "del", app["package"]]
            else:
                print_error("Менеджер пакетов не найден")
                return False
            
            try:
                subprocess.run(cmd, capture_output=True, text=True)
                print_success(f"Пакет удалён: {app['package']}")
            except Exception as e:
                print_error(f"Ошибка удаления: {e}")
        
        # Удаление сервиса ZIN
        service_file = self.config_dir / "services.d" / f"{app_name}.toml"
        if service_file.exists():
            if self.dry_run:
                print_step(f"[DRY] Удалить файл: {service_file}")
            else:
                try:
                    service_file.unlink()
                    print_success(f"Удалён сервис: {service_file}")
                except Exception as e:
                    print_error(f"Ошибка удаления файла: {e}")
        
        return True
    
    def create_directory_structure(self, base_path: str, structure: dict):
        """Создание структуры папок и файлов"""
        base = Path(base_path)
        
        for name, content in structure.items():
            path = base / name
            
            if isinstance(content, dict):
                # Это директория
                if self.dry_run:
                    print_step(f"[DRY] Создать директорию: {path}")
                else:
                    path.mkdir(parents=True, exist_ok=True)
                    print_success(f"Создана директория: {path}")
                
                # Рекурсивное создание
                self.create_directory_structure(str(path), content)
            
            elif isinstance(content, str):
                # Это файл
                self.create_file(path, content, 0o644)
    
    def list_applications(self):
        """Список доступных приложений"""
        print_colored("\nДоступные приложения:", Colors.CYAN, bold=True)
        print("-" * 50)
        
        for name, app in self.applications.items():
            status = "✓" if shutil.which(name) else "○"
            print(f"  {status} {name:15} - {app['description']}")
        
        print()
    
    def status(self):
        """Показать статус установки"""
        print_colored("\nСтатус ZIN:", Colors.CYAN, bold=True)
        print("-" * 50)
        
        # Проверка бинарников
        zin_path = self.bin_dir / "zin"
        if zin_path.exists():
            print_success(f"ZIN установлен: {zin_path}")
        else:
            print_error("ZIN не установлен")
        
        # Проверка конфигурации
        if self.config_dir.exists():
            print_success(f"Конфигурация: {self.config_dir}")
        else:
            print_warning("Конфигурация не найдена")
        
        # Init система
        init_system = self.detect_init_system()
        print_info(f"Init-система: {init_system}")
        
        # Сервисы
        services_dir = self.config_dir / "services.d"
        if services_dir.exists():
            services = list(services_dir.glob("*.toml"))
            print_info(f"Сервисов настроено: {len(services)}")
        
        print()


def main():
    parser = argparse.ArgumentParser(
        description="ZIN Installer & Manager",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Примеры использования:
  %(prog)s install                    - Установить ZIN
  %(prog)s install-app nginx          - Установить nginx
  %(prog)s remove-app nginx           - Удалить nginx
  %(prog)s create-dir /path/to/dir    - Создать директорию
  %(prog)s create-file /path/file.txt - Создать файл
  %(prog)s status                     - Показать статус
  %(prog)s list-apps                  - Список приложений
  %(prog)s --dry-run install          - Пробный запуск
        """
    )
    
    parser.add_argument(
        "command",
        nargs="?",
        choices=[
            "install", "uninstall",
            "install-app", "remove-app",
            "create-dir", "create-file",
            "status", "list-apps",
            "enable", "disable", "restart"
        ],
        help="Команда для выполнения"
    )
    
    parser.add_argument(
        "target",
        nargs="?",
        help="Цель (пакет, файл, директория, сервис)"
    )
    
    parser.add_argument(
        "--prefix",
        default="/usr/local",
        help="Префикс установки (по умолчанию: /usr/local)"
    )
    
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Пробный запуск без реальных изменений"
    )
    
    parser.add_argument(
        "--content",
        help="Содержимое для создаваемого файла"
    )
    
    parser.add_argument(
        "--mode",
        default="644",
        help="Режим доступа для файла (по умолчанию: 644)"
    )
    
    args = parser.parse_args()
    
    # Создание установщика
    installer = ZinInstaller(prefix=args.prefix, dry_run=args.dry_run)
    
    if args.command == "install":
        installer.install_zin()
    
    elif args.command == "uninstall":
        print_warning("Удаление ZIN пока не поддерживается")
    
    elif args.command == "install-app":
        if not args.target:
            print_error("Укажите имя приложения")
            sys.exit(1)
        installer.install_application(args.target)
    
    elif args.command == "remove-app":
        if not args.target:
            print_error("Укажите имя приложения")
            sys.exit(1)
        installer.uninstall_application(args.target)
    
    elif args.command == "create-dir":
        if not args.target:
            print_error("Укажите путь к директории")
            sys.exit(1)
        
        if args.dry_run:
            print_step(f"[DRY] Создать директорию: {args.target}")
        else:
            if installer.check_root():
                Path(args.target).mkdir(parents=True, exist_ok=True)
                print_success(f"Создана директория: {args.target}")
    
    elif args.command == "create-file":
        if not args.target:
            print_error("Укажите путь к файлу")
            sys.exit(1)
        
        content = args.content or ""
        mode = int(args.mode, 8)
        
        if args.dry_run:
            print_step(f"[DRY] Создать файл: {args.target}")
        else:
            if installer.check_root():
                installer.create_file(args.target, content, mode)
    
    elif args.command == "status":
        installer.status()
    
    elif args.command == "list-apps":
        installer.list_applications()
    
    elif args.command == "enable":
        if not args.target:
            print_error("Укажите имя сервиса")
            sys.exit(1)
        if args.dry_run:
            print_step(f"[DRY] zin enable {args.target}")
        else:
            subprocess.run(["zin", "enable", args.target])
    
    elif args.command == "disable":
        if not args.target:
            print_error("Укажите имя сервиса")
            sys.exit(1)
        if args.dry_run:
            print_step(f"[DRY] zin disable {args.target}")
        else:
            subprocess.run(["zin", "disable", args.target])
    
    elif args.command == "restart":
        if not args.target:
            print_error("Укажите имя сервиса")
            sys.exit(1)
        if args.dry_run:
            print_step(f"[DRY] zin restart {args.target}")
        else:
            subprocess.run(["zin", "restart", args.target])
    
    else:
        parser.print_help()


if __name__ == "__main__":
    main()
