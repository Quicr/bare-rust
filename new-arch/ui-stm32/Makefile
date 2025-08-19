FLASHER=../../../hactar/software/flasher/main.py
PORT=/dev/tty.usbserial-110

.PHONY: flash

flash:
	cargo objcopy --release --target=thumbv7em-none-eabihf -- -O binary target/ui.bin
	python3 ${FLASHER} --baud=115200 --chip=ui --port=${PORT} --bin=target/ui.bin
