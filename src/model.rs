#[derive(Debug)]
pub struct State {
	pub temps: ::Result<Vec<Option<::acpi::Temp>>>,
	pub visible_temp_sensors: VisibleTempSensors,
	pub temp_scale: ::acpi::TempScale,

	pub fan_is_writable: bool,

	pub fan: ::Result<(::acpi::FanLevel, ::acpi::FanSpeed)>,
	pub desired_fan_mode: DesiredFanMode,
	pub desired_manual_fan_level: DesiredManualFanLevel,
}

impl State {
	pub fn new(num_temp_sensors: usize, fan_update_interval: ::std::time::Duration) -> ::Result<Self> {
		let mut temps = vec![None; num_temp_sensors];
		let temps = ::acpi::read_temps(&mut temps).map(|()| temps);

		Ok(State {
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
