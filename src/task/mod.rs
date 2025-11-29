// src/task/mod.rs

use core::{future::Future, pin::Pin};
use alloc::boxed::Box;
use core::task::{Context, Poll};

pub mod simple_executor;

pub struct Task {
    future: Pin<Box<dyn Future<Output = ()>>>,
}

impl Task {
    /// Creates a new Task wrapping the given future.
    /// 
    /// Returns the created Task.
    /// Static lifetime is required for the future to ensure it lives long enough.
    pub fn new(future: impl Future<Output = ()> + 'static) -> Task {
        Task {
            future: Box::pin(future),
        }
    }

    /// Polls the task's future to make progress.
    /// 
    /// returns Poll<()>, indicating whether the future is ready or pending.
    fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}