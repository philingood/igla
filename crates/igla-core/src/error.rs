//! Ошибки ядра.

use std::fmt;

/// Что пошло не так при разборе или проверке исходных данных.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// Параметр вне области, где модель что-то означает.
    OutOfRange {
        /// Имя поля так, как его видит пользователь.
        field: &'static str,
        value: f64,
        /// Человеческое описание допустимого диапазона.
        expected: &'static str,
    },
    /// Файл проекта не разобрался.
    Parse(String),
    /// Файл проекта объявляет версию схемы, которой эта сборка не знает.
    UnsupportedSchema { found: u32, supported: u32 },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::OutOfRange {
                field,
                value,
                expected,
            } => write!(f, "{field} = {value}: требуется {expected}"),
            Error::Parse(msg) => write!(f, "файл проекта не разобран: {msg}"),
            Error::UnsupportedSchema { found, supported } => write!(
                f,
                "схема файла проекта {found} не поддерживается (эта сборка понимает {supported})"
            ),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
