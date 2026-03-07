#!/bin/bash
# Скрипт сборки ZIN

set -e

echo "=== ZIN Build Script ==="

# Проверка наличия Rust
if ! command -v cargo &> /dev/null; then
    echo "Rust не найден. Установите: https://rustup.rs/"
    exit 1
fi

echo "Версия Rust: $(rustc --version)"

# Сборка
echo "Сборка..."
cargo build --release

echo ""
echo "=== Сборка завершена ==="
echo ""
echo "Бинарники:"
echo "  - target/release/zin"
echo "  - target/release/zind"
echo ""
echo "Для установки выполните: sudo ./install.sh"
