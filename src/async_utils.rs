use std::sync::Arc;

use tokio::sync::{OnceCell};



#[derive(Debug, Clone)]
pub struct AsyncCache<T> {
    value: Arc<OnceCell<T>>,
}

impl<T> AsyncCache<T>
where
    T: Send + Sync + 'static,
{
    pub fn new() -> Self {
        Self {
            value: Arc::new(OnceCell::new()),
        }
    }

    pub fn new_and_init<F, C, Fut>(func: F, callback: Option<C>) -> Self
    where
        F: FnOnce() -> Fut + Send + 'static,
        C: FnOnce() -> () + Send + 'static,
        Fut: std::future::Future<Output = T> + Send + 'static,
    {
        let cache = Self::new();
        cache.init(func, callback);
        cache
    }

    /// Tries to get the cached value if it is ready, otherwise returns None.
    pub fn try_get(&self) -> Option<&T> {
        self.value.get()
    }


    /// Spawns the background initialization task if not already started.
    pub fn init<F, C, Fut>(&self, func: F, callback: Option<C>)
    where
        F: FnOnce() -> Fut + Send + 'static,
        C: FnOnce() -> () + Send + 'static,
        Fut: std::future::Future<Output = T> + Send + 'static,
    {
        if self.value.get().is_some() {
            return;
        }

        let cell = self.value.clone();

        tokio::spawn(async move {
            let result = func().await;
            let _ = cell.set(result);
            if let Some(callback) = callback {
                callback();
            };
        });
    }

  
    pub fn is_available(&self) -> bool {
        self.try_get().is_some()
    }
}