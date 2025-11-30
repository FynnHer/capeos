// src/task/simple_executor.rs

use super::Task;
use alloc::collections::VecDeque;
use core::task::{Waker, RawWaker};
use core::task::RawWakerVTable;
use core::task::{Context, Poll};

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

    /// Runs the executor until all tasks are complete.
    /// 
    /// this method takes the first task from the queue, polls it, and if it's not complete,
    /// re-adds it to the end of the queue.
    pub fn run(&mut self) {
        while let Some(mut task) = self.task_queue.pop_front() {
            let waker = dummy_waker();
            let mut context = Context::from_waker(&waker);
            match task.poll(&mut context) {
                Poll::Ready(()) => {} // Task is complete, do nothing
                Poll::Pending => {
                    self.task_queue.push_back(task); // Re-add the task to the queue
                }
            }
        }
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