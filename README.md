# chipokto

chipokto is a [Chip8](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#00Cn) and [SuperChip8](https://github.com/trapexit/chip-8_documentation/blob/master/Misc/SCHIP-8%20v1.1.txt) emulator written in Rust. It was designed as a vehicle to teach end-to-end software engineering concepts.

![Space Invaders on Chipokto](/docs/chipokto_INVADERS_screenshot.png)
![Pong on Chipokto](chipokto_PONG_screenshot.png)

## Development

The emulator is divided into the following subsystems:

* **okto/** - Library defining the Chip8 and SuperChip8 emulator.
* **oktodis/** - Disassembler using the _okto_ library.
* **chipokto/** - Graphical emulator application using the _okto_ library.

To build all of the packages simply run the following in the root directory:

```shell
cargo build
```
