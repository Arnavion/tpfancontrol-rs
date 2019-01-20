pub(crate) enum Error {
	Acpi(std::path::PathBuf, std::io::Error),
	Config(std::io::Error),
	Enxio,
}

impl std::fmt::Debug for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Error::Acpi(path, err) => write!(f, "sysfs error with {}: {}", path.display(), err),
			Error::Config(err) => write!(f, "could not parse config file: {}", err),
			Error::Enxio => write!(f, "sysfs error: ENXIO"),
		}
	}
}
