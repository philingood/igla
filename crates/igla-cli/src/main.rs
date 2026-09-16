//! `igla` — командная строка.
//!
//! Пока умеет ровно то, что имеет смысл без физики: проверить файл проекта и
//! показать методичку. Команды `solve`, `contour` и `sweep` появятся вместе с
//! портом расчёта — выкладывать заглушки, которые печатают «не реализовано»,
//! смысла нет.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use igla_core::ProjectFile;

/// Методичка вшита в бинарник: `igla docs` обязан работать там, где
/// репозитория рядом нет.
const METHOD: &str = include_str!("../../../docs/method.md");

#[derive(Parser)]
#[command(
    name = "igla",
    version,
    about = "Расчёт сопла внешнего расширения (метод Ангелино)"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Проверить файл проекта и напечатать параметры, приведённые к СИ
    Check {
        /// Путь к файлу проекта (TOML)
        file: PathBuf,
    },
    /// Напечатать методические указания
    Docs,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("igla: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    match Cli::parse().command {
        Command::Check { file } => {
            let text =
                std::fs::read_to_string(&file).map_err(|e| format!("{}: {e}", file.display()))?;
            let params = ProjectFile::from_toml_str(&text)
                .and_then(ProjectFile::into_params)
                .map_err(|e| format!("{}: {e}", file.display()))?;
            let json = serde_json::to_string_pretty(&params).map_err(|e| e.to_string())?;
            println!("{json}");
            Ok(())
        }
        Command::Docs => {
            print!("{METHOD}");
            Ok(())
        }
    }
}
