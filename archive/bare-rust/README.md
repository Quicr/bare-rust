# bare-rust

Bare rust project for STM32

Read the [Architecture.md](Architecture.md) document for more
information.

# Instalations

```sh
rustup target add thumbv6m-none-eab
rustup target add thumbv7em-none-eabihf
```

```sh
brew install probe-rs-tools
or
cargo install probe-rs-tools 
```


# Running on Board

Connect up the stlink to board.

```sh
make flash
```

or 

```sh
cd crates/ui; cargo flash --chip STM32F405RG --bin ui --no-default-features --features bsp/board-hactar12,hal/stm32f405 --target=thumbv7em-none-eabihf
````


In a separate window, on a mac, can monitor USB console with

```sh
screen /dev/tty.usbserial<SOOMETHING> 115200
```

To exit screen, type ^A^\


# Running with GDB

From in the main directory, start

```sh
openocd -f crates/ui/openocd.cfg
```

and leave running. Then in another window do

```sh
cd crates/ui; cargo run --bin ui --no-default-features --features bsp/board-hactar12,hal/stm32f405 --target=thumbv7em-none-eabihf
```

or for the MGMT chip

from in the main directory, start

```sh
openocd -f crates/mgmt/openocd.cfg
```

```shell
cd crates/mgmt; cargo run --bin mgmt  --no-default-features --features hal/stm32f072 --target=thumbv6m-none-eabi
```
this will bring you to gdb prompt where you can type "c" to continue.

Can flash the mgmt chip with (make sure openocd is not running)

```shell
cd crates/mgmt; cargo flash --chip STM32F072CB --bin mgmt --no-default-features --features hal/stm32f072 --target=thumbv6m-none-eabi --release
```


# Running on Emulator

Does not work yet.

```sh
TBD broken
cargo build --features bsp/board-qemu,exit --target=thumbv7em-none-eabihf 
qemu-system-arm  -S -no-reboot -cpu cortex-m4  -machine netduinoplus2  -gdb tcp::3333  -nographic  -semihosting-config enable=on,target=native -kernel target/thumbv7em-none-eabihf/debug/ui  --trace "memory_region_ops_*" 
arm-none-eabi-gdb -q  target/thumbv7em-none-eabihf/debug/ui --init-eval-command="target extended-remote localhost:3333"
```

The -S cause the above to wait for the debugger to connect.

# Running on the Simulator

```aiignore
make run-sim
```

# Notes

Useful things to look at size of binary:

```aiignore
cd crates/ui
cargo build  --release
arm-none-eabi-size ../../target/thumbv7em-none-eabihf/release/ui 
cargo nm --release -- -a -n

cargo objdump --bin mgmt --no-default-features --features hal/stm32f072 --target=thumbv6m-none-eabi   --  --disassemble-symbols="mgmt::my_main::h198d2405b1d924dc"  --source   --demangle

cargo objdump --bin mgmt --no-default-features --features hal/stm32f072 --target=thumbv6m-none-eabi   --  --disassemble-all  --source   --demangle

 cargo objdump --release  --bin ui --no-default-features --features bsp/board-hactar12,hal/stm32f405 --target=thumbv7em-none-eabihf  --  --disassemble-symbols="ui::fib::fib::h66fa5b0d20d9db18"  --source   --demangle

cargo objdump  --bin ui --no-default-features --features bsp/board-hactar12,hal/stm32f405 --target=thumbv7em-none-eabihf  --  --disassemble-symbols="ui::fib::fib::hc749cb1bc8cd917c"  --source   --demangle

```
