#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

#[cfg(feature = "ev12")]
mod ev12;

#[cfg(feature = "ev13")]
mod ev13;

mod task;
mod traits;

use defmt_rtt as _;
use panic_halt as _;

#[rtic::app(device = stm32f4xx_hal::pac, peripherals = true)]
mod app {
    use crate::task::*;
    use crate::traits::*;

    #[cfg(feature = "ev12")]
    use crate::ev12 as hactar;

    #[cfg(feature = "ev13")]
    use crate::ev13 as hactar;

    type Hactar = hactar::Board;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        button_a: ButtonA<Hactar>,
        button_b: ButtonB<Hactar>,
        green_led_signal: GreenLedSignal<Hactar>,
        scan_inputs_timer: ScanInputsTimer<Hactar>,
        green_led_timer: GreenLedTimer<Hactar>,
        events: EventReceiver<Hactar>,
        blink_task: BlinkTask<GreenLed<Hactar>>,
        button_a_task: ButtonTask<RedLed<Hactar>>,
        button_b_task: ButtonTask<BlueLed<Hactar>>,
    }

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local) {
        let mut board = Hactar::new(ctx.device);

        let blink_task = BlinkTask::new(board.green_led());
        let button_a_task = ButtonTask::new(ButtonId::A, board.red_led());
        let button_b_task = ButtonTask::new(ButtonId::B, board.blue_led());

        defmt::info!("setup complete");

        (
            Shared {},
            Local {
                button_a: board.button_a(),
                button_b: board.button_b(),
                green_led_signal: board.green_led_signal(),
                scan_inputs_timer: board.scan_inputs_timer(),
                green_led_timer: board.green_led_timer(),
                events: board.events(),
                blink_task,
                button_a_task,
                button_b_task,
            },
        )
    }

    #[idle(local=[events, blink_task, button_a_task, button_b_task])]
    fn idle(ctx: idle::Context) -> ! {
        let mut task_master = TaskMaster::new(ctx.local.events.clone());
        task_master.add_task(ctx.local.blink_task);
        task_master.add_task(ctx.local.button_a_task);
        task_master.add_task(ctx.local.button_b_task);

        loop {
            task_master.run();
        }
    }

    #[task(binds = TIM2, local=[scan_inputs_timer, button_a, button_b])]
    fn scan_inputs(ctx: scan_inputs::Context) {
        ctx.local.scan_inputs_timer.reset();
        ctx.local.button_a.scan();
        ctx.local.button_b.scan();
    }

    #[task(binds = TIM3, local=[green_led_timer, green_led_signal])]
    fn green_led_timer(ctx: green_led_timer::Context) {
        ctx.local.green_led_timer.reset();
        ctx.local.green_led_signal.fire();
    }
}
