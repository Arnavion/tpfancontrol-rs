pub fn read_temps(temps: &mut [Option<Temp>]) -> ::Result<()> {
	for (i, out) in temps.iter_mut().enumerate() {
		let path = HWMON_PATH.join(format!("temp{}_input", i + 1));
		match read_line(&path) {
			Ok(temp) => *out = Some(Temp(((temp as f64) / 1000.).into())),

			Err(::Error(::ErrorKind::Io(err), _)) => if let Some(::libc::ENXIO) = err.raw_os_error() {
				*out = None;
			}
			else {
				return Err(err.into());
			},

			Err(err) => return Err(err.into()),
		}
	}

	Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Temp(pub ::ordered_float::NotNaN<f64>);

impl Temp {
	pub fn display(self, scale: TempScale) -> TempDisplay {
		TempDisplay::new(*self.0, scale)
	}
}

#[derive(Clone, Copy, Debug)]
pub enum TempScale {
	Celsius,
	Fahrenheit,
}

impl Default for TempScale {
	fn default() -> Self {
		TempScale::Celsius
	}
}

impl ::std::fmt::Display for TempScale {
	fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
		match *self {
			TempScale::Celsius => write!(f, "\u{B0}C"),
			TempScale::Fahrenheit => write!(f, "\u{B0}F"),
		}
	}
}

#[derive(Clone, Copy, Debug)]
pub struct TempDisplay(f64, TempScale);

impl TempDisplay {
	fn new(temp: f64, scale: TempScale) -> Self {
		let temp = match scale {
			TempScale::Celsius => temp,
			TempScale::Fahrenheit => temp * 9.0 / 5.0 + 32.0,
		};

		TempDisplay(temp.round(), scale)
	}
}

impl ::std::fmt::Display for TempDisplay {
	fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
		write!(f, "{} {}", self.0, self.1)
	}
}

pub fn read_fan() -> ::Result<(FanLevel, FanSpeed)> {
	let pwm_mode = read_line(&PWM_ENABLE_PATH)?;
	let level = match pwm_mode {
		2 => FanLevel::Auto,
		1 => {
			let hwmon_level = read_line(&PWM_PATH)?;
			FanLevel::Firmware(FanFirmwareLevel::from_hwmon_level(hwmon_level).ok_or_else(|| format!("unrecognized hwmon level {}", hwmon_level))?)
		},
		0 => FanLevel::FullSpeed,
		level => return Err(format!("unrecognized PWM mode {}", level).into()),
	};

	let speed = FanSpeed(read_line(&FAN_INPUT_PATH)?);

	Ok((level, speed))
}

pub fn fan_is_writable(update_interval: ::std::time::Duration) -> ::Result<bool> {
	use ::std::io::Write;

	match ::std::fs::File::create(&*FAN_WATCHDOG_PATH) {
		Ok(mut file) => {
			write!(&mut file, "{}", update_interval.as_secs() * 2)?;
			Ok(true)
		},

		Err(err) => if let ::std::io::ErrorKind::PermissionDenied = err.kind() {
			Ok(false)
		}
		else {
			Err(err.into())
		},
	}
}

pub fn write_fan(fan_level: FanLevel) -> ::Result<()> {
	use ::std::io::Write;

	match fan_level {
		FanLevel::Auto => {
			let mut file = ::std::fs::File::create(&*PWM_ENABLE_PATH)?;
			write!(file, "2")?;
		},

		FanLevel::Firmware(fan_firmware_level) => {
			{
				let mut file = ::std::fs::File::create(&*PWM_ENABLE_PATH)?;
				write!(file, "1")?;
			}

			{
				let mut file = ::std::fs::File::create(&*PWM_PATH)?;
				write!(file, "{}", fan_firmware_level.to_hwmon_level())?;
			}
		},

		FanLevel::FullSpeed => {
			let mut file = ::std::fs::File::create(&*PWM_ENABLE_PATH)?;
			write!(file, "0")?;
		},
	}

	Ok(())
}

#[derive(Copy, Clone, Debug)]
pub enum FanLevel {
	Auto,
	Firmware(FanFirmwareLevel),
	FullSpeed,
}

impl ::std::fmt::Display for FanLevel {
	fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
		match *self {
			FanLevel::Auto => write!(f, "Auto"),
			FanLevel::Firmware(level) => write!(f, "{}", level),
			FanLevel::FullSpeed => write!(f, "Full speed"),
		}
	}
}

#[derive(Clone, Copy, Debug)]
pub struct FanSpeed(u32);

impl ::std::fmt::Display for FanSpeed {
	fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
		write!(f, "{} RPM", self.0)
	}
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FanFirmwareLevel {
	Zero = 0,
	One = 36,
	Two = 72,
	Three = 109,
	Four = 145,
	Five = 182,
	Six = 218,
	Seven = 255,
}

impl FanFirmwareLevel {
	fn from_hwmon_level(hwmon_level: u32) -> Option<FanFirmwareLevel> {
		match hwmon_level {
			hwmon_level if hwmon_level == FanFirmwareLevel::Zero as u32 => Some(FanFirmwareLevel::Zero),
			hwmon_level if hwmon_level == FanFirmwareLevel::One as u32 => Some(FanFirmwareLevel::One),
			hwmon_level if hwmon_level == FanFirmwareLevel::Two as u32 => Some(FanFirmwareLevel::Two),
			hwmon_level if hwmon_level == FanFirmwareLevel::Three as u32 => Some(FanFirmwareLevel::Three),
			hwmon_level if hwmon_level == FanFirmwareLevel::Four as u32 => Some(FanFirmwareLevel::Four),
			hwmon_level if hwmon_level == FanFirmwareLevel::Five as u32 => Some(FanFirmwareLevel::Five),
			hwmon_level if hwmon_level == FanFirmwareLevel::Six as u32 => Some(FanFirmwareLevel::Six),
			hwmon_level if hwmon_level == FanFirmwareLevel::Seven as u32 => Some(FanFirmwareLevel::Seven),
			_ => None,
		}
	}

	fn to_hwmon_level(self) -> u32 {
		self as u32
	}
}

impl ::std::fmt::Display for FanFirmwareLevel {
	fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
		match *self {
			FanFirmwareLevel::Zero => write!(f, "0"),
			FanFirmwareLevel::One => write!(f, "1"),
			FanFirmwareLevel::Two => write!(f, "2"),
			FanFirmwareLevel::Three => write!(f, "3"),
			FanFirmwareLevel::Four => write!(f, "4"),
			FanFirmwareLevel::Five => write!(f, "5"),
			FanFirmwareLevel::Six => write!(f, "6"),
			FanFirmwareLevel::Seven => write!(f, "7"),
		}
	}
}

lazy_static! {
	/// Path to the root of the hardware monitoring sysfs interface provided by the thinkpad-acpi kernel module
	static ref HWMON_PATH: ::std::path::PathBuf = {
		for dir_entry in ::std::fs::read_dir("/sys/class/hwmon").unwrap() {
			if let Ok(dir_entry) = dir_entry {
				let dir_path = dir_entry.path();
				if let Ok(mut name_file) = ::std::fs::File::open(dir_path.join("name")) {
					let mut name = String::new();
					if let Ok(_) = ::std::io::Read::read_to_string(&mut name_file, &mut name) {
						if name == "thinkpad\n" {
							return dir_path;
						}
					}
				}
			}
		}

		panic!("could not find hwmon device for thinkpad_acpi");
	};

	/// Path of the file with the fan speed
	static ref FAN_INPUT_PATH: ::std::path::PathBuf = HWMON_PATH.join("fan1_input");

	/// Path of the fan watchdog file
	static ref FAN_WATCHDOG_PATH: ::std::path::PathBuf = HWMON_PATH.join("device").join("driver").join("fan_watchdog");

	/// Path of the file with the pwm mode
	static ref PWM_ENABLE_PATH: ::std::path::PathBuf = HWMON_PATH.join("pwm1_enable");

	/// Path of the file with the fan level
	static ref PWM_PATH: ::std::path::PathBuf = HWMON_PATH.join("pwm1");
}

fn read_line(path: &::std::path::Path) -> ::Result<u32> {
	let file = ::std::fs::File::open(path)?;
	let file = ::std::io::BufReader::new(file);

	Ok(match ::std::io::BufRead::lines(file).next() {
		Some(Ok(line)) => line.parse()?,
		Some(Err(err)) => Err(err)?,
		None => Err("empty file")?,
	})
}
