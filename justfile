# Показать список рецептов
default:
    @just --list

# Сборка всего воркспейса
build:
    cargo build --workspace

# Релизная сборка: бинарники в target/release/
release:
    cargo build --release --workspace

# Открыть окно
gui:
    cargo run -p igla-gui

# Проверить файл проекта и показать параметры в СИ
check file="examples/annular-lh2.toml":
    cargo run -q -p igla-cli -- check {{file}}

# Напечатать методические указания
docs:
    cargo run -q -p igla-cli -- docs

# Тесты
test:
    cargo test --workspace

# Отформатировать
fmt:
    cargo fmt --all

# Линтер с теми же настройками, что в CI
lint:
    cargo clippy --workspace --all-targets -- -D warnings

# Всё, что гоняет CI — прогнать перед пушем
ci:
    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace

# Убрать артефакты сборки
clean:
    cargo clean
