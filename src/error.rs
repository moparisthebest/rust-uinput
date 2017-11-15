use std::fmt;
use std::error;
use std::ffi;
use std::io;
use nix;

#[cfg(feature = "udev")]
use udev;

/// UInput error.
#[derive(Debug)]
pub enum Error {
	/// System errors.
	Nix(nix::Error),

	/// Errors with internal nulls in names.
	Nul(ffi::NulError),

	#[cfg(feature = "udev")]
	/// Errors coming from udev.
	Udev(udev::Error),

	Io(io::Error),

	/// The uinput file could not be found.
	NotFound,

	/// error reading input_event
	ShortRead,
}

impl From<ffi::NulError> for Error {
	fn from(value: ffi::NulError) -> Self {
		Error::Nul(value)
	}
}

impl From<nix::Error> for Error {
	fn from(value: nix::Error) -> Self {
		Error::Nix(value)
	}
}

#[cfg(feature = "udev")]
impl From<udev::Error> for Error {
	fn from(value: udev::Error) -> Self {
		Error::Udev(value)
	}
}

impl From<io::Error> for Error {
	fn from(value: io::Error) -> Self {
		Error::Io(value)
	}
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		f.write_str(error::Error::description(self))
	}
}

impl error::Error for Error {
	fn description(&self) -> &str {
		match self {
			&Error::Nix(ref err) =>
				err.description(),

			&Error::Nul(ref err) =>
				err.description(),

			#[cfg(feature = "udev")]
			&Error::Udev(ref err) =>
				err.description(),

			&Error::Io(ref err) =>
				err.description(),

			&Error::NotFound =>
				"Device not found.",

			&Error::ShortRead =>
				"Error while reading from device file.",
		}
	}
}
