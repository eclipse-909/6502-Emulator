use {
	crate::hardware::{
		imp::interrupt::{Interrupt, InterruptSpecs},
		keyboard::Keyboard
	},
	std::{
		collections::{BinaryHeap, HashMap},
		sync::{
			Arc,
			atomic::{AtomicBool, Ordering}
		}
	},
	tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver}
};

/**The io_rx receives InterruptSpecs from any I/O device "connected". At the end of every instruction cycle,
the CPU gathers all the received InterruptSpecs from the io_rx and pushes them into the priority queue if any exist.
Immediately after, the CPU pops an InterruptSpecs from the priority queue, uses its IQR to lookup the Interrupt device in the io_devices HashMap,
and reads the value in that devices output buffer.*/
pub struct InterruptController {
	pub priority_queue: BinaryHeap<InterruptSpecs>,
	pub io_devices: HashMap<u8, Box<dyn Interrupt>>,
	pub io_rx: UnboundedReceiver<InterruptSpecs>,
	running: Arc<AtomicBool>
}

impl InterruptController {
	pub fn new() -> Self {
		let mut map: HashMap<u8, Box<dyn Interrupt>> = HashMap::new();
		let (tx, rx) = unbounded_channel::<InterruptSpecs>();
		let running: Arc<AtomicBool> = Arc::new(AtomicBool::new(true));
		let keyboard: Box<Keyboard> = Box::new(Keyboard::new(tx.clone(), running.clone()));
		map.insert(keyboard.get_interrupt_specs().iqr, keyboard);
		Self {
			priority_queue: BinaryHeap::new(),
			io_devices: map,
			io_rx: rx,
			running
		}
	}
}

//RAII - stops dependent tokio threads gracefully on destruction
impl Drop for InterruptController {
	fn drop(&mut self) {self.running.store(false, Ordering::Relaxed);}
}