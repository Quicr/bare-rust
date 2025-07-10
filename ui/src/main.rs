//! Application main loop and entry point

#![no_std]
#![cfg_attr(not(feature = "std"), no_main)]

#[cfg(feature = "std")]
extern crate std;

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
//use crate::tasks::text_edit_task;

#[cfg(not(feature = "std"))]
#[no_mangle]
#[inline(never)]
/// Entry point for the application when the `std` feature is not enabled.
pub extern "C" fn main() -> ! {
    my_main();

    loop {}
}

#[cfg(feature = "std")]
/// Entry point for the application when the `std` feature is enabled.
fn main() {
    led::set(Color::Blue);
    my_main();
}

//#[link_section = ".data"]
static mut HEAP_TASK_DATA: tasks::TaskData = tasks::TaskData::new();

fn alloc_task_data() -> &'static mut tasks::TaskData {
    #[allow(static_mut_refs)]
    unsafe {
        &mut HEAP_TASK_DATA
    }
}

#[cfg(feature = "std")]
fn print_memory_sizes() {
    use crate::tasks::*;

    std::println!("Size of Msg enum: {}", std::mem::size_of::<Msg>());
    std::println!(
        "Size of tasks::TaskData: {}",
        std::mem::size_of::<tasks::TaskData>()
    );
    std::println!(
        "Size of text_edit_task::Data: {}",
        std::mem::size_of::<text_edit_task::Data>()
    );
}

const TEST_PRINT_DATA: &[u8; 4] = b"1234";

#[inline(never)]
/// Main function that initializes the system and runs the task manager.
fn my_main() {
    //msg::test_msg();

    let mut bsp = bsp::BSP::new();

    bsp.init();

    //#[cfg(debug_assertions)]
    #[cfg(not(feature = "std"))]
    bsp.validate();

    //let stuff : [u8;1] = [ 0b0101_0101 ];
    //stuff.print_console();
    //led::set(Color::Black);
    //loop {};

    led::set(Color::Blue);

    b"Starting\r\n".print_console();

    // TODO remove - just testing
    if false {
        if cfg!(not(feature = "std")) {
            b"  Pre  DMA\r\n".print_console();
            //let data = b"TEST DMA \r\n";
            //let static const test_print_data = b"1234";
            #[allow(unused_unsafe)]
            unsafe {
                // TODO remove unsafe
                hal::uart::write1_dma(TEST_PRINT_DATA);
            }

            fib::fib_test();
            b"  Post  DMA\r\n".print_console();
        }
    }

    #[cfg(feature = "std")]
    print_memory_sizes();

    let (mut sender, receiver): (mpsc::Sender<msg::Msg>, mpsc::Receiver<msg::Msg>) =
        mpsc::channel();

    let mut metrics = metrics::Metrics::new();

    let mut data: &mut tasks::TaskData = alloc_task_data();

    data.junk_data[0] = 1;

    //let mut data2 = tasks::TaskData {
    //    text_edit: tasks::text_edit_task::Data::new(),
    //};

    let mut task_mgr = tasks::TaskMgr::new(&mut sender, &mut bsp, &mut data, &mut metrics);

    // this is removed for now as using button for mock keyboard
    //let button_task = tasks::buttons_task::ButtonTask {};
    //task_mgr.add_task(&button_task);

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

    //let fib_task = tasks::fib_task::FibTask {};
    //task_mgr.add_task(&fib_task);

    led::set(Color::Green);

    let (stack_usage, stack_current, stack_reserved) = stack::usage(false);
    if cfg!(not(feature = "std")) {
        b"  Starting stack usage: ".print_console();
        (stack_usage as u32).print_console();
        b" bytes\r\n".print_console();

        b"  Starting stack current: ".print_console();
        (stack_current as u32).print_console();
        b" bytes\r\n".print_console();

        b"  Starting stack reserved: ".print_console();
        (stack_reserved as u32).print_console();
        b" bytes\r\n".print_console();
    }

    // fib::fib_test();
    #[cfg(feature = "exit")]
    task_mgr.sender.send(Msg::Keyboard { key: '\r' });

    loop {
        task_mgr.run();
        dispatch::process(receiver, &mut task_mgr);

        #[cfg(feature = "exit")]
        {
            b"Stopping\r\n".print_console();
            #[cfg(not(feature = "std"))]
            hal::semihost::exit(0);
            #[cfg(feature = "std")]
            break;
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
    //use super::*;

    //#[test]
    //fn test_main() {
    //    main();
    //}

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

        crate::fib::fib_test();

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
