// src/task/simple_executor.rs

use super::Task;
use alloc::collections::VecDeque;
use core::task::{Waker, RawWaker};
use core::task::RawWakerVTable;

pub struct SimpleExecutor {
    task_queue: VecDeque<Task>,
}

impl SimpleExecutor {
    /// Creates a new SimpleExecutor.
    /// 
    /// Returns the created executor.
    pub fn new() -> SimpleExecutor {
        SimpleExecutor {
            task_queue: VecDeque::new(),
        }
    }

    /// Spawns a new task by adding it to the executor's task queue.
    /// 
    /// Takes ownership of the task to be spawned.
    pub fn spawn(&mut self, task: Task) {
        self.task_queue.push_back(task)
    }
}

fn dummy_raw_waker() -> RawWaker {
    fn no_op(_: *const ()) {}
    fn clone(_: *const ()) -> RawWaker {
        dummy_raw_waker()
    }

    let vtable = &RawWakerVTable::new(clone, no_op, no_op, no_op);
    RawWaker::new(0 as *const (), vtable)
}

fn dummy_waker() -> Waker {
    unsafe { Waker::from_raw(dummy_raw_waker()) }
}