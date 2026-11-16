@ EmuDome test ROM: magenta screen, green while A is held.
@ Only meant to check core, video and input without using commercial games.
.arm
.section .text
.global _start
_start:
    b start
    .space 156              @ logo (not checked by the replacement BIOS)
    .ascii "EMUDOMETEST "   @ title, 12 bytes
    .ascii "ETST"           @ game code
    .ascii "00"             @ maker code
    .byte 0x96, 0, 0        @ fixed value, unit code, device type
    .space 7
    .byte 0                 @ version
    .byte 0                 @ checksum, computed by build.py
    .space 2
start:
    mov r0, #0x04000000
    ldr r1, =0x0403         @ mode 3, BG2 on
    strh r1, [r0]
loop:
    ldr r2, =0x04000130     @ KEYINPUT, bit at 0 = pressed
    ldrh r3, [r2]
    tst r3, #1              @ A button
    ldreq r4, =0x03E0       @ green
    ldrne r4, =0x7C1F       @ magenta
    orr r4, r4, r4, lsl #16
    mov r5, #0x06000000
    ldr r6, =19200          @ 240*160 pixels / 2 per word
fill:
    str r4, [r5], #4
    subs r6, r6, #1
    bne fill
    b loop
.pool
