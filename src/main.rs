use crate::{
	hardware::hardware::Hardware,
	system::System
};

mod system;
mod hardware;

fn main() {
	let start_address: u16 = 0x0000;
	//Bubble-sort program, 245 bytes long
	let sort_program: &[u8] = &[
		0xA2, 0x03, 0xFF, 0xD6, 0x00, 0xAD, 0xEB, 0x00, 0xA2, 0x01, 0xAC, 0xEC, 0x00, 0xFF, 0xA2, 0x03, 0xFF, 0xE8, 0x00, 0xEE, 0x0B, 0x00, 0x6D, 0xD5, 0x00, 0xD0, 0xED, 0xAD, 0xEB, 0x00, 0x6D, 0xD5, 0x00, 0x8D, 0xD2, 0x00, 0xAE, 0xD2, 0x00, 0xEC, 0xCF, 0x00, 0xD0, 0x1C, 0xA2, 0x03, 0xFF, 0xDD, 0x00, 0xAD, 0xEB, 0x00, 0xA2, 0x01, 0xAC, 0xEC, 0x00, 0xFF, 0xA2, 0x03, 0xFF, 0xE8, 0x00, 0xEE, 0x37, 0x00, 0x6D, 0xD5, 0x00, 0xD0, 0xED, 0x00, 0xA9, 0x00, 0x8D, 0xD3, 0x00, 0x8D, 0xD1, 0x00, 0xAD, 0xD2, 0x00, 0x6D, 0xD0, 0x00, 0xAA, 0xEC, 0xD1, 0x00, 0xD0, 0x0C, 0xAD, 0xD3, 0x00, 0x6D, 0x47, 0x00, 0xD0, 0x5D, 0xA2, 0x03, 0xD0, 0xC6, 0xAD, 0x37, 0x00, 0x6D, 0xD1, 0x00, 0x8D, 0x78, 0x00, 0x8D, 0x9C, 0x00, 0x8D, 0xA6, 0x00, 0xAE, 0xEC, 0x00, 0x6D, 0xD4, 0x00, 0x8D, 0x8A, 0x00, 0x8D, 0x96, 0x00, 0x8D, 0x9F, 0x00, 0x8D, 0xA2, 0x00, 0xEC, 0xEC, 0x00, 0xD0, 0x07, 0xA8, 0xA2, 0x01, 0xD0, 0x1A, 0xD0, 0x8F, 0xAC, 0xEC, 0x00, 0x98, 0xD0, 0x17, 0xAD, 0xEC, 0x00, 0xAC, 0xEC, 0x00, 0x8D, 0xEC, 0x00, 0x98, 0x8D, 0xEC, 0x00, 0xA9, 0x01, 0x8D, 0xD3, 0x00, 0xEE, 0xD1, 0x00, 0xD0, 0x9E, 0x6D, 0xD5, 0x00, 0xA8, 0x8A, 0x6D, 0xD5, 0x00, 0xAA, 0xD0, 0xDB, 0xA2, 0x01, 0xD0, 0xEC, 0xAD, 0xD0, 0x00, 0x6D, 0xD5, 0x00, 0x8D, 0xD0, 0x00, 0xEE, 0xCF, 0x00, 0xD0, 0xC4, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0xFF, 0x49, 0x6E, 0x69, 0x74, 0x20, 0x20, 0x00, 0x20, 0x20, 0x53, 0x6F, 0x72, 0x74, 0x65, 0x64, 0x20, 0x20, 0x00, 0x2C, 0x20, 0x00, 0x0A, 0x08, 0x03, 0x01, 0x09, 0x07, 0x05, 0x02, 0x0A, 0x04, 0x06
	];
	
	let _ = lib::elapsed_ms();//initializes the timer to get the elapsed time
	let mut system: System = System::new();
	
	system.clock.get_specs_mut().debug = false;
	system.clock.cpu.get_specs_mut().debug = false;
	system.clock.memory.specs.debug = false;
	
	system.load_main_program(start_address, sort_program);
	system.start();
}

mod lib {
	use std::time::Instant;
	
	static mut START_TIME: Option<Instant> = None;
	
	/**Gets the elapsed ms since the program started.*/
	pub fn elapsed_ms() -> u128 {
		unsafe {
			if START_TIME.is_none() { START_TIME = Some(Instant::now()); }
			return Instant::now().duration_since(START_TIME.unwrap()).as_millis();
		}
	}
	/**Converts a u16 to a (u8,u8) tuple in little-endian format.*/
	pub fn u16_to_little_endian(value: u16) -> (u8, u8) {
		let byte1: u8 = (value & 0xFF) as u8;
		let byte2: u8 = ((value >> 8) & 0xFFu16) as u8;
		return (byte1, byte2);
	}
}

/*
=======================================================================================================================================
	ATTENTION READER:
	
	The code below is a little old. I have since made my own text-editor/assembler to make coding in assembly a little easier.
	I have changed the code slightly, but I am preserving the stuff down below for an arbitrary reason.
	The MAIN_PROGRAM array above is still used to load the program into memory, and it's hex will be different from the hex
	in the table below.
=======================================================================================================================================
*/

/* Program Description:
	This is a sorting algorithm that will take an array in memory and do a bubble sort in place.
	It will print the array before and after sorting.
*/
/* C code:
    int i, j, neg_i = 0;
    bool swapped;
    for (i = 0; i < n; i++) {
        swapped = false;
        for (j = 0; j < n + neg_i; j++) {
            if (arr[j] > arr[j + 1]) {
                swap(&arr[j], &arr[j + 1]); //swap function not shown
                swapped = true;
            }
        }
 
        // If no two elements were swapped by inner loop, then break
        if (swapped == false)
            break;
            
        neg_i--;
    }
*/
/*ASM Pseudocode:
	program:
						;Print the original array. Implementation not shown
					load array_size into A                  ;initialize n
					add dec to A
					store A to n
		outer:      load n into X                           ;outer loop condition
					compare i with X
					if !0 then branch to outer_body         ;if i != n
		            load 1 into A                           ;jump to end of program
					branch to sorted
		outer_body: load 0 into A                           ;swapped = false
					store A into swapped
		inner:      load n into A                           ;if j != n + neg_i
					add neg_i to A
					transfer A to X
					compare j with X
					if !0 then branch to inner_body
					load 1 into A
					branch to post_inner
		inner_body: load low_j_a into A
					add j to A
					store A to low_j_a
					store A to low_j_b
					store A to low_j_c
					load the following address into X       ;array[j] into X
		low_j_a:        low byte of array address
						high byte of array address
					add inc to A
					store A to low_j1_a
					store A to low_j1_b
					store A to low_j1_c
					store A to low_j1_d
					compare X with the following address    ;if array[j] == array[j+1], I can skip the array[j] > array[j+1] check and continue in the inner loop
		low_j1_a:       low byte of array address
						high byte of array address
					if !0 branch to if_eq
					transfer A to Y
					load 1 into A
					branch to in_cont
		if_eq:	    load the following address into Y       ;array[j+1] into Y
		low_j1_b:       low byte of array address
						high byte of array address
		ineq_loop:	transfer X to A                         ;if array[j] > array[j+1]
					add dec to A                            ;decrement X and Y until the first one reaches 0
					if !0 then branch to check_y
					load 1 into A
					branch to swapping
		check_y:    transfer A to X
					transfer Y to A
					add dec to A
					transfer A to Y
					if !0 then branch to ineq_loop
					load 1 into A
					branch to in_cont
		swapping:   load the following address into A       ;array[j] into A
		low_j_b:        low byte of array address
						high byte of array address
					load the following address into Y       ;array[j+1] into Y
		low_j1_c:       low byte of array address
						high byte of array address
					store A to the following address        ;A into array[j+1]
		low_j1_d:       low byte of array address
						high byte of array address
					transfer Y to A
					store A to the following address        ;Y to A into array[j]
		low_j_c:        low byte of array address
						high byte of array address
					load 1 into A                           ;swapped = true
					store A into swapped
		in_cont:    increment j                             ;end of inner loop
					load 1 into A
					branch to inner
		post_inner: load swapped into A                     ;check swapped
					if !0 then branch to out_cont
					load 1 into A
					branch to sorted                        ;if nothing was swapped, then jump to end of program
		out_cont:   load neg_i into A                       ;end of outer loop
					add dec to A                            ;neg_i-- and i++ then loop to outer
					increment i
					branch to outer
		sorted:         ;implementation not shown. represents the end of the program, and prints the array
		
	data:
		i = 0
		neg_i = 0
		j = 0
		swapped = 0
		n = array_size - 1  ;initialized at runtime
		dec = 0xff
		inc = 1
		array_size = 10
		array = {4, 9, 7, 6, 8, 10, 3, 5, 1, 2}
*/
/* Code: Making this table cost me my sanity, so I made a text-editor-assembler
	Address | Label                 | Assembly            | Hex Dump | Comments
	------------------------------------------------------------------------------------------------------------------------
	0x0000  |                       | LDX #$03            | A2 03    | Print "Initial Array  \0"
	0x0002  |                       | SYS init_array_dscr | FF D9 00 |
	0x0005  |                       | LDA array_size      | AD FC 00 | Print the values of the array
	0x0008  | init_print_loop       | LDX #$01            | A2 01    |
	0x000a  |                       | LDY                 | AC       |
	0x000b  | init_index            | 00                  | 00       |
	0x000c  |                       | 01                  | 01       |
	0x000d  |                       | SYS                 | FF       |
	0x000e  |                       | LDX #$03            | A2 03    |
	0x0010  |                       | SYS delimiter       | FF F9 00 |
	0x0013  |                       | INC init_index      | EE 0B 00 |
	0x0016  |                       | ADC decrement       | 6D D8 00 |
	0x0019  |                       | BNE init_print_loop | D0 ED    | Branch back to the start of the array
	0x001b  |                       | LDA array_size      | AD FC 00 | Sort the array in place using bubble sort
	0x001e  |                       | ADC decrement       | 6D D8 00 | initialize n
	0x0021  |                       | STA n               | 8D D5 00 |
	0x0024  | outer                 | LDX n               | AE D5 00 | outer loop condition
	0x0027  |                       | CPX i               | EC D2 00 |
	0x002a  |                       | BNE outer_body      | D0 04    | if i != n
	0x002c  |                       | LDA #$01            | A9 01    | jump to end of program
	0x002e  |                       | BNE sorted          | D0 86    |
	0x0030  | outer_body            | LDA #$00            | A9 00    | swapped = false
	0x0032  |                       | STA swapped         | 8D D6 00 |
	0x0035  | inner                 | LDA n               | AD D5 00 | if j != n + neg_i
	0x0038  |                       | ADC neg_i           | 6D D3 00 |
	0x003b  |                       | TAX                 | AA       |
	0x003c  |                       | CPX j               | EC D4 00 |
	0x003f  |                       | BNE inner_body      | D0 04    |
	0x0041  |                       | LDA #$01            | A9 01    |
	0x0043  |                       | BNE post_inner      | D0 5D    |
	0x0045  | inner_body            | LDA low_j_a         | AD 55 00 |
	0x0048  |                       | ADC j               | 6D D4 00 |
	0x004b  |                       | STA low_j_a         | 8D 55 00 |
	0x004e  |                       | STA low_j_b         | 8D 8A 00 |
	0x0051  |                       | STA low_j_c         | 8D 94 00 |
	0x0054  |                       | LDX                 | AE       | array[j] into X
	0x0055  | low_j_a               | 00                  | 00       |
	0x0056  |                       | 01                  | 01       |
	0x0057  |                       | ADC increment       | 6D D7 00 |
	0x005a  |                       | STA low_j1_a        | 8D 67 00 |
	0x005d  |                       | STA low_j1_b        | 8D 71 00 |
	0x0060  |                       | STA low_j1_c        | 8D 8D 00 |
	0x0063  |                       | STA low_j1_d        | 8D 90 00 |
	0x0066  |                       | CPX                 | EC       | if array[j] == array[j+1], I can skip the array[j] > array[j+1] check and continue in the inner loop
	0x0067  | low_j1_a              | 00                  | 00       |
	0x0068  |                       | 01                  | 01       |
	0x0069  |                       | BNE if_eq           | D0 05    |
	0x006b  |                       | TAY                 | A8       |
	0x006c  |                       | LDA #$01            | A9 01    |
	0x006e  |                       | BNE in_cont         | D0 2B    |
	0x0070  | if_eq                 | LDY                 | AC       | array[j+1] into Y
	0x0071  | low_j1_b              | 00                  | 00       |
	0x0072  |                       | 01                  | 01       |
	0x0073  | ineq_loop             | TXA                 | 8A       | if array[j] > array[j+1]
	0x0074  |                       | ADC decrement       | 6D D8 00 | decrement X and Y until the first one reaches 0
	0x0077  |                       | BNE check_y         | D0 04    |
	0x0079  |                       | LDA #$01            | A9 01    |
	0x007b  |                       | BNE swapping        | D0 0C    |
	0x007d  | check_y               | TAX                 | AA       |
	0x007e  |                       | TYA                 | 98       |
	0x007f  |                       | ADC decrement       | 6D D8 00 |
	0x0082  |                       | TAY                 | A8       |
	0x0083  |                       | BNE ineq_loop       | D0 EE    |
	0x0085  |                       | LDA #$01            | A9 01    |
	0x0087  |                       | BNE in_cont         | D0 12    |
	0x0089  | swapping              | LDA                 | AD       | array[j] into A
	0x008a  | low_j_b               | 00                  | 00       |
	0x008b  |                       | 01                  | 01       |
	0x008c  |                       | LDY                 | AC       | array[j+1] into Y
	0x008d  | low_j1_c              | 00                  | 00       |
	0x008e  |                       | 01                  | 01       |
	0x008f  |                       | STA                 | 8D       |
	0x0090  | low_j1_d              | 00                  | 00       |
	0x0091  |                       | 01                  | 01       |
	0x0092  |                       | TYA                 | 98       |
	0x0093  |                       | STA                 | 8D       | Y to A into array[j]
	0x0094  | low_j_c               | 00                  | 00       |
	0x0095  |                       | 01                  | 01       |
	0x0096  |                       | LDA #$01            | A9 01    | swapped = true
	0x0098  |                       | STA swapped         | 8D D6 00 |
	0x009b  | in_cont               | INC j               | EE D4 00 | end of inner loop
	0x009e  |                       | LDA #$01            | A9 01    |
	0x00a0  |                       | BNE inner           | D0 93    |
	0x00a2  | post_inner            | LDA swapped         | AD D6 00 | check swapped
	0x00a5  |                       | BNE out_cont        | D0 04    |
	0x00a7  |                       | LDA #$01            | A9 01    |
	0x00a9  |                       | BNE sorted          | D0 0B    | if nothing was swapped, then jump to end of program
	0x00ab  | out_cont              | LDA neg_i           | AD D3 00 | end of outer loop
	0x00ae  |                       | ADC decrement       | 6D D8 00 | neg_i-- and i++ then loop to outer
	0x00b1  |                       | INC i               | EE D2 00 |
	0x00b4  |                       | BNE outer           | D0 6E    |
	0x00b6  | sorted                | LDX #$03            | A2 03    | Print "Sorted Array   \0"
	0x00b8  |                       | SYS sort_array_dscr | FF E9 00 |
	0x00bb  |                       | LDA array_size      | AD FC 00 | Print the values of the array
	0x00be  | sort_print_loop       | LDX #$01            | A2 01    |
	0x00c0  |                       | LDY                 | AC       |
	0x00c1  | sort_index            | 00                  | 00       |
	0x00c2  |                       | 01                  | 01       |
	0x00c3  |                       | SYS                 | FF       |
	0x00c4  |                       | LDX #$03            | A2 03    |
	0x00c6  |                       | SYS delimiter       | FF F9 00 |
	0x00c9  |                       | INC sort_index      | EE B9 00 |
	0x00cc  |                       | ADC decrement       | 6D D8 00 |
	0x00cf  |                       | BNE sort_print_loop | D0 ED    | Branch back to the start of the array
	0x00d1  |                       | BRK                 | 00       |
	0x00d2  | i                     | DAT 00              | 00       |
	0x00d3  | neg_i                 | DAT 00              | 00       |
	0x00d4  | j                     | DAT 00              | 00       |
	0x00d5  | n                     | DAT 00              | 00       |
	0x00d6  | swapped               | DAT 00              | 00       |
	0x00d7  | increment             | DAT 01              | 01       | Used to increment the accumulator
	0x00d8  | decrement             | DAT ff              | FF       | Used to decrement the accumulator
	0x00d9  | init_array_dscr       | DAT __              | 49 6E 69 74 69 61 6C 20 41 72 72 61 79 20 20 00
																	 | "Initial Array  \0"
	0x00e9  | sort_array_dscr       | DAT __              | 53 6F 72 74 65 64 20 41 72 72 61 79 20 20 20 00
																	 | "Sorted Array   \0"
	0x00f9  | delimiter             | DAT 2c 20 00        | 2C 20 00 | Array elements are comma-separated ", \0"
	0x00fc  | array_size            | DAT 0a              | 0A       | The size of the following array
	0x00fd  |                       | DAT 00 00 00        | 00 00 00 | Buffer to put array entirely on the next page
	0x0100  | array                 | DAT __              | 04 09 07 06 08 0A 03 05 01 02
																	 | The first element of the array
*/