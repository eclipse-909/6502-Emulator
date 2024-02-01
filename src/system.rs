use {
	tokio::{
		time::{sleep, Duration},
		sync::Mutex
	},
	std::sync::{
		Arc,
		atomic::AtomicBool
	},
	crate::hardware::{
		clock::Clock,
		hardware::{Hardware, HardwareSpecs}
	}
};

pub struct System {
	specs: HardwareSpecs,
	clock: Clock,
	running: Arc<Mutex<AtomicBool>>//Temporary code - Since I am stopping the server setting running to false in another task, I need a mutex for now
}

impl Hardware for System {
	fn get_specs(&self) -> &HardwareSpecs {return &self.specs;}
	
	fn new() -> Self {
		let system: Self = Self {
			specs: HardwareSpecs::new_default("System"),
			clock: Clock::new(),
			running: Arc::new(Mutex::new(AtomicBool::from(false)))
		};
		system.log("Created");
		return system;
	}
}

impl System {
	const CLOCK_INTERVAL_MICRO: u64 = 100_000;//number is in microseconds or 0.001 milliseconds
	
	pub async fn start_system(&mut self) {
		*self.running.lock().await = AtomicBool::from(true);
		self.clock.memory.display_memory(0x0000, 0x0014);
		//Temporary code
		let clone_running: Arc<Mutex<AtomicBool>> = Arc::clone(&self.running);
		tokio::spawn(async move {
			tokio::time::sleep(Duration::from_secs(1)).await;//for now, I will just stop the system after 1 second has passed
			*clone_running.lock().await = AtomicBool::from(false);
		});
		//Temporary code
		loop {
			if !*self.running.lock().await.get_mut() {return;}
			self.clock.invoke();
			sleep(Duration::from_micros(Self::CLOCK_INTERVAL_MICRO)).await;
		}
	}
	
	async fn stop_system(&mut self) {
		*self.running.lock().await = AtomicBool::from(false);
	}
}