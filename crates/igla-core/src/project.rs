//! Формат файла проекта.
//!
//! Единственное место во всём крейте, где встречаются бары, миллиметры,
//! километры и проценты. Всё, что уходит внутрь, уже приведено к СИ
//! (см. [`crate::params`]). Поля названы с единицей в имени именно затем,
//! чтобы при чтении файла не приходилось гадать.

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::params::{Contour, DesignPoint, Geometry, Params};
use crate::propellant::Propellant;

/// Версия схемы, которую понимает эта сборка.
///
/// Файл всегда несёт `schema = N`. Стоит один раз выпустить формат без номера —
/// и совместимость задним числом уже не починить.
pub const SCHEMA: u32 = 1;

/// Файл проекта как он лежит на диске.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectFile {
    pub schema: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub geometry: GeometryFile,
    pub propellant: PropellantFile,
    pub chamber: ChamberFile,
    pub design: DesignFile,
    #[serde(default)]
    pub contour: ContourFile,
    #[serde(default)]
    pub operating: OperatingFile,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum GeometryFile {
    Annular { throat_radius_mm: f64 },
    Linear { throat_gap_mm: f64, width_mm: f64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum PropellantFile {
    LoxLh2,
    LoxLch4,
    LoxRp1,
    N2o4Udmh,
    Solid,
    Air,
    Custom {
        gamma: f64,
        chamber_temperature_k: f64,
        molar_mass_g_mol: f64,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ChamberFile {
    pub pressure_bar: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum DesignFile {
    AreaRatio { epsilon: f64 },
    PressureRatio { npr: f64 },
    Altitude { altitude_km: f64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ContourFile {
    /// Доля длины иглы после усечения, %. По умолчанию игла полная.
    pub truncation_pct: f64,
    pub points: usize,
}

impl Default for ContourFile {
    fn default() -> Self {
        Self {
            truncation_pct: 100.0,
            points: 160,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct OperatingFile {
    /// Высота полёта для рабочей точки, км. По умолчанию земля.
    pub altitude_km: f64,
}

impl ProjectFile {
    /// Разобрать файл проекта из строки TOML.
    ///
    /// Ядро не читает диск — строку подаёт вызывающий.
    pub fn from_toml_str(text: &str) -> Result<Self> {
        toml::from_str(text).map_err(|e| Error::Parse(e.to_string()))
    }

    /// Записать обратно в TOML.
    pub fn to_toml_string(&self) -> Result<String> {
        toml::to_string_pretty(self).map_err(|e| Error::Parse(e.to_string()))
    }

    /// Привести к расчётным параметрам: проверить схему, перевести в СИ,
    /// прогнать [`Params::validate`].
    pub fn into_params(self) -> Result<Params> {
        if self.schema != SCHEMA {
            return Err(Error::UnsupportedSchema {
                found: self.schema,
                supported: SCHEMA,
            });
        }

        let geometry = match self.geometry {
            GeometryFile::Annular { throat_radius_mm } => Geometry::Annular {
                throat_radius: throat_radius_mm * 1e-3,
            },
            GeometryFile::Linear {
                throat_gap_mm,
                width_mm,
            } => Geometry::Linear {
                throat_gap: throat_gap_mm * 1e-3,
                width: width_mm * 1e-3,
            },
        };

        let propellant = Propellant::from(self.propellant);

        let design = match self.design {
            DesignFile::AreaRatio { epsilon } => DesignPoint::AreaRatio { epsilon },
            DesignFile::PressureRatio { npr } => DesignPoint::PressureRatio { npr },
            DesignFile::Altitude { altitude_km } => DesignPoint::Altitude {
                altitude: altitude_km * 1e3,
            },
        };

        let params = Params {
            geometry,
            propellant,
            chamber_pressure: self.chamber.pressure_bar * 1e5,
            design,
            contour: Contour {
                truncation: self.contour.truncation_pct / 100.0,
                points: self.contour.points,
            },
            flight_altitude: self.operating.altitude_km * 1e3,
        };
        params.validate()?;
        Ok(params)
    }
}

impl From<PropellantFile> for Propellant {
    fn from(file: PropellantFile) -> Self {
        match file {
            PropellantFile::LoxLh2 => Propellant::LoxLh2,
            PropellantFile::LoxLch4 => Propellant::LoxLch4,
            PropellantFile::LoxRp1 => Propellant::LoxRp1,
            PropellantFile::N2o4Udmh => Propellant::N2o4Udmh,
            PropellantFile::Solid => Propellant::Solid,
            PropellantFile::Air => Propellant::Air,
            PropellantFile::Custom {
                gamma,
                chamber_temperature_k,
                molar_mass_g_mol,
            } => Propellant::Custom {
                gamma,
                chamber_temperature: chamber_temperature_k,
                molar_mass: molar_mass_g_mol * 1e-3,
            },
        }
    }
}

impl From<Propellant> for PropellantFile {
    fn from(p: Propellant) -> Self {
        match p {
            Propellant::LoxLh2 => PropellantFile::LoxLh2,
            Propellant::LoxLch4 => PropellantFile::LoxLch4,
            Propellant::LoxRp1 => PropellantFile::LoxRp1,
            Propellant::N2o4Udmh => PropellantFile::N2o4Udmh,
            Propellant::Solid => PropellantFile::Solid,
            Propellant::Air => PropellantFile::Air,
            Propellant::Custom {
                gamma,
                chamber_temperature,
                molar_mass,
            } => PropellantFile::Custom {
                gamma,
                chamber_temperature_k: chamber_temperature,
                molar_mass_g_mol: molar_mass * 1e3,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
schema = 1
name = "Кольцевой LH2"

[geometry]
kind = "annular"
throat_radius_mm = 80.0

[propellant]
kind = "lox-lh2"

[chamber]
pressure_bar = 80.0

[design]
mode = "area_ratio"
epsilon = 40.0

[contour]
truncation_pct = 22.0
points = 160

[operating]
altitude_km = 0.0
"#;

    #[test]
    fn units_are_converted_at_the_boundary() {
        let params = ProjectFile::from_toml_str(SAMPLE)
            .unwrap()
            .into_params()
            .unwrap();
        assert_eq!(params.chamber_pressure, 80e5);
        assert_eq!(params.contour.truncation, 0.22);
        assert!(matches!(
            params.geometry,
            Geometry::Annular { throat_radius } if (throat_radius - 0.08).abs() < 1e-12
        ));
    }

    #[test]
    fn optional_sections_fall_back_to_a_full_spike_at_sea_level() {
        let minimal = r#"
schema = 1
[geometry]
kind = "linear"
throat_gap_mm = 20.0
width_mm = 800.0
[propellant]
kind = "air"
[chamber]
pressure_bar = 5.0
[design]
mode = "pressure_ratio"
npr = 40.0
"#;
        let params = ProjectFile::from_toml_str(minimal)
            .unwrap()
            .into_params()
            .unwrap();
        assert_eq!(params.contour.truncation, 1.0);
        assert_eq!(params.flight_altitude, 0.0);
    }

    #[test]
    fn a_future_schema_is_refused_rather_than_guessed() {
        let text = SAMPLE.replacen("schema = 1", "schema = 2", 1);
        let err = ProjectFile::from_toml_str(&text)
            .unwrap()
            .into_params()
            .unwrap_err();
        assert!(matches!(err, Error::UnsupportedSchema { found: 2, .. }));
    }

    #[test]
    fn round_trip_through_toml_keeps_the_file() {
        let file = ProjectFile::from_toml_str(SAMPLE).unwrap();
        let text = file.to_toml_string().unwrap();
        assert_eq!(ProjectFile::from_toml_str(&text).unwrap(), file);
    }
}
