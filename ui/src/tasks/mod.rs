//! The `tasks` module is responsible for managing and running various tasks in the system.
//! It provides the necessary structures and traits to define tasks, manage their execution,
//! and track their metrics.

mod no_task;

pub mod buttons_task;
pub mod chat_task;
pub mod crypto_task;
pub mod fib_task;
pub mod keyboard_task;
pub mod link_task;
pub mod metrics_task;
pub mod render_task;
pub mod text_edit_task;

use crate::metrics::Metrics;
use crate::msg::Msg;
use crate::stack;
use bsp::console::Print;

#[cfg(feature = "std")]
extern crate std;

extern crate bsp;

/// Structure containing information about a task.
pub struct TaskInfo {
    /// The name of the task.
    //pub name: &'static str,
    pub name: &'static [u8; 8],
    /// The interval at which the task should run, in microseconds.
    pub run_every_us: u32,
    /// The maximum allowed execution time for the task, in microseconds.
    pub time_budget_us: u64,
    /// The maximum allowed memory usage for the task, in bytes.
    pub mem_budget_bytes: u32,
}

const JUNK_DATA_SIZE: usize = 0x1; // must be at least 1 byte

pub struct TaskData {
    pub junk_data: [u8; JUNK_DATA_SIZE],
    pub text_edit: text_edit_task::Data,
    pub chat: chat_task::Data,
    pub render: render_task::Data,
    pub crypto: crypto_task::Data,
    pub net_link: link_task::Data,
}

impl TaskData {
    /// Creates a new `TaskData` instance with an empty buffer.
    pub const fn new() -> Self {
        TaskData {
            text_edit: text_edit_task::Data::new(),
            chat: chat_task::Data::new(),
            render: render_task::Data::new(),
            crypto: crypto_task::Data::new(),
            net_link: link_task::Data::new(),
            junk_data: [0; JUNK_DATA_SIZE],
        }
    }
}

/// Trait that defines the behavior of a task.
pub trait Task {
    /// Method to execute the task.
    fn run(
        &self,
        sender: &mut crate::mpsc::Sender<Msg>,
        bsp: &mut bsp::BSP,
        data: &mut TaskData,
        metrics: &mut Metrics,
    );

    /// Returns the task information.
    fn info(&self) -> &'static TaskInfo;
}

/// The maximum number of tasks that can be managed by the `TaskMgr`.
pub const MAX_TASKS: usize = 10;

/// Structure that manages the execution of tasks.
pub struct TaskMgr<'a> {
    /// An array of tasks to be managed.
    tasks: [&'a dyn Task; MAX_TASKS],
    /// An array of timestamps indicating the last run time of each task.
    last_run: [hal::timer::MicroSeconds; MAX_TASKS],
    /// The number of tasks currently managed.
    num_tasks: usize,
    /// A message sender for inter-task communication.
    pub sender: &'a mut crate::mpsc::Sender<Msg>,
    /// A reference to the Board Support Package (BSP).
    pub bsp: &'a mut bsp::BSP,
    // hold all the data for the task to use
    pub data: &'a mut TaskData,
    /// A reference to the metrics structure for tracking task performance.
    pub metrics: &'a mut Metrics,
}

/// A placeholder task used when no task is assigned.
const NO_TASK: no_task::NoTask = no_task::NoTask {};

impl<'a> TaskMgr<'a> {
    /// Creates a new `TaskMgr` instance.
    pub fn new(
        s: &'a mut crate::mpsc::Sender<Msg>,
        bsp: &'a mut bsp::BSP,
        data: &'a mut TaskData,
        metrics: &'a mut Metrics,
    ) -> TaskMgr<'a> {
        TaskMgr {
            tasks: [&NO_TASK; MAX_TASKS],
            last_run: [hal::timer::MicroSeconds(0); MAX_TASKS],
            num_tasks: 0,
            sender: s,
            bsp: bsp,
            data: data,
            metrics: metrics,
        }
    }

    /// Adds a task to the `TaskMgr`.
    pub fn add_task(&mut self, task: &'a dyn Task) {
        if self.num_tasks >= MAX_TASKS {
            panic!("Too many tasks");
        }
        self.tasks[self.num_tasks] = task;
        self.num_tasks += 1;
    }

    /// Runs all the tasks managed by the `TaskMgr`, ensuring
    /// they adhere to their time and memory budgets.
    /// The method also updates the task metrics.
    pub fn run(&mut self) {
        stack::usage(true); // reset stack usage
        let (base_stack_usage, ..) = stack::usage(false);

        for i in 0..self.num_tasks {
            let t = self.tasks[i];
            let info = t.info();

            let now = hal::timer::current_time();

            if now.sub(self.last_run[i]).as_u64() < info.run_every_us as u64 {
                continue;
            }

            let start_time = hal::timer::current_time();
            t.run(self.sender, self.bsp, self.data, self.metrics);
            let end_time = hal::timer::current_time();
            let (end_stack_usage, ..) = stack::usage(false);

            self.last_run[i] = start_time;

            let duration = end_time.sub(start_time).as_u64();
            if duration > info.time_budget_us {
                b"Exceeded time budget\r\n".print_console();

                b" start=".print_console();
                start_time.as_u64().print_console();
                b" us\r\n".print_console();

                b" end=".print_console();
                end_time.as_u64().print_console();
                b" us\r\n".print_console();

                b" duration=".print_console();
                duration.print_console();
                b" us\r\n".print_console();

                b"Task overran time budget".print_console();
                //TODO panic!("Task {} overran time budget", i);
            }

            let stack_usage = end_stack_usage - base_stack_usage;
            if stack_usage > info.mem_budget_bytes as usize {
                b"Exceeded memory budget\r\n  usage==".print_console();
                (stack_usage as u64).print_console();
                b"\r\n".print_console();
                //TODO panic!("Task {} overran memory budget", i);
            }

            // Update metrics
            if self.metrics.task_name[i][0] == 0 {
                for j in 0..info.name.len() {
                    self.metrics.task_name[i][j] = info.name[j];
                }
            }
            self.metrics.task_run_count[i] += 1;
            if stack_usage > self.metrics.task_max_stack[i] as usize {
                self.metrics.task_max_stack[i] = stack_usage as u32;
            }
            if duration as u32 > self.metrics.task_max_duration_us[i] {
                self.metrics.task_max_duration_us[i] = duration as u32;
            }
        }
    }
}
