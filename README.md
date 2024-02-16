See README.md in master branch for details about the project.
# Download and run on your device
Your computer might tell you not to trust the executable if you download it.
If you don't want to take my word for it, then you just have to recompile the code yourself.  

To compile on your device, follow these steps:
* Make sure you have the rust compiler installed on your device.
Install it here: https://www.rust-lang.org/tools/install
* In the CLI, cd into the project folder so your working directory is Rust-422-tsiraM
* In the CLI, type "cargo run" and hope for no errors
# Comments for Lab 1
<ul>
    <li>
        I had to restructure the object hierarchy so that the different components can
        interact with each other more easily. I will likely try to refactor it after
        the instructions for lab 2 are released because encapsulation doesn't seem to
        be a good way of structuring this project.
    </li>
    <li>
        I also got bored and made a "Hello World!" program in assembly. If the instructions
        for future labs ask to do it a different way, I'll refactor it (the SYS instruction
        literally just calls the print!() macro which heavily abstracts the actual
        inner-workings of the processor's I/O).
    </li>
</ul>