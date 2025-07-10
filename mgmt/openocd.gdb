target extended-remote localhost:3333
set confirm off

load

set print asm-demangle on

break Default_Handler
break mgmt::startup::panic
#break mgmt::startup::Reset_Handler


# break at start of program
#stepi

#break mgmt::my_main
break mgmt::main

