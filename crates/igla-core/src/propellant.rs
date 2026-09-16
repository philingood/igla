//! Топливные пары и их термодинамические свойства.
//!
//! Значения — осреднённые по камере, пригодные для учебного расчёта: состав
//! продуктов сгорания не считается, γ постоянна по соплу.

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

/// Свойства рабочего тела, уже приведённые к числам.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PropellantProperties {
    /// Показатель адиабаты γ, безразмерный.
    pub gamma: f64,
    /// Температура в камере, К.
    pub chamber_temperature: f64,
    /// Молярная масса продуктов сгорания, кг/моль.
    pub molar_mass: f64,
}

/// Топливная пара: либо из таблицы, либо заданная вручную.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum Propellant {
    /// Кислород / водород.
    LoxLh2,
    /// Кислород / метан.
    LoxLch4,
    /// Кислород / керосин.
    LoxRp1,
    /// Амил / гептил.
    N2o4Udmh,
    /// РДТТ (АП / HTPB).
    Solid,
    /// Холодный воздух — лабораторная продувка.
    Air,
    /// Задано вручную.
    Custom {
        gamma: f64,
        /// К.
        chamber_temperature: f64,
        /// кг/моль.
        molar_mass: f64,
    },
}

impl Propellant {
    /// Термодинамика этой пары.
    pub fn properties(self) -> PropellantProperties {
        let (gamma, chamber_temperature, molar_mass_g) = match self {
            Propellant::LoxLh2 => (1.20, 3560.0, 13.6),
            Propellant::LoxLch4 => (1.23, 3520.0, 20.0),
            Propellant::LoxRp1 => (1.24, 3670.0, 21.4),
            Propellant::N2o4Udmh => (1.26, 3380.0, 22.6),
            Propellant::Solid => (1.25, 3200.0, 25.0),
            Propellant::Air => (1.40, 300.0, 28.97),
            Propellant::Custom {
                gamma,
                chamber_temperature,
                molar_mass,
            } => {
                return PropellantProperties {
                    gamma,
                    chamber_temperature,
                    molar_mass,
                };
            }
        };
        PropellantProperties {
            gamma,
            chamber_temperature,
            // Таблица — в г/моль, как во всех справочниках; ядро держит кг/моль.
            molar_mass: molar_mass_g * 1e-3,
        }
    }

    /// Название для интерфейса.
    pub fn label(self) -> &'static str {
        match self {
            Propellant::LoxLh2 => "Кислород / водород",
            Propellant::LoxLch4 => "Кислород / метан",
            Propellant::LoxRp1 => "Кислород / керосин",
            Propellant::N2o4Udmh => "Амил / гептил",
            Propellant::Solid => "РДТТ",
            Propellant::Air => "Воздух (холодный)",
            Propellant::Custom { .. } => "Задано вручную",
        }
    }

    /// Табличные пары в порядке показа в интерфейсе.
    pub const TABLE: [Propellant; 6] = [
        Propellant::LoxLh2,
        Propellant::LoxLch4,
        Propellant::LoxRp1,
        Propellant::N2o4Udmh,
        Propellant::Solid,
        Propellant::Air,
    ];

    pub(crate) fn validate(self) -> Result<()> {
        let p = self.properties();
        if !(1.0..2.0).contains(&p.gamma) {
            return Err(Error::OutOfRange {
                field: "propellant.gamma",
                value: p.gamma,
                expected: "1 < γ < 2",
            });
        }
        if p.chamber_temperature <= 0.0 {
            return Err(Error::OutOfRange {
                field: "propellant.chamber_temperature",
                value: p.chamber_temperature,
                expected: "температура выше нуля, К",
            });
        }
        if p.molar_mass <= 0.0 {
            return Err(Error::OutOfRange {
                field: "propellant.molar_mass",
                value: p.molar_mass,
                expected: "молярная масса больше нуля",
            });
        }
        Ok(())
    }
}
