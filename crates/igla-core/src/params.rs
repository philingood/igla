//! Исходные данные расчёта — в СИ.

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::propellant::Propellant;

/// Схема сопла.
///
/// Размер критики лежит внутри варианта намеренно: у кольцевой иглы это радиус
/// кольца, у линейного модуля — высота щели, и подставить одно вместо другого
/// нельзя. В прототипе было одно поле `throat`, смысл которого зависел от
/// соседнего поля `geometry` — источник тихих ошибок.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Geometry {
    /// Кольцевая (осесимметричная) игла.
    Annular {
        /// Радиус критического сечения, м.
        throat_radius: f64,
    },
    /// Линейный двускатный модуль.
    Linear {
        /// Высота критической щели, м.
        throat_gap: f64,
        /// Длина щели (ширина модуля), м.
        width: f64,
    },
}

/// Чем задана расчётная точка. Ровно одно из трёх — по построению типа.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum DesignPoint {
    /// Степень расширения ε = Ae/At.
    AreaRatio { epsilon: f64 },
    /// Перепад на сопле pк/pа в расчётной точке.
    PressureRatio { npr: f64 },
    /// Расчётная высота, м; давление среды берётся из модели стандартной атмосферы.
    Altitude { altitude: f64 },
}

/// Параметры построения контура.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Contour {
    /// Доля полной длины иглы, которая остаётся после усечения: 0 < τ ≤ 1.
    pub truncation: f64,
    pub points: usize,
}

/// Полный набор исходных данных.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Params {
    pub geometry: Geometry,
    pub propellant: Propellant,
    pub chamber_pressure: f64,
    pub design: DesignPoint,
    pub contour: Contour,
    pub flight_altitude: f64,
}

/// Верхняя граница модели стандартной атмосферы, м.
pub const MAX_ALTITUDE: f64 = 86_000.0;

impl Params {
    /// Проверить, что данные попадают в область применимости модели.
    ///
    /// Вызывается один раз на границе; дальше решатель считает данные валидными.
    pub fn validate(&self) -> Result<()> {
        self.propellant.validate()?;

        match self.geometry {
            Geometry::Annular { throat_radius } => {
                positive("geometry.throat_radius", throat_radius)?;
            }
            Geometry::Linear { throat_gap, width } => {
                positive("geometry.throat_gap", throat_gap)?;
                positive("geometry.width", width)?;
            }
        }

        positive("chamber.pressure", self.chamber_pressure)?;

        match self.design {
            DesignPoint::AreaRatio { epsilon } => {
                if !epsilon.is_finite() || epsilon <= 1.0 {
                    return Err(Error::OutOfRange {
                        field: "design.epsilon",
                        value: epsilon,
                        expected: "ε > 1 — иначе расширяющегося участка нет",
                    });
                }
            }
            DesignPoint::PressureRatio { npr } => {
                if !npr.is_finite() || npr <= 1.0 {
                    return Err(Error::OutOfRange {
                        field: "design.npr",
                        value: npr,
                        expected: "pк/pа > 1 — иначе течение не разгоняется",
                    });
                }
            }
            DesignPoint::Altitude { altitude } => {
                if !(0.0..=MAX_ALTITUDE).contains(&altitude) {
                    return Err(Error::OutOfRange {
                        field: "design.altitude",
                        value: altitude,
                        expected: "0 … 86 000 м — область модели атмосферы",
                    });
                }
            }
        }

        if !self.contour.truncation.is_finite()
            || self.contour.truncation <= 0.0
            || self.contour.truncation > 1.0
        {
            return Err(Error::OutOfRange {
                field: "contour.truncation",
                value: self.contour.truncation,
                expected: "0 < τ ≤ 1",
            });
        }
        if self.contour.points < 16 {
            return Err(Error::OutOfRange {
                field: "contour.points",
                value: self.contour.points as f64,
                expected: "не меньше 16 точек",
            });
        }
        if !(0.0..=MAX_ALTITUDE).contains(&self.flight_altitude) {
            return Err(Error::OutOfRange {
                field: "operating.altitude",
                value: self.flight_altitude,
                expected: "0 … 86 000 м — область модели атмосферы",
            });
        }
        Ok(())
    }
}

fn positive(field: &'static str, value: f64) -> Result<()> {
    if value > 0.0 {
        Ok(())
    } else {
        Err(Error::OutOfRange {
            field,
            value,
            expected: "значение больше нуля",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Params {
        Params {
            geometry: Geometry::Annular {
                throat_radius: 0.08,
            },
            propellant: Propellant::LoxLh2,
            chamber_pressure: 80e5,
            design: DesignPoint::AreaRatio { epsilon: 40.0 },
            contour: Contour {
                truncation: 0.22,
                points: 160,
            },
            flight_altitude: 0.0,
        }
    }

    #[test]
    fn sample_is_valid() {
        assert!(sample().validate().is_ok());
    }

    #[test]
    fn area_ratio_below_one_is_rejected_not_clamped() {
        let mut p = sample();
        p.design = DesignPoint::AreaRatio { epsilon: 0.8 };
        let err = p.validate().unwrap_err();
        assert!(matches!(
            err,
            Error::OutOfRange {
                field: "design.epsilon",
                ..
            }
        ));
    }

    #[test]
    fn zero_throat_is_rejected() {
        let mut p = sample();
        p.geometry = Geometry::Annular { throat_radius: 0.0 };
        assert!(p.validate().is_err());
    }
}
