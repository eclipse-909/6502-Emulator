use std::time::Instant;

pub struct HardwareSpecs {
	id: u8,
	name: String,
	debug: bool
}

static mut START_TIME: Option<Instant> = None;

impl HardwareSpecs {
	/**Creates a new instance of HardwareSpecs. Defaults id to 0 and debug to true.*/
	pub fn new_default(name: &str) -> Self {
		return Self {
			id: 0,
			name: String::from(name),
			debug: true
		};
	}
}

pub trait Hardware {
	fn get_specs(&self) -> &HardwareSpecs;
	
	/**Creates a new instance and outputs "Created" to the log.*/
	fn new() -> Self;
	
	/**Gets the elapsed ms since the program started.*/
	fn elapsed_ms() -> u128 {
		unsafe {
			if START_TIME.is_none() {START_TIME = Some(Instant::now());}
			return Instant::now().duration_since(START_TIME.unwrap()).as_millis();
		}
	}
	
	/**Logs a message to the console with a specific format with hardware specs. Use this instead of println!() when printing.*/
	fn log(&self, message: &str) {
		if self.get_specs().debug {
			println!("ID: {} - Name: {} - Time: {:?} - Message: {}", self.get_specs().id, self.get_specs().name, Self::elapsed_ms(), message);
		}
	}
}