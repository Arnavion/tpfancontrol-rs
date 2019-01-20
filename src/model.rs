#[derive(Debug)]
pub(crate) struct State {
	pub(crate) config: Config,

	pub(crate) temps: Result<Vec<Option<crate::acpi::Temp>>, crate::Error>,
	pub(crate) visible_temp_sensors: VisibleTempSensors,
	pub(crate) temp_scale: crate::acpi::TempScale,

	pub(crate) fan_is_writable: bool,

	pub(crate) fan: Result<(crate::acpi::FanLevel, crate::acpi::FanSpeed), crate::Error>,
	pub(crate) desired_fan_mode: DesiredFanMode,
	pub(crate) desired_manual_fan_level: DesiredManualFanLevel,
}

impl State {
	pub(crate) fn new(fan_update_interval: std::time::Duration) -> Result<Self, crate::Error> {
		let config: Config = {
			let mut file = std::fs::File::open("/etc/tpfancontrol/config.toml").map_err(crate::Error::Config)?;
			let mut config = String::new();
			let _ = std::io::Read::read_to_string(&mut file, &mut config).map_err(crate::Error::Config)?;
			toml::from_str(&config).map_err(|err| crate::Error::Config(std::io::Error::new(std::io::ErrorKind::Other, err)))?
		};

		let num_temp_sensors = config.sensors.len();
		let mut temps = vec![None; num_temp_sensors];
		let temps = crate::acpi::read_temps(&mut temps).map(|()| temps);

		Ok(State {
			config,

			temps,
			visible_temp_sensors: Default::default(),
			temp_scale: Default::default(),

			fan_is_writable: crate::acpi::fan_is_writable(fan_update_interval)?,

			fan: crate::acpi::read_fan(),
			desired_fan_mode: Default::default(),
			desired_manual_fan_level: Default::default(),
		})
	}

	pub(crate) fn update_sensors(&mut self) {
		self.temps = std::mem::replace(&mut self.temps, Ok(vec![])).and_then(|mut temps| crate::acpi::read_temps(&mut temps[..]).map(|()| temps));

		self.fan = crate::acpi::read_fan();
	}
}

#[derive(Debug)]
pub(crate) struct Config {
	pub(crate) sensors: Vec<Option<String>>,
	pub(crate) fan_level: Vec<(crate::acpi::Temp, DesiredManualFanLevel)>,
}

impl<'de> serde::Deserialize<'de> for Config {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::de::Deserializer<'de> {
		struct Inner {
			sensors: std::collections::HashMap<String, String>,
			fan_level: std::collections::HashMap<String, String>,
		}

		// TODO: Replace with `#[derive(serde_derive::Deserialize)]` when https://github.com/rust-lang/rust/issues/55779 is fixed
		impl<'de> serde::Deserialize<'de> for Inner {
			fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
				struct Visitor;

				impl<'de> serde::de::Visitor<'de> for Visitor {
					type Value = Inner;

					fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
						write!(f, "struct Config")
					}

					fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error> where A: serde::de::MapAccess<'de> {
						let mut value_sensors: Option<_> = None;
						let mut value_fan_level: Option<_> = None;

						while let Some(key) = serde::de::MapAccess::next_key(&mut map)? {
							match key {
								"sensors" => value_sensors = serde::de::MapAccess::next_value(&mut map)?,
								"fan_level" => value_fan_level = serde::de::MapAccess::next_value(&mut map)?,
								_ => { let _: serde::de::IgnoredAny = serde::de::MapAccess::next_value(&mut map)?; },
							}
						}

						Ok(Inner {
							sensors: value_sensors.ok_or_else(|| serde::de::Error::missing_field("sensors"))?,
							fan_level: value_fan_level.ok_or_else(|| serde::de::Error::missing_field("fan_level"))?,
						})
					}
				}

				deserializer.deserialize_struct("Config", &["sensors", "fan_level"], Visitor)
			}
		}

		let inner: Inner = serde::Deserialize::deserialize(deserializer)?;

		let mut result = Config {
			sensors: Default::default(),
			fan_level: Default::default(),
		};

		for (key, value) in inner.sensors {
			let index = key.parse().map_err(|_| serde::de::Error::invalid_value(serde::de::Unexpected::Str(&key), &"a sensor index"))?;
			if result.sensors.len() < index {
				result.sensors.resize(index, None);
			}
			result.sensors[index - 1] = Some(value);
		}

		for (key, value) in inner.fan_level {
			let temp: f64 = key.parse().map_err(|_| serde::de::Error::invalid_value(serde::de::Unexpected::Str(&key), &"a temperature in degrees Celsius"))?;
			let level = match &*value {
				"0" => DesiredManualFanLevel::Firmware(crate::acpi::FanFirmwareLevel::Zero),
				"1" => DesiredManualFanLevel::Firmware(crate::acpi::FanFirmwareLevel::One),
				"2" => DesiredManualFanLevel::Firmware(crate::acpi::FanFirmwareLevel::Two),
				"3" => DesiredManualFanLevel::Firmware(crate::acpi::FanFirmwareLevel::Three),
				"4" => DesiredManualFanLevel::Firmware(crate::acpi::FanFirmwareLevel::Four),
				"5" => DesiredManualFanLevel::Firmware(crate::acpi::FanFirmwareLevel::Five),
				"6" => DesiredManualFanLevel::Firmware(crate::acpi::FanFirmwareLevel::Six),
				"7" => DesiredManualFanLevel::Firmware(crate::acpi::FanFirmwareLevel::Seven),
				"full-speed" => DesiredManualFanLevel::FullSpeed,
				_ => return Err(serde::de::Error::invalid_value(serde::de::Unexpected::Str(&value), &"0-7 or full-speed")),
			};

			result.fan_level.push((crate::acpi::Temp(temp.into()), level));
		}

		result.fan_level.sort_by_key(|(temp, _)| *temp);

		Ok(result)
	}
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum VisibleTempSensors {
	All,
	Active,
}

impl Default for VisibleTempSensors {
	fn default() -> Self {
		VisibleTempSensors::Active
	}
}

impl std::fmt::Display for VisibleTempSensors {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			VisibleTempSensors::All => write!(f, "all"),
			VisibleTempSensors::Active => write!(f, "active"),
		}
	}
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum DesiredFanMode {
	Bios,
	Smart,
	Manual,
}

impl Default for DesiredFanMode {
	fn default() -> Self {
		DesiredFanMode::Smart
	}
}

impl std::fmt::Display for DesiredFanMode {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			DesiredFanMode::Bios => write!(f, "BIOS"),
			DesiredFanMode::Smart => write!(f, "Smart"),
			DesiredFanMode::Manual => write!(f, "Manual"),
		}
	}
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DesiredManualFanLevel {
	Firmware(crate::acpi::FanFirmwareLevel),
	FullSpeed,
}

impl Default for DesiredManualFanLevel {
	fn default() -> Self {
		DesiredManualFanLevel::FullSpeed
	}
}

impl std::fmt::Display for DesiredManualFanLevel {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			DesiredManualFanLevel::Firmware(fan_firmware_level) => write!(f, "{}", fan_firmware_level),
			DesiredManualFanLevel::FullSpeed => write!(f, "Full speed"),
		}
	}
}
