# Spaceship!

*Spaceship* is my first attempt at making a 2d space shooter game in Rust with Bevy.

![Screenshot of the game, showing the player ship surrounded by enemy ships](/demo.png)

It's not sophisticated, but includes the following:
- Low-res sprite graphics
- Sound effects for shooting and explosions
- Keyboard (arrow keys + space) or Gamepad support
- Level system that increases enemy speed
- Player death resets the level

### Platform support

I've only tested on Linux and in a browser using wasm.

The linux build is set up for the `mold` linker; feel free to delete
`.cargo/config` to get the stock linker.

The webassembly build can be previewed using `wasm-server-runner`; there isn't a proper wrapper page yet.

### Notes

The font is "Mono MMM 5" by Marcelo Magalhães Macedo.
