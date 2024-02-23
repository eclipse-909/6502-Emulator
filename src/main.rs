use crate::{
	hardware::hardware::Hardware,
	system::System
};

mod system;
mod hardware;

fn main() {
	let _ = System::elapsed_ms();//initializes the timer to get the elapsed time
	let mut system: System = System::new();
	system.load_main_program(MAIN_ADDR, MAIN_PROGRAM);
	system.start_system();
}

pub const MAIN_ADDR: u16 = 0x0300;
/**
main:

	Program:
	 * load 3 into X
     * syscall - with address of string
     * break
     * define string in memory here

	Address   | Assembly            | Hex Dump
	------------------------------------------------------------------------
	0x0300    |   LDX #$03          | A2 03
	0x0302    |   SYS $0306         | FF 06 03
	0x0305    |   BRK               | 00
	0x0306    |   ;)                | 48 65 6C 6C 6F 20 57 6F 72 6C 64 21 00
*/
const MAIN_PROGRAM: &[u8] = &[
	0xA2, 0x03,
	0xFF, 0x06, 0x03,
	00,
	0x48, 0x65, 0x6C, 0x6C, 0x6F, 0x20, 0x57, 0x6F, 0x72, 0x6C, 0x64, 0x21, 0x00
];