# MEIF!

**MEIF** stands for _Minimally Efficient Image Format/File_ — it's actually pretty small, though writing it is a pain. Of course, it comes with quirks.

## Features

- Funny
- Indexed colors (256 colors)
- Lossless compression (funny compression)
- 8192×8192 pixel limit

## What it doesn't have

- Transparency
- Animation
- Metadata
- Truecolor
- And more!

## Funny stuff I regret making decisions on

- This project itself. Haha I need help.
- Technically, with the 4 bytes after `DIMN`, you could have a 65535×65535 image... but I decided to limit it to 8192×8192 because of this formula:

  ```rust
  let w = 32 * b00 - b01;
  let h = 32 * b10 - b11;
  ```

# Getting started
## Building
To build the project, you need to have Rust and Cargo installed. You can install them from [rustup.rs](https://rustup.rs/).
```bash
$ cargo build --release
```

## How to use the CLI
Start by converting an image to MEIF format:
```bash
$ cargo run --release image.png
```
This will create a file called `image.png.meif`. Now you can run
```bash
$ cargo run --release image.png.meif
```
This command will open a macroquad window and display the image.

## Writing a MEIF File
1. Start with MEIF, followed by four 0x00 bytes for no reason.
2. Then add 0x69, 0x42, and two more 0x00 bytes.
3. Write the DIMN section (followed by the 4 bytes used in the formula above).
4. Write INDX, followed by a list of hex colors (3 bytes each).
5. Write the DATA section according to the funny compression algorithm in `src/utils.rs`
6. Finally, write `DONE!` to indicate the end of the file, or else you'll get an error.

Honestly, just look at the code. Good luck reading it. ;)

## How to read a MEIF file
Don't.
