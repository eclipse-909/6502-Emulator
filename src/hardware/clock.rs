use {
	crate::hardware::{
		hardware::{Hardware, HardwareSpecs},
		imp::clock_listener::ClockListener,
		cpu::Cpu,
		memory::{Memory, MemEvent, N_WAYS}
	},
	tokio::sync::mpsc::{Sender, Receiver, channel}
};

pub struct Clock {
	pub specs: HardwareSpecs,
	pub cpu: Cpu,
	pub memory: [Memory; N_WAYS as usize]
}

impl Clock {
	pub fn new() -> Self {
		//I know it's MULTIPLE producer single consumer, but it's too late, and I'm lazy
		const ARRAY_REPEAT_VALUE: Option<(Sender<MemEvent>, Receiver<MemEvent>)> = None;
		let mut cpu_to_mem_channels: [Option<(Sender<MemEvent>, Receiver<MemEvent>)>; N_WAYS as usize] = [ARRAY_REPEAT_VALUE; N_WAYS as usize];
		let mut mem_to_cpu_channels: [Option<(Sender<MemEvent>, Receiver<MemEvent>)>; N_WAYS as usize] = [ARRAY_REPEAT_VALUE; N_WAYS as usize];
		for i in 0..N_WAYS as usize {
			let (cpu_tx, cpu_rx) = channel::<MemEvent>(1);
			let (mem_tx, mem_rx) = channel::<MemEvent>(1);
			cpu_to_mem_channels[i] = Some((cpu_tx, mem_rx));
			mem_to_cpu_channels[i] = Some((mem_tx, cpu_rx));
		}
		let clock: Self = Self {
			specs: HardwareSpecs::new("Clock"),
			cpu: Cpu::new(cpu_to_mem_channels.map(|element| {
				match element {
					Some(pair) => {pair}
					None => {panic!("Compile time logical error");}
				}
			})),
			memory: mem_to_cpu_channels.map(|element| {
				match element {
					Some(pair) => {Memory::new(pair.0, pair.1)}
					None => {panic!("Compile time logical error");}
				}
			})
		};
		clock.log("Created");
		clock
	}
}

impl Hardware for Clock {
	fn get_specs(&self) -> &HardwareSpecs {&self.specs}
}

impl ClockListener for Clock {
	///Whether a device is registered is determined by if its pulse function is called here.
	fn pulse(&mut self) {
		self.cpu.pulse();
		self.memory.iter_mut().for_each(|mem| {mem.pulse();});
	}
}