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
	system.load_program(PRINT_STR_ADDR, PRINT_STR_PROGRAM);
	system.start_system();
}

pub const MAIN_ADDR: u16 = 0x0300;
/**
main:

	Program:
	 * load 1 into X
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

pub const PRINT_STR_ADDR: u16 = 0x0200;
/**
print_str:

	Assume that the address of the string pointer is already stored in 0x00 and 0x01,
	and assume that the calling function's address was pushed onto the stack.
	
	Program:
	 * load the X register with the immediate value 0
	 * start loop here: transfer the value of X into Y
	 * load into the accumulator the value pointed to by the address stored in 0x00 and 0x01, using the Y register as an offset
	 * if the value in the accumulator is 0, then return
	 * transfer the value in the accumulator into Y
	 * transfer the value of X into the accumulator
	 * load X with the immediate value of 2
	 * syscall - this is simply just the opcode 0xFF
	 * transfer the value in the accumulator to X
	 * increment X
	 * jump back to the start of the loop
	
	Address   | Assembly            | Hex Dump
	------------------------------------------
	0x0200    | LDX #$00            | A2 00
		      | Loop:               |
	0x0202    |   TXA               | 8A
	0x0203    |   TAY               | A8
	0x0204    |   LDA ($00),Y       | B1 00
	0x0206    |   BEQ EndLoop       | F0 0A
	0x0208    |   TAY               | A8
	0x0209    |   TXA               | 8A
	0x020A    |   LDX #$02          | A2 02
	0x020C    |   SYS               | FF
	0x020D    |   TAX               | AA
	0x020E    |   INX               | E8
	0x020F    |   JMP Loop          | 4C 02 02
		      | EndLoop:            |
	0x0212    |   RTS               | 60
*/
const PRINT_STR_PROGRAM: &[u8] = &[
	0xA2, 0x00,
	0x8A,
	0xA8,
	0xB1, 0x00,
	0xF0, 0x0A,
	0xA8,
	0x8A,
	0xA2, 0x02,
	0xFF,
	0xAA,
	0xE8,
	0x4C, 0x02, 0x02,
	0x60
];