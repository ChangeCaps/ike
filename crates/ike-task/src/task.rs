use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures_lite::Future;

#[derive(Debug)]
pub struct Task<T>(async_executor::Task<T>);

impl<T> Task<T> {
    pub fn new(task: async_executor::Task<T>) -> Self {
        Self(task)
    }

    pub fn detach(self) {
        self.0.detach();
    }

    pub async fn cancel(self) -> Option<T> {
        self.0.cancel().await
    }
}

impl<T> Future for Task<T> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.0).poll(cx)
    }
}
