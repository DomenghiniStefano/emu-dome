# NDS test ROM

`emudome-test.nds` is a minimal Nintendo DS program written for EmuDome (see `test.s`). It tests
the core, both screens, a button and the touch screen without using commercial games:

- top screen: magenta, green while A is held;
- bottom screen: dark blue; while touched its colour encodes the point the game read:
  red = x / 8, green = y / 8, blue = 16 (x 0..255, y 0..191).

Each screen shows only its backdrop colour (no graphics data). The ARM7 reads the touch screen
over SPI and leaves the point in main RAM for the ARM9, as a real DS game does.

It is a homebrew game (ARM9 offset under 0x4000): melonDS boots it directly with its built-in
BIOS and firmware and checks nothing taken from a commercial card. No logo in the header.

Rebuilding needs only Python 3 (`build.py` holds a tiny assembler for the instructions `test.s`
uses): `python3 build.py`.

Windowless check (both screens as PNGs, touch through the shell's mapping):

    cargo run -p emudome-core --example headless -- cores/melondsds_libretro.dll \
        tools/test-rom-nds/emudome-test.nds --screens <prefix> --touch

Licence: GPL-3.0-or-later, like the rest of EmuDome (`LICENSE` in this folder).
