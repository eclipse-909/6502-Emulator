use crate::{
	hardware::{
		hardware::Hardware
	},
	system::System
};

mod system;
mod hardware;

#[tokio::main]
async fn main() {
	System::elapsed_ms();//initializes the timer to get the elapsed time
	let mut system: System = System::new();
	system.start_system().await;
}