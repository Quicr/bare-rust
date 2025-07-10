
.PHONY: run-sim build flash build-mgmt run-mgmt test doc clean cov all

all: build build-mgmt test doc 

run-sim:
	cargo run --bin ui --no-default-features --features bsp/board-sim,hal/stm32f405,hal/std,bsp/std,ui/std,ui/exit


build:
	cd ui ; cargo build --bin ui --no-default-features --features bsp/board-hactar12,hal/stm32f405 --target=thumbv7em-none-eabihf --verbose

build-A:
	cd ui ; cargo build --bin ui --no-default-features --features bsp/board-blinkA,hal/stm32f405 --target=thumbv7em-none-eabihf --verbose


flash:
	cd ui ; cargo flash --chip STM32F405RGTx --bin ui --no-default-features --connect-under-reset --features bsp/board-hactar12,hal/stm32f405 --target=thumbv7em-none-eabihf


flash-A:
	cd ui ; cargo flash --chip STM32F405RGTx --bin ui --no-default-features --features bsp/board-blinkA,hal/stm32f405 --target=thumbv7em-none-eabihf


build-mgmt:
	cd mgmt; cargo build --bin mgmt --no-default-features --features hal/stm32f072 --target=thumbv6m-none-eabi  --verbose


flash-mgmt:
	cd mgmt; cargo flash --chip STM32F072CBTx --bin mgmt --no-default-features --features hal/stm32f072 --target=thumbv6m-none-eabi --release


run-mgmt:
	echo run "openocd -f mgmt/openocd.cfg" in background
	echo run something like "screen /dev/tty.usbserial-120 115200" in another window
	echo do "cd mgmt; cargo run --bin mgmt --features hal/stm32f072 --target=thumbv6m-none-eabi  --verbose"


test:
	cargo test --workspace --lib --tests --bin ui  --no-default-features --features bsp/board-sim,hal/stm32f405,hal/std,ui/std,ui/exit  -- --test-threads=1
	cargo test --workspace --doc  --no-default-features --features bsp/board-sim,hal/stm32f405,hal/std,ui/std,ui/exit  -- --test-threads=1


doc:
	cargo doc --workspace --no-default-features --features bsp/board-sim,hal/stm32f405,hal/std,ui/std,ui/exit


clean:
	cd mgmt && cargo clean
	cd ui && cargo clean

cov:
	cargo llvm-cov --workspace --lib --tests --bin ui --no-default-features --features bsp/board-sim,hal/stm32f405,hal/std,ui/std,ui/exit  -- --test-threads=1
