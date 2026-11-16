"""Assembles the test ROM: needs binutils-arm-none-eabi."""
import pathlib, subprocess, sys
here = pathlib.Path(__file__).parent
out = here / "emudome-test.gba"
subprocess.run(["arm-none-eabi-as", "-o", here / "test.o", here / "test.s"], check=True)
subprocess.run(["arm-none-eabi-ld", "-Ttext=0x08000000", "-o", here / "test.elf", here / "test.o"], check=True)
subprocess.run(["arm-none-eabi-objcopy", "-O", "binary", here / "test.elf", out], check=True)
rom = bytearray(out.read_bytes())
rom[0xBD] = (-(sum(rom[0xA0:0xBD]) + 0x19)) & 0xFF
rom += b"\xff" * (-len(rom) % 1024)
out.write_bytes(rom)
for f in ("test.o", "test.elf"):
    (here / f).unlink()
print(out, len(rom), "bytes")
