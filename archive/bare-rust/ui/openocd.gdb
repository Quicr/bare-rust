target extended-remote localhost:3333
set confirm off

load

set print asm-demangle on

#break Default_Handler

break ui::startup::panic
break ui::startup::Reset_Handler
#break ui::my_main
break ui::main

# break at start of program
stepi


