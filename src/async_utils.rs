use std::sync::Arc;

use tokio::sync::{OnceCell};
use tokio::task::JoinHandle;
use std::future::Future;

#[derive(Debug)]
pub enum TaskState<T> {
    Pending,
    Completed(T),
    Panicked,
}

#[derive(Debug, Clone)]
pub struct Deferred<T> {
    value: Arc<OnceCell<T>>,
    task_handle: Arc<OnceCell<JoinHandle<()>>>,
}

impl<T> Deferred<T>
where
    T: Send + Sync + 'static,
{
    pub fn new() -> Self {
        Self {
            value: Arc::new(OnceCell::new()),
            task_handle: Arc::new(OnceCell::new()),
        }
    }

    pub fn start<F, Fut>(computation: F) -> Self
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = T> + Send + 'static,
    {
        let deferred = Self::new();
        deferred.begin(computation);
        deferred
    }
    pub fn start_with_callback<F, C, Fut>(computation: F, callback: C) -> Self
    where
        F: FnOnce() -> Fut + Send + 'static,
        C: FnOnce() -> () + Send + 'static,
        Fut: Future<Output = T> + Send + 'static,
    {
        let deferred = Self::new();
        deferred.begin_with_callback(computation, callback);
        deferred
    }

    
    pub fn begin<F, Fut>(&self, computation: F) -> bool
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = T> + Send + 'static,
    {
        self._begin(computation, None::<fn()>)
    }

    fn begin_with_callback<F, C, Fut>(&self, computation: F, callback: C) -> bool
    where
        F: FnOnce() -> Fut + Send + 'static,
        C: FnOnce() -> () + Send + 'static,
        Fut: Future<Output = T> + Send + 'static,
    {
        self._begin(computation, Some(callback))
    }

    fn _begin<F, C, Fut>(&self, computation: F, callback: Option<C>) -> bool
    where
        F: FnOnce() -> Fut + Send + 'static,
        C: FnOnce() -> () + Send + 'static,
        Fut: Future<Output = T> + Send + 'static,
    {
        if self.value.get().is_some() || self.task_handle.get().is_some() {
            return false;
        }

        let cell = self.value.clone();
        
        let handle = tokio::spawn(async move {
            let result = computation().await;
            let _ = cell.set(result);
            if let Some(callback) = callback {
                callback()
            }
        });

        let _ = self.task_handle.set(handle);
        true
    }


    /// Returns the current state of the computation
    pub fn get_state(&self) -> TaskState<&T> {
        // Check if value is ready first
        if let Some(value) = self.value.get() {
            return TaskState::Completed(value);
        }

        // Check if task panicked
        if let Some(handle) = self.task_handle.get() {
            if handle.is_finished() {
                // Task finished but no value was set - must have panicked
                return TaskState::Panicked;
            }
        }

        TaskState::Pending
    }

    /// Non-blocking get that can detect panics
    pub fn try_get(&self) -> Option<&T> {
        match self.get_state() {
            TaskState::Completed(value) => Some(value),
            _ => None,
        }
    }

    /// Check if task panicked
    pub fn has_panicked(&self) -> bool {
        matches!(self.get_state(), TaskState::Panicked)
    }

    pub fn is_ready(&self) -> bool {
        matches!(self.get_state(), TaskState::Completed(_))
    }

    pub fn is_pending(&self) -> bool {
        matches!(self.get_state(), TaskState::Pending)
    }
    
}