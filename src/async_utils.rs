use std::sync::Arc;

use tokio::{runtime::Handle, sync::{mpsc::UnboundedSender, OnceCell, RwLock}, task::JoinHandle};

use crate::{action::Action, dispatcher, flux::SendAction};


#[derive(Debug, Clone)]
pub struct AsyncCache<T> {
    value: Arc<OnceCell<T>>,
    dispatcher_tx: Option<UnboundedSender<Action>>,
    id: Option<u32>
}

impl<T> AsyncCache<T>
where
    T: Send + Sync + Clone + std::fmt::Debug + 'static,
{
    pub fn new(dispatcher_tx: Option<UnboundedSender<Action>>, id: Option<u32>) -> Self {
        Self {
            value: Arc::new(OnceCell::new()),
            dispatcher_tx,
            id
        }
    }

    pub fn new_and_init<F, Fut>(dispatcher_tx: Option<UnboundedSender<Action>>, id: Option<u32>, func: F) -> Self
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: std::future::Future<Output = T> + Send + 'static,
    {
        let cache = Self::new(dispatcher_tx, id);
        cache.init(func);
        cache
    }

    /// Tries to get the cached value if it is ready, otherwise returns None.
    pub fn try_get(&self) -> Option<&T> {
        self.value.get()
    }


    /// Spawns the background initialization task if not already started.
    pub fn init<F, Fut>(&self, func: F)
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: std::future::Future<Output = T> + Send + 'static,
    {
        if self.value.get().is_some() {
            return;
        }

        let cell = self.value.clone();
        let dispatcher_tx = self.dispatcher_tx.clone();
        let action_id = self.id; 

        tokio::spawn(async move {
            let result = func().await;
            let _ = cell.set(result);
            if let Some(dispatcher_tx) = dispatcher_tx.as_ref() {
                dispatcher_tx.send(Action::AsyncCachedRecievedData(action_id)).unwrap()
            };
        });
    }

  
    pub fn is_available(&self) -> bool {
        self.try_get().is_some()
    }

}

impl<T> SendAction for AsyncCache<T> {
    fn send(&self, action: Action) -> Result<(), tokio::sync::mpsc::error::SendError<Action>> {
        if let Some(dispatcher) = self.dispatcher_tx.as_ref() {
            dispatcher.send(action)
        } else {
            Ok(())
        }
        
    }
}