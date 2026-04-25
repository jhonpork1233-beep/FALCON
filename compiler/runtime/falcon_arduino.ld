/* ATmega328P (Arduino Uno/Nano) Linker Script for Falcon Baremetal
 * Memory: 32KB Flash, 2KB SRAM, 1KB EEPROM
 * CPU: 8-bit AVR (16 MHz)
 */

OUTPUT_FORMAT(elf32-avr)
OUTPUT_ARCH(avr:5)   /* ATmega328P = avr5 arch */
ENTRY(__vectors)

MEMORY
{
    flash  (rx)  : ORIGIN = 0x0000, LENGTH = 32K
    sram   (rw)  : ORIGIN = 0x0100, LENGTH = 2K    /* 0x0100-0x08FF */
}

SECTIONS
{
    /* Interrupt Vector Table — must be at very start of flash */
    .vectors : {
        KEEP(*(.vectors))
        KEEP(*(.vectors.*))
    } > flash

    /* Program code */
    .text : ALIGN(2) {
        *(.text .text.*)
        *(.progmem.data)
    } > flash

    /* Read-only data (constants, strings) */
    .rodata : ALIGN(2) {
        *(.rodata .rodata.*)
    } > flash

    /* Initialized data — stored in flash, copied to SRAM at startup */
    .data : ALIGN(2) {
        __data_start = .;
        *(.data .data.*)
        __data_end = .;
    } > sram AT > flash

    __data_load_start = LOADADDR(.data);

    /* Uninitialized data — zeroed at startup */
    .bss : ALIGN(2) {
        __bss_start = .;
        *(.bss .bss.*)
        *(COMMON)
        __bss_end = .;
    } > sram

    /* Stack grows downward from top of SRAM */
    __stack_top = ORIGIN(sram) + LENGTH(sram);

    /DISCARD/ : {
        *(.comment)
        *(.note*)
        *(.eh_frame*)
    }
}
