use std::{
    mem,
    pin::Pin,
    sync::Arc,
    thread::{self, JoinHandle},
};

use futures_lite::{future, pin, Future};

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

impl Default for TaskPool {
    fn default() -> Self {
        Self::new()
    }
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

    pub fn scope<'scope, F, T>(&self, f: F) -> Vec<T>
    where
        F: FnOnce(&mut Scope<'scope, T>) + Send + 'scope,
        T: Send + 'static,
    {
        TaskPool::LOCAL_EXECUTOR.with(|local_executor| {
            // SAFETY:
            // this function block until all futures have returned,
            // meaning self lives for at least 'scope.
            // but rust doesn't know, so we need to transmute lifetimes
            let executor: &async_executor::Executor = &*self.executor;
            let executor: &'scope async_executor::Executor = unsafe { mem::transmute(executor) };
            let local_executor: &'scope async_executor::LocalExecutor =
                unsafe { mem::transmute(local_executor) };

            let mut scope = Scope {
                executor,
                local_executor,
                spawned: Vec::new(),
            };

            f(&mut scope);

            if scope.spawned.is_empty() {
                Vec::new()
            } else if scope.spawned.len() == 1 {
                vec![future::block_on(&mut scope.spawned[0])]
            } else {
                let fut = async move {
                    let mut results = Vec::with_capacity(scope.spawned.len());
                    for task in scope.spawned {
                        results.push(task.await);
                    }

                    results
                };

                // Pin the futures on the stack.
                pin!(fut);

                // SAFETY: This function blocks until all futures complete, so we do not read/write
                // the data from futures outside of the 'scope lifetime. However,
                // rust has no way of knowing this so we must convert to 'static
                // here to appease the compiler as it is unable to validate safety.
                let fut: Pin<&mut (dyn Future<Output = Vec<T>>)> = fut;
                let fut: Pin<&'static mut (dyn Future<Output = Vec<T>> + 'static)> =
                    unsafe { mem::transmute(fut) };

                // The thread that calls scope() will participate in driving tasks in the pool
                // forward until the tasks that are spawned by this scope() call
                // complete. (If the caller of scope() happens to be a thread in
                // this thread pool, and we only have one thread in the pool, then
                // simply calling future::block_on(spawned) would deadlock.)
                let mut spawned = local_executor.spawn(fut);
                loop {
                    if let Some(result) = future::block_on(future::poll_once(&mut spawned)) {
                        break result;
                    };

                    self.executor.try_tick();
                    local_executor.try_tick();
                }
            }
        })
    }

    pub fn spawn<T>(&self, future: impl Future<Output = T> + Send + 'static) -> Task<T>
    where
        T: Send + 'static,
    {
        Task::new(self.executor.spawn(future))
    }

    pub fn spawn_local<T>(&self, future: impl Future<Output = T> + Send + 'static) -> Task<T>
    where
        T: Send + 'static,
    {
        Task::new(TaskPool::LOCAL_EXECUTOR.with(|executor| executor.spawn(future)))
    }
}

#[derive(Debug)]
pub struct Scope<'scope, T> {
    executor: &'scope async_executor::Executor<'scope>,
    local_executor: &'scope async_executor::LocalExecutor<'scope>,
    spawned: Vec<async_executor::Task<T>>,
}

impl<'scope, T: Send + 'scope> Scope<'scope, T> {
    pub fn spawn(&mut self, future: impl Future<Output = T> + Send + 'scope)
    where
        T: Send + 'scope,
    {
        self.spawned.push(self.executor.spawn(future));
    }

    pub fn spawn_local(&mut self, future: impl Future<Output = T> + Send + 'scope)
    where
        T: Send + 'scope,
    {
        self.spawned.push(self.local_executor.spawn(future));
    }
}
