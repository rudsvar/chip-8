/*!

A CHIP-8 emulator as specified at https://en.wikipedia.org/wiki/CHIP-8.

# Crossterm Frontend

If you want to try the emulator on some programs, there is a ready-to-use implementation
you can run by using `cargo run --release --bin crossterm_frontend -- <program>`.
You can then use the keys 0-9 and a-f to to give input, but which ones to use depend on the CHIP-8 program.

# Library

If you are not interested in handling input (key presses and such),
then you can use `Emulator::new()` to get an emulator to work with.

The main way of running a program is to load instructions as bytes.

```rust
use chip_8::emulator::{Emulator, input::DummyInput, output::DummyOutput};

let mut emulator = Emulator::<DummyInput, DummyOutput>::new();

// Load a program at address 0x200.
let clear_display = [0x00, 0xE0];
emulator.load(&clear_display);
emulator.step(); // Will now clear the display
```

Alternatively, you can experiment by executing instructions manually.

```rust
use chip_8::emulator::{Emulator, input::DummyInput, output::DummyOutput};
use chip_8::emulator::instruction::{Instruction, Reg, Const, Addr};

let mut emulator = Emulator::<DummyInput, DummyOutput>::new();

// Execute instructions manually
emulator.execute_single(Instruction::ClearScreen);

// Or many sequentially
emulator.execute_many(&[
    Instruction::Goto(Addr(0x250)),
    Instruction::SetRegToConst(Reg(0xA), Const(35)),
    Instruction::SetRegToReg(Reg(0xB), Reg(0xA))
]);
```

## Custom input and output

To get keypresses, you must implement `EmulatorInput` and `EmulatorOutput`,
which represent somewhere to get keyboard input from and a screen respectively.
These tell the emulator how to get the currently presses keys, and how to draw to the screen.
Take a look at `src/emulator/input.rs` and `src/emulator/output.rs` to see how to implement this, then do the following.

```ignore
use chip_8::emulator::Emulator;

let mut emulator = Emulator::with_io(MyInput::new(), MyOutput::new());
```

You can then implement the emulator using your own custom frontend, as done with crossterm in crossterm_frontend.
*/

pub mod emulator;
pub mod util;
