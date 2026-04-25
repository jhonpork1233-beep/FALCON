/* ATmega328P Linker Script for Falcon Baremetal */
/* Arduino Uno: 32KB Flash, 2KB SRAM */

MEMORY
{
    FLASH (rx)  : ORIGIN = 0x0000, LENGTH = 32K
    SRAM  (rwx) : ORIGIN = 0x0100, LENGTH = 2K
}

SECTIONS
{
    /* Reset vector — jumps to _start */
    .vectors :
    {
        KEEP(*(.vectors))
    } > FLASH

    /* Program code */
    .text :
    {
        *(.text)
        *(.text.*)
    } > FLASH

    /* Read-only data */
    .rodata :
    {
        *(.rodata)
        *(.rodata.*)
    } > FLASH

    /* Initialized data (copied from flash to SRAM at startup) */
    .data :
    {
        _sdata = .;
        *(.data)
        *(.data.*)
        _edata = .;
    } > SRAM AT > FLASH

    /* Uninitialized data (zeroed at startup) */
    .bss :
    {
        _sbss = .;
        *(.bss)
        *(.bss.*)
        *(COMMON)
        _ebss = .;
    } > SRAM

    /* Stack at top of SRAM */
    _stack_top = ORIGIN(SRAM) + LENGTH(SRAM);
}

ENTRY(_start)
