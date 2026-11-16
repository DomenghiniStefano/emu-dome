# Test ROM

`emudome-test.gba` is a minimal GBA program written for EmuDome (see `test.s`):
it fills the screen with magenta and turns green while A is held.
It tests core, video and input without using commercial games.

Rebuilding it needs `binutils-arm-none-eabi`: `python3 build.py`.
