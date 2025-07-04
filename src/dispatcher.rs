use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

use crate::action::Action;

pub struct Dispatcher {
    action_rx: UnboundedReceiver<Action>,
    stores_tx: Vec<UnboundedSender<Action>>,
}
impl Dispatcher {
    pub fn new() -> (Self, UnboundedSender<Action>) {
        let (action_tx, action_rx) = mpsc::unbounded_channel::<Action>();
        (
            Dispatcher {
                action_rx,
                stores_tx: Vec::new(),
            },
            action_tx,
        )
    }
    pub fn add_store(&mut self, store_tx: UnboundedSender<Action>) -> &Self {
        self.stores_tx.push(store_tx);
        self
    }

    pub async fn dispatch(mut self) {
        loop {
            //tokio::task::yield_now().await;
            let action = self
                .action_rx
                .recv()
                .await
                .expect("Channel has been closed");
            // println!("action founded: {:?}", action);
            for store_rx in &self.stores_tx {
                store_rx
                    .send(action)
                    .expect("Error while sending the action");
            }
        }
    }
}
