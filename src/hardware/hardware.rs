use crate::lib;

/**Metadata for Hardware objects.*/
pub struct HardwareSpecs {
	pub id: u8,
	pub(crate) name: String,
	pub debug: bool
}

impl HardwareSpecs {
	/**Creates a new instance of HardwareSpecs. Defaults id to 0 and debug to true.*/
	pub fn new(name: &str) -> Self {
		return Self {
			id: 0,
			name: String::from(name),
			debug: true
		};
	}
}

pub trait Hardware {
	/**Returns the HardwareSpecs of this object. This function exists because there's no inheritance in Rust.*/
	fn get_specs(&self) -> &HardwareSpecs;
	/**Logs a message to the console with a specific format with hardware specs. Use this instead of println!() when printing.
	I tried to use impl Into<String> but that's not object safe for traits that impl Hardware*/
	fn log(&self, message: &str) {
		if self.get_specs().debug {
			println!("ID: {} - Name: {} - Time: {:?} - Message: {}", self.get_specs().id, self.get_specs().name, lib::elapsed_ms(), message);
		}
	}
}