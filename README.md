See README.md in master branch for details about the project.
# Notes for Final Project
### Clock Time Interval
* Between pulses, I tell the thread to sleep for 0 seconds, and I call the sleep
function every 30 clock pulses. The program only takes a few seconds
to finish. If you want to slow it down, you can increase the clock interval or increase
the number of times sleep is called. Sleeping for 0 seconds does introduce a significant
delay due to scheduling, so I recommend increasing the number of times the sleep function
is called instead of increasing the clock interval.
### Interrupt and Keyboard Input
* During the time the program is running, if click on the console,
you can type characters on your keyboard, and they will be immediately printed back
to the console. They will have virtually no effect on the program.
It's just so you can verify the interrupts are working correctly.
### Additional Features

####  Pipelining
* Fetch/Decode/Execute parts of the CPU can run at the same time, however only one
part may access memory at the same time. Since most clock cycles require memory access,
this only slightly increases efficiency and speed.
#### Multiple Execution Units
* There are 2 execution units, but the second one basically
never gets used because only one part of the CPU can access memory at a time.
It would work better if I had an operating system that used virtual addresses which mapped
to addresses that were split up between memory modules. That way different memory modules
can be accessed at a time by the CPU.
#### Memory Interleaving
* Memory is broken up 8-ways to work well with a wide path memory access
and cache.
#### Cache
* One cache module with 16 lines and 8-byte lines, for a total of 128 bytes.
This is the biggest reduction in clock cycles.
#### Performance
* Before implementing the additional features, it took 21,302 cycles to run the program.
Run the program and the new performance is outputted to the console.
### Known Issues
* The program should run and work as intended, but you may notice that the performance specs
are different every time you run it. There are no random numbers or additional input
(besides the interrupt devices which shouldn't have an effect on the performance), so we should
expect the same numbers every time, but that's not what we get. It works, so I'm not going to
try to figure out what's causing it.
# Download and run on your device
Your computer might tell you not to trust the executable if you download it.
If you don't want to take my word for it, then you just have to recompile the code yourself.
To compile on your device, follow these steps:
* Make sure you have the rust compiler installed on your device.
  Install it here: https://www.rust-lang.org/tools/install
* In the CLI, cd into the project folder so your working directory is Rust-422-tsiraM
* In the CLI, type "cargo run" and hope for no errors