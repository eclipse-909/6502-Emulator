use {
	crate::{
		ascii::ascii,
		hardware::{
			hardware::{Hardware, HardwareSpecs},
			imp::interrupt::{Interrupt, InterruptSpecs}
		}
	},
	crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
	std::{
		sync::{
			Arc,
			atomic::{AtomicBool, AtomicU8, Ordering}
		},
		time::Duration
	},
	tokio::sync::mpsc::UnboundedSender
};

pub struct Keyboard {
	hard_specs: HardwareSpecs,
	int_specs: InterruptSpecs,
	pub out_buf: Arc<AtomicU8>
}

impl Hardware for Keyboard {
	fn get_specs(&self) -> &HardwareSpecs {return &self.hard_specs;}
	fn get_specs_mut(&mut self) -> &mut HardwareSpecs {return &mut self.hard_specs;}
}

impl Interrupt for Keyboard {
	fn get_out_buf(&self) -> u8 {return self.out_buf.load(Ordering::Relaxed);}
	fn get_interrupt_specs(&self) -> &InterruptSpecs {return &self.int_specs;}
}

impl Keyboard {
	pub fn new(tx: UnboundedSender<InterruptSpecs>, running: Arc<AtomicBool>) -> Self {
		let keyboard: Self = Self {
			hard_specs: HardwareSpecs::new("Keyboard"),
			int_specs: InterruptSpecs::new(0, 0, "Keyboard"),
			out_buf: Arc::new(AtomicU8::new(0x00))
		};
		keyboard.log("Created");
		
		//make an async task to listen for keyboard input
		let specs: InterruptSpecs = keyboard.int_specs.clone();
		let out_buf: Arc<AtomicU8> = keyboard.out_buf.clone();
		tokio::spawn(async move {
			//keeps running until the owning InterruptController is dropped
			while running.load(Ordering::Relaxed) {
				if event::poll(Duration::from_secs(0)).unwrap() {
					if let Ok(Event::Key(KeyEvent{code: KeyCode::Char(c), modifiers: _, kind: KeyEventKind::Press, state: _ })) = event::read() {
						out_buf.store(ascii::DECODER.get(&c).unwrap_or(&0x00).clone(), Ordering::Relaxed);
						if let Err(_) = tx.send(specs.clone()) {
							break;
						}
					}
				}
			}
		});
		
		keyboard
	}
}