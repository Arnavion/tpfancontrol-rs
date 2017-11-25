#[derive(Debug)]
pub struct State {
	pub config: Config,

	pub temps: ::Result<Vec<Option<::acpi::Temp>>>,
	pub visible_temp_sensors: VisibleTempSensors,
	pub temp_scale: ::acpi::TempScale,

	pub fan_is_writable: bool,

	pub fan: ::Result<(::acpi::FanLevel, ::acpi::FanSpeed)>,
	pub desired_fan_mode: DesiredFanMode,
	pub desired_manual_fan_level: DesiredManualFanLevel,
}

impl State {
	pub fn new(fan_update_interval: ::std::time::Duration) -> ::Result<Self> {
		use ::ResultExt;

		let config: Config = {
			let mut file = ::std::fs::File::open("/etc/tpfancontrol/config.toml").chain_err(|| "Could not open /etc/tpfancontrol/config.toml")?;
			let mut config = String::new();
			let _ = ::std::io::Read::read_to_string(&mut file, &mut config)?;
			::toml::from_str(&config)?
		};

		let num_temp_sensors = config.sensors.len();
		let mut temps = vec![None; num_temp_sensors];
		let temps = ::acpi::read_temps(&mut temps).map(|()| temps);

		Ok(State {
			config,

			temps,
			visible_temp_sensors: Default::default(),
			temp_scale: Default::default(),

			fan_is_writable: ::acpi::fan_is_writable(fan_update_interval)?,

			fan: ::acpi::read_fan(),
			desired_fan_mode: Default::default(),
			desired_manual_fan_level: Default::default(),
		})
	}

	pub fn update_sensors(&mut self) {
		self.temps = ::std::mem::replace(&mut self.temps, Ok(vec![])).and_then(|mut temps| ::acpi::read_temps(&mut temps[..]).map(|()| temps));

		self.fan = ::acpi::read_fan();
	}
}

#[derive(Debug)]
pub struct Config {
	pub sensors: Vec<Option<String>>,
	pub fan_level: Vec<(::acpi::Temp, ::model::DesiredManualFanLevel)>,
}

impl<'de> ::serde::Deserialize<'de> for Config {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: ::serde::de::Deserializer<'de> {
		#[derive(Deserialize)]
		struct Inner {
			sensors: ::std::collections::HashMap<String, String>,
			fan_level: ::std::collections::HashMap<String, String>,
		}

		let inner: Inner = ::serde::Deserialize::deserialize(deserializer)?;

		let mut result = Config {
			sensors: Default::default(),
			fan_level: Default::default(),
		};

		for (key, value) in inner.sensors {
			let index = key.parse().map_err(|_| ::serde::de::Error::invalid_value(::serde::de::Unexpected::Str(&key), &"a sensor index"))?;
			if result.sensors.len() < index {
				result.sensors.resize(index, None);
			}
			result.sensors[index - 1] = Some(value);
		}

		for (key, value) in inner.fan_level {
			let temp: f64 = key.parse().map_err(|_| ::serde::de::Error::invalid_value(::serde::de::Unexpected::Str(&key), &"a temperature in degrees Celsius"))?;
			let level = match &*value {
				"0" => ::model::DesiredManualFanLevel::Firmware(::acpi::FanFirmwareLevel::Zero),
				"1" => ::model::DesiredManualFanLevel::Firmware(::acpi::FanFirmwareLevel::One),
				"2" => ::model::DesiredManualFanLevel::Firmware(::acpi::FanFirmwareLevel::Two),
				"3" => ::model::DesiredManualFanLevel::Firmware(::acpi::FanFirmwareLevel::Three),
				"4" => ::model::DesiredManualFanLevel::Firmware(::acpi::FanFirmwareLevel::Four),
				"5" => ::model::DesiredManualFanLevel::Firmware(::acpi::FanFirmwareLevel::Five),
				"6" => ::model::DesiredManualFanLevel::Firmware(::acpi::FanFirmwareLevel::Six),
				"7" => ::model::DesiredManualFanLevel::Firmware(::acpi::FanFirmwareLevel::Seven),
				"full-speed" => ::model::DesiredManualFanLevel::FullSpeed,
				_ => return Err(::serde::de::Error::invalid_value(::serde::de::Unexpected::Str(&value), &"0-7 or full-speed")),
			};

			result.fan_level.push((::acpi::Temp(temp.into()), level));
		}

		result.fan_level.sort_by_key(|&(temp, _)| temp);

		Ok(result)
	}
}

#[derive(Clone, Copy, Debug)]
pub enum VisibleTempSensors {
	All,
	Active,
}

impl Default for VisibleTempSensors {
	fn default() -> Self {
		VisibleTempSensors::Active
	}
}

impl ::std::fmt::Display for VisibleTempSensors {
	fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
		match *self {
			VisibleTempSensors::All => write!(f, "all"),
			VisibleTempSensors::Active => write!(f, "active"),
		}
	}
}

#[derive(Clone, Copy, Debug)]
pub enum DesiredFanMode {
	Bios,
	Smart,
	Manual,
}

impl Default for DesiredFanMode {
	fn default() -> Self {
		DesiredFanMode::Smart
	}
}

impl ::std::fmt::Display for DesiredFanMode {
	fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
		match *self {
			DesiredFanMode::Bios => write!(f, "BIOS"),
			DesiredFanMode::Smart => write!(f, "Smart"),
			DesiredFanMode::Manual => write!(f, "Manual"),
		}
	}
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DesiredManualFanLevel {
	Firmware(::acpi::FanFirmwareLevel),
	FullSpeed,
}

impl Default for DesiredManualFanLevel {
	fn default() -> Self {
		DesiredManualFanLevel::FullSpeed
	}
}

impl ::std::fmt::Display for DesiredManualFanLevel {
	fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
		match *self {
			DesiredManualFanLevel::Firmware(fan_firmware_level) => write!(f, "{}", fan_firmware_level),
			DesiredManualFanLevel::FullSpeed => write!(f, "Full speed"),
		}
	}
}
