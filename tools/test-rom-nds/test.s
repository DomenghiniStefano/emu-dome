@ EmuDome NDS test ROM: one program for each CPU, assembled by build.py (no toolchain needed).
@ Only meant to check core, both screens, buttons and touch without using commercial games.
@
@ Top screen:    magenta, green while A is held.
@ Bottom screen: dark blue; while touched its colour encodes the touch point:
@                red = x / 8, green = y / 8, blue = 16 (x 0..255, y 0..191 on the bottom screen).
@
@ Both screens show only their backdrop colour (palette entry 0, no background layer), so the
@ program needs no VRAM and no graphics data. The touch screen is read by the ARM7 (it owns the
@ SPI bus), which leaves the point in main RAM for the ARM9.
@
@ Syntax: the subset build.py understands (data processing, ldr/str with an immediate offset,
@ ldr =constant, b/bl, condition suffixes eq/ne). `.cpu` starts a program: name and load address.

.cpu arm9 0x02000000
    ldr r0, =0x04000304     @ POWCNT1
    ldr r1, =0x8203         @ LCDs on, 2D engines A and B on, engine A on the top screen
    str r1, [r0, #0]
    mov r0, #0x04000000     @ DISPCNT of engine A
    mov r1, #0x00010000     @ display mode 1 (graphics), BG mode 0, no layer: backdrop only
    str r1, [r0, #0]
    add r2, r0, #0x1000     @ DISPCNT of engine B
    str r1, [r2, #0]
    mov r3, #0x05000000     @ engine A palette, entry 0 = top screen backdrop
    add r4, r3, #0x400      @ engine B palette, entry 0 = bottom screen backdrop
    ldr r5, =0x02300000     @ touch word written by the ARM7
    ldr r6, =0x04000130     @ KEYINPUT, a bit at 0 = pressed
loop9:
    ldrh r7, [r6, #0]
    tst r7, #1              @ A
    ldreq r8, =0x03E0       @ green
    ldrne r8, =0x7C1F       @ magenta
    strh r8, [r3, #0]
    ldr r7, [r5, #0]        @ bit 31 pen down, bits 0-7 x, bits 8-15 y
    tst r7, #0x80000000
    ldreq r8, =0x7C00       @ not touched: blue 31
    beq bottom9
    and r9, r7, #0xF8
    mov r8, r9, lsr #3      @ red = x / 8
    and r9, r7, #0xF800
    orr r8, r8, r9, lsr #6  @ green = y / 8 (bits 5-9)
    orr r8, r8, #0x4000     @ blue = 16 (bits 10-14)
bottom9:
    strh r8, [r4, #0]
    b loop9

.cpu arm7 0x037F8000
    ldr r0, =0x040001C0     @ SPICNT (SPIDATA at +2)
    ldr r5, =0x02300000     @ touch word for the ARM9
    ldr r10, =0x04000136    @ EXTKEYIN, bit 6 at 0 = pen down
loop7:
    ldrh r2, [r10, #0]
    tst r2, #0x40
    movne r3, #0
    strne r3, [r5, #0]
    bne loop7
    mov r4, #0xD0           @ touch controller: measure X, 12 bits
    bl adc7
    mov r8, r6, lsr #4      @ the core gives x * 16
    mov r4, #0x90           @ measure Y
    bl adc7
    mov r9, r6, lsr #4
    orr r8, r8, r9, lsl #8
    orr r8, r8, #0x80000000
    str r8, [r5, #0]
    b loop7

@ One conversion of the touch controller: command in r4, the 12-bit result in r6.
adc7:
    ldr r1, =0x8A01         @ SPI on, touch controller, chip select held, 2 MHz
    strh r1, [r0, #0]
    strh r4, [r0, #2]       @ the command
wait1:
    ldrh r2, [r0, #0]
    tst r2, #0x80           @ busy
    bne wait1
    mov r2, #0
    strh r2, [r0, #2]
wait2:
    ldrh r2, [r0, #0]
    tst r2, #0x80
    bne wait2
    ldrh r6, [r0, #2]       @ high byte
    and r6, r6, #0xFF
    ldr r1, =0x8201         @ last byte: chip select released afterwards
    strh r1, [r0, #0]
    mov r2, #0
    strh r2, [r0, #2]
wait3:
    ldrh r2, [r0, #0]
    tst r2, #0x80
    bne wait3
    ldrh r7, [r0, #2]       @ low byte
    and r7, r7, #0xFF
    orr r6, r7, r6, lsl #8
    mov r6, r6, lsr #3
    mov pc, lr
