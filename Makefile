
.PHONY: run-sim build flash build-mgmt run-mgmt test doc clean cov all

all: build build-mgmt test doc 

run-sim:
	cargo run --bin ui --no-default-features --features bsp/board-sim,hal/stm32f405,hal/std,bsp/std,ui/std,ui/exit


build:
	cd crates/ui ; cargo build --bin ui --no-default-features --features bsp/board-hactar12,hal/stm32f405 --target=thumbv7em-none-eabihf --verbose


flash:
	cd crates/ui ; cargo flash --chip STM32F405RG --releasecargo flash --chip STM32F405RG --bin ui --no-default-features --features bsp/board-hactar12,hal/stm32f405 --target=thumbv7em-none-eabihf


build-mgmt:
	cd crates/mgmt; cargo build --bin mgmt --no-default-features --features hal/stm32f072 --target=thumbv6m-none-eabi  --verbose


flash-mgmt:
	cd crates/mgmt; cargo flash --chip STM32F072CB --bin mgmt --no-default-features --features hal/stm32f072 --target=thumbv6m-none-eabi --release


run-mgmt:
	echo run "openocd -f crates/mgmt/openocd.cfg" in background
	echo run something like "screen /dev/tty.usbserial-120 115200" in another window
	echo do "cd crates/mgmt; cargo run --bin mgmt --features hal/stm32f072 --target=thumbv6m-none-eabi  --verbose"


test:
	cargo test --workspace --lib --tests --bin ui  --no-default-features --features bsp/board-sim,hal/stm32f405,hal/std,ui/std,ui/exit  -- --test-threads=1
	cargo test --workspace --doc  --no-default-features --features bsp/board-sim,hal/stm32f405,hal/std,ui/std,ui/exit  -- --test-threads=1


doc:
	cargo doc --workspace --no-default-features --features bsp/board-sim,hal/stm32f405,hal/std,ui/std,ui/exit


clean:
	cargo clean


cov:
	cargo llvm-cov --workspace --lib --tests --bin ui --no-default-features --features bsp/board-sim,hal/stm32f405,hal/std,ui/std,ui/exit  -- --test-threads=1
