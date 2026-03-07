#!/bin/bash
# Скрипт установки ZIN init-системы

set -e

echo "=== ZIN Installation Script ==="

# Проверка прав root
if [ "$EUID" -ne 0 ]; then
    echo "Пожалуйста, запустите от root (sudo ./install.sh)"
    exit 1
fi

# Определение архитектуры
ARCH=$(uname -m)
case $ARCH in
    x86_64) ARCH_DIR="x86_64-unknown-linux-gnu" ;;
    aarch64) ARCH_DIR="aarch64-unknown-linux-gnu" ;;
    *) echo "Неподдерживаемая архитектура: $ARCH"; exit 1 ;;
esac

# Пути установки
PREFIX="${PREFIX:-/usr/local}"
BIN_DIR="${PREFIX}/bin"
CONFIG_DIR="${PREFIX}/etc/zin"
SYSTEMD_DIR="/lib/systemd/system"
OPENRC_DIR="/etc/init.d"

echo "Архитектура: $ARCH"
echo "Префикс: $PREFIX"

# Создание директорий
echo "Создание директорий..."
mkdir -p "$BIN_DIR"
mkdir -p "$CONFIG_DIR/services.d"
mkdir -p "$CONFIG_DIR/rollback"
mkdir -p "$CONFIG_DIR/targets"

# Копирование бинарников
if [ -f "target/release/zin" ]; then
    echo "Копирование zin..."
    cp target/release/zin "$BIN_DIR/"
    chmod +x "$BIN_DIR/zin"
else
    echo "Бинарник не найден. Выполните: cargo build --release"
    exit 1
fi

if [ -f "target/release/zind" ]; then
    echo "Копирование zind..."
    cp target/release/zind "$BIN_DIR/"
    chmod +x "$BIN_DIR/zind"
fi

# Копирование примеров конфигурации
echo "Копирование примеров конфигурации..."
cp examples/services.d/*.toml "$CONFIG_DIR/services.d/" 2>/dev/null || true

# Установка systemd unit (если доступен systemd)
if [ -d "/run/systemd/system" ]; then
    echo "Установка systemd unit..."
    cat > "$SYSTEMD_DIR/zin.service" << 'EOF'
[Unit]
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
EOF
    systemctl daemon-reload
    echo "Systemd unit установлен: zin.service"
fi

# Установка OpenRC init script (если доступен OpenRC)
if [ -d "/etc/init.d" ] && [ ! -d "/run/systemd/system" ]; then
    echo "Установка OpenRC init script..."
    cat > "$OPENRC_DIR/zin" << 'EOF'
#!/sbin/openrc-run

description="ZIN Init System"

depend() {
    need local-fs
    use logger
}

start() {
    ebegin "Starting ZIN"
    /usr/local/bin/zin start
    eend $?
}

stop() {
    ebegin "Stopping ZIN"
    /usr/local/bin/zin stop
    eend $?
}
EOF
    chmod +x "$OPENRC_DIR/zin"
    rc-update add zin default 2>/dev/null || true
    echo "OpenRC init script установлен"
fi

echo ""
echo "=== Установка завершена ==="
echo ""
echo "Дальнейшие шаги:"
echo "  1. Проверьте конфигурацию: zin info"
echo "  2. Оптимизируйте под систему: zin optimize"
echo "  3. Запустите сервисы: zin start"
echo ""
echo "Путь к конфигурации: $CONFIG_DIR"
