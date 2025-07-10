/* Info for STM32F072C8T6 */

_Heap_Size = 0x0;
_Stack_Size = 0x100;

MEMORY
{
  FLASH  (RX) : ORIGIN = 0x08000000, LENGTH = 128K
  RAM    (XRW): ORIGIN = 0x20000000, LENGTH = 16K
}

ENTRY(Reset_Handler);

/* setup stack */
_estack = ORIGIN(RAM) + LENGTH(RAM);

SECTIONS
{
 .vector_table ORIGIN(FLASH) :
  {
    LONG(_estack);
    KEEP(*(.vector_table.reset_vector));
    KEEP(*(.vector_table.exceptions));
  } > FLASH

  .text : ALIGN(4)
  {
    *(.text .text.*);
  } > FLASH

   .rodata : ALIGN(4)
   {
    *(.rodata .rodata.*);
   } > FLASH

   .data : ALIGN(4) /* AT(ADDR(.rodata) + SIZEOF(.rodata)) */
   {
     _sdata = .;
     *(.data .data.*);
     _edata = .;
   }  > RAM AT > FLASH

   _sidata = LOADADDR(.data);

   .bss : ALIGN(4)
   {
      _sbss = .;
      *(.bss .bss.*);
      _ebss = .;
   } > RAM

   .heap_start :
   {
    _heap_start = .;
    . = ALIGN(4);
    . = . + _Heap_Size;
    _stack_reserve_start = .;
    . = . + _Stack_Size;
    _stack_reserve_end = .;
    . = ALIGN(4);
   } > RAM

   .stack_sizes (INFO) :
   {
     KEEP(*(.stack_sizes));
   }
}