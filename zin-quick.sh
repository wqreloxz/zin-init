#!/bin/bash
# ZIN Quick Reference - Шпаргалка по командам

set -e

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
RED='\033[0;31m'
NC='\033[0m' # No Color

print_header() {
    echo -e "${CYAN}========================================${NC}"
    echo -e "${CYAN}$1${NC}"
    echo -e "${CYAN}========================================${NC}"
}

print_section() {
    echo -e "\n${BLUE}▶ $1${NC}"
}

print_command() {
    echo -e "  ${GREEN}$1${NC}"
    if [ -n "$2" ]; then
        echo -e "    ${YELLOW}# $2${NC}"
    fi
}

# Основная справка
show_help() {
    print_header "ZIN Quick Reference"
    
    print_section "Установка (installer.py)"
    print_command "sudo python3 installer.py install" "Установить ZIN"
    print_command "sudo python3 installer.py install-app nginx" "Установить nginx"
    print_command "sudo python3 installer.py create-dir /path/to/dir" "Создать директорию"
    print_command "sudo python3 installer.py create-file /path/file --content 'text'" "Создать файл"
    print_command "python3 installer.py status" "Показать статус"
    print_command "python3 installer.py list-apps" "Список приложений"
    
    print_section "Управление сервисами (zin)"
    print_command "zin start" "Запустить все сервисы"
    print_command "zin start nginx" "Запустить nginx"
    print_command "zin stop nginx" "Остановить nginx"
    print_command "zin restart nginx" "Перезапустить nginx"
    print_command "zin status" "Статус всех сервисов"
    print_command "zin status nginx" "Статус nginx"
    print_command "zin list" "Список сервисов"
    print_command "zin enable nginx" "Включить автозагрузку"
    print_command "zin disable nginx" "Отключить автозагрузку"
    
    print_section "Конфигурация"
    print_command "zin config show nginx" "Показать конфигурацию"
    print_command "zin config edit nginx" "Редактировать конфигурацию"
    print_command "zin config export /backup/config.toml" "Экспорт конфигурации"
    print_command "zin config import /backup/config.toml" "Импорт конфигурации"
    
    print_section "Точки отката (Rollback)"
    print_command "zin rollback list" "Список точек отката"
    print_command "zin rollback create nginx -d 'Описание'" "Создать точку отката"
    print_command "zin rollback restore <id> nginx -y" "Восстановить из точки"
    print_command "zin rollback delete <id> nginx" "Удалить точку отката"
    
    print_section "Система"
    print_command "zin info" "Информация о системе"
    print_command "zin optimize" "Оптимизировать под систему"
    print_command "zin optimize --dry-run" "Показать рекомендации"
    print_command "zin validate" "Проверить конфигурацию"
    print_command "zin init" "Инициализировать конфигурацию"
    
    print_section "Логи"
    print_command "zin logs" "Логи всех сервисов"
    print_command "zin logs nginx -n 100" "Последние 100 строк лога"
    print_command "zin logs nginx -f" "Следить за логом"
    
    print_section "Примеры использования"
    echo -e "  ${GREEN}# Установка веб-сервера${NC}"
    echo -e "    sudo python3 installer.py install"
    echo -e "    sudo python3 installer.py install-app nginx"
    echo -e "    zin enable nginx"
    echo -e "    zin start nginx"
    echo ""
    echo -e "  ${GREEN}# Создание точки отката перед изменениями${NC}"
    echo -e "    zin rollback create nginx -d 'Перед обновлением'"
    echo -e "    # ... внесение изменений ..."
    echo -e "    zin rollback restore <id> nginx -y"
    echo ""
    echo -e "  ${GREEN}# Оптимизация сервера${NC}"
    echo -e "    zin optimize --system-type server"
    echo -e "    zin start"
    
    echo ""
}

# Быстрая установка
quick_install() {
    print_header "Быстрая установка ZIN"
    
    echo -e "${YELLOW}Проверка зависимостей...${NC}"
    
    # Проверка Python
    if ! command -v python3 &> /dev/null; then
        echo -e "${RED}✗ Python3 не найден${NC}"
        exit 1
    fi
    echo -e "${GREEN}✓ Python3 найден${NC}"
    
    # Проверка Rust
    if ! command -v cargo &> /dev/null; then
        echo -e "${YELLOW}⚠ Rust не найден. Установка...${NC}"
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source $HOME/.cargo/env
    fi
    echo -e "${GREEN}✓ Rust найден${NC}"
    
    # Сборка
    echo -e "${YELLOW}Сборка ZIN...${NC}"
    cargo build --release
    
    # Установка
    echo -e "${YELLOW}Установка ZIN...${NC}"
    sudo python3 installer.py install
    
    echo -e "${GREEN}✓ Установка завершена!${NC}"
    echo ""
    echo "Для начала работы:"
    echo "  zin info     - информация о системе"
    echo "  zin optimize - оптимизация"
    echo "  zin start    - запуск сервисов"
}

# Показать доступные приложения
show_apps() {
    print_header "Доступные приложения"
    
    python3 installer.py list-apps
}

# Проверка статуса
check_status() {
    print_header "Статус ZIN"
    
    python3 installer.py status
    echo ""
    zin status
}

# Main
case "${1:-help}" in
    help|--help|-h)
        show_help
        ;;
    install)
        quick_install
        ;;
    apps)
        show_apps
        ;;
    status)
        check_status
        ;;
    *)
        echo -e "${RED}Неизвестная команда: $1${NC}"
        show_help
        ;;
esac
