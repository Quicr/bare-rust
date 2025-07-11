//! Application main loop and entry point

#![no_std]
#![no_main]


extern crate bsp;
extern crate hal;

use crate::channel::mpsc;
use bsp::console::Print;

use bsp::led;
use bsp::led::Color;

mod channel;
mod dispatch;
mod fib;
mod font;
mod metrics;
mod msg;
mod semihost;
mod stack;
mod startup;
mod tasks;
mod vec;

pub use msg::Msg;

#[no_mangle]
#[inline(never)]
/// Entry point for the application.
pub extern "C" fn main() -> ! {
    my_main();

    loop {}
}

static mut HEAP_TASK_DATA: tasks::TaskData = tasks::TaskData::new();

fn alloc_task_data() -> &'static mut tasks::TaskData {
    #[allow(static_mut_refs)]
    unsafe {
        &mut HEAP_TASK_DATA
    }
}


#[inline(never)]
/// Main function that initializes the system and runs the task manager.
fn my_main() {

    let mut bsp = bsp::BSP::new();

    bsp.init();

    bsp.validate();


    led::set(Color::Blue);

    b"Starting\r\n".print_console();



    let (mut sender, receiver): (mpsc::Sender<msg::Msg>, mpsc::Receiver<msg::Msg>) =
        mpsc::channel();

    let mut metrics = metrics::Metrics::new();

    let mut data: &mut tasks::TaskData = alloc_task_data();

    data.junk_data[0] = 1;


    let mut task_mgr = tasks::TaskMgr::new(&mut sender, &mut bsp, &mut data, &mut metrics);


    let chat_task = tasks::chat_task::ChatTask {};
    task_mgr.add_task(&chat_task);

    let crypto_task = tasks::crypto_task::CryptoTask {};
    task_mgr.add_task(&crypto_task);

    let keyboard_task = tasks::keyboard_task::KeyboardTask {};
    task_mgr.add_task(&keyboard_task);

    let metrics_task = tasks::metrics_task::MetricsTask {};
    task_mgr.add_task(&metrics_task);

    let net_link_task = tasks::link_task::LinkTask {};
    task_mgr.add_task(&net_link_task);

    let render_task = tasks::render_task::RenderTask {};
    task_mgr.add_task(&render_task);

    let text_edit_task = tasks::text_edit_task::TextEditTask {};
    task_mgr.add_task(&text_edit_task);


    led::set(Color::Green);

    let (stack_usage, stack_current, stack_reserved) = stack::usage(false);
    b"  Starting stack usage: ".print_console();
    (stack_usage as u32).print_console();
    b" bytes\r\n".print_console();

    b"  Starting stack current: ".print_console();
    (stack_current as u32).print_console();
    b" bytes\r\n".print_console();

    b"  Starting stack reserved: ".print_console();
    (stack_reserved as u32).print_console();
    b" bytes\r\n".print_console();

    #[cfg(feature = "exit")]
    task_mgr.sender.send(Msg::Keyboard { key: '\r' });

    loop {
        task_mgr.run();
        dispatch::process(receiver, &mut task_mgr);

        #[cfg(feature = "exit")]
        {
            b"Stopping\r\n".print_console();
            hal::semihost::exit(0);
        }
        #[cfg(test)]
        #[allow(unreachable_code)]
        {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_tasks() {
        let mut bsp = bsp::BSP::new();
        bsp.init();

        led::set(Color::Blue);

        bsp.validate();

        let (mut sender, receiver): (mpsc::Sender<msg::Msg>, mpsc::Receiver<msg::Msg>) =
            mpsc::channel();

        let mut metrics = metrics::Metrics::new();

        let mut data: &mut tasks::TaskData = alloc_task_data();

        let mut task_mgr = tasks::TaskMgr::new(&mut sender, &mut bsp, &mut data, &mut metrics);

        let button_task = tasks::buttons_task::ButtonTask {};
        task_mgr.add_task(&button_task);

        let chat_task = tasks::chat_task::ChatTask {};
        task_mgr.add_task(&chat_task);

        let crypto_task = tasks::crypto_task::CryptoTask {};
        task_mgr.add_task(&crypto_task);

        let keyboard_task = tasks::keyboard_task::KeyboardTask {};
        task_mgr.add_task(&keyboard_task);

        let metrics_task = tasks::metrics_task::MetricsTask {};
        task_mgr.add_task(&metrics_task);

        let net_link_task = tasks::link_task::LinkTask {};
        task_mgr.add_task(&net_link_task);

        let render_task = tasks::render_task::RenderTask {};
        task_mgr.add_task(&render_task);

        let text_edit_task = tasks::text_edit_task::TextEditTask {};
        task_mgr.add_task(&text_edit_task);

        let fib_task = tasks::fib_task::FibTask {};
        task_mgr.add_task(&fib_task);

        for i in 0..100 {
            task_mgr.run();
            dispatch::process(receiver, &mut task_mgr);

            if i == 5 {
                task_mgr.sender.send(Msg::Keyboard { key: 'A' });
            }
            if i == 10 {
                task_mgr.sender.send(Msg::Keyboard { key: '\r' });
            }
        }

        let stack_usage = stack::usage(false).0 as u32;
        if true {
            b"  test stack usage: ".print_console();
            stack_usage.print_console();
            b" bytes\r\n".print_console();
        }

        led::set(Color::Green);
    }
}
