use std::{
    sync::Arc,
    thread::{self, JoinHandle},
};

use futures_lite::{future, Future};

use crate::Task;

#[derive(Default)]
pub struct TaskPoolBuilder {
    num_threads: Option<usize>,
    stack_size: Option<usize>,
    thread_name: Option<String>,
}

#[must_use]
impl TaskPoolBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn num_threads(mut self, num_threads: usize) -> Self {
        self.num_threads = Some(num_threads);
        self
    }

    pub fn build(self) -> TaskPool {
        TaskPool::new_internal(
            self.num_threads,
            self.stack_size,
            self.thread_name.as_deref(),
        )
    }
}

#[derive(Debug)]
struct TaskPoolInner {
    threads: Vec<JoinHandle<()>>,
    shutdown: async_channel::Sender<()>,
}

impl Drop for TaskPoolInner {
    fn drop(&mut self) {
        self.shutdown.close();

        let panicking = thread::panicking();
        for join_handle in self.threads.drain(..) {
            let res = join_handle.join();
            if !panicking {
                res.expect("Task thread panicked during execution.");
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct TaskPool {
    executor: Arc<async_executor::Executor<'static>>,
    inner: Arc<TaskPoolInner>,
}

impl TaskPool {
    thread_local! {
        static LOCAL_EXECUTOR: async_executor::LocalExecutor<'static> = async_executor::LocalExecutor::new();
    }

    pub fn new() -> Self {
        TaskPoolBuilder::default().build()
    }

    fn new_internal(
        num_threads: Option<usize>,
        stack_size: Option<usize>,
        thread_name: Option<&str>,
    ) -> Self {
        let (shutdown_tx, shutdown_rx) = async_channel::unbounded::<()>();

        let executor = Arc::new(async_executor::Executor::new());

        let num_threads = num_threads.unwrap_or_else(num_cpus::get);

        let threads = (0..num_threads)
            .map(|i| {
                let ex = Arc::clone(&executor);
                let shutdown = shutdown_rx.clone();

                let thread_name = if let Some(thread_name) = thread_name {
                    format!("{} ({})", thread_name, i)
                } else {
                    format!("TaskPool ({})", i)
                };

                let mut thread_builder = thread::Builder::new().name(thread_name);

                if let Some(stack_size) = stack_size {
                    thread_builder = thread_builder.stack_size(stack_size);
                }

                thread_builder
                    .spawn(move || {
                        let shutdown = ex.run(shutdown.recv());

                        future::block_on(shutdown).unwrap_err();
                    })
                    .expect("failed to spawn thread.")
            })
            .collect();

        Self {
            executor,
            inner: Arc::new(TaskPoolInner {
                threads,
                shutdown: shutdown_tx,
            }),
        }
    }

    pub fn thread_count(&self) -> usize {
        self.inner.threads.len()
    }

    pub fn spawn<T>(&self, future: impl Future<Output = T> + Send + 'static) -> Task<T>
    where
        T: Send + 'static,
    {
        Task::new(self.executor.spawn(future))
    }
}

#[derive(Debug)]
pub struct Scope<'scope, T> {
    executor: &'scope async_executor::Executor<'scope>,
    local_executor: &'scope async_executor::LocalExecutor<'scope>,
    spawned: Vec<async_executor::Task<T>>,
}

impl<'scope, T: Send + 'scope> Scope<'scope, T> {}
