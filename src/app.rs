use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;
use std::{io, time::Duration};

use ratatui::{Frame, buffer::Buffer, layout::Rect, widgets::Widget};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

use crate::config::miliseconds_per_frame;
use crate::settings::Settings;
use crate::stores::Store;
use crate::{action::Action, ui::screens::ScreenRouterComponent};

enum CurrentScreen {
    Menu,
    BasicType,
    Settings,
}

#[derive(Debug)]
pub struct AppStore {
    action_rx: UnboundedReceiver<Action>,
    dispatcher_tx: UnboundedSender<Action>,
    exit: bool,
    screen_router: ScreenRouterComponent,
    time_last_frame: Instant,
}
impl AppStore {
    fn new(
        dispatcher_tx: UnboundedSender<Action>,
        settings: Rc<RefCell<Settings>>,
    ) -> (Self, UnboundedSender<Action>) {
        let (action_tx, action_rx) = unbounded_channel::<Action>();
        let screen_router = ScreenRouterComponent::new(dispatcher_tx.clone(), settings.clone());
        (
            AppStore {
                action_rx,
                dispatcher_tx,
                screen_router,
                exit: false,
                time_last_frame: Instant::now(),
            },
            action_tx,
        )
    }
    async fn update(&mut self) {
        // Set up a timeout of 10ms
        let duration = Instant::now() - self.time_last_frame;
        let timeout = tokio::time::sleep(
            Duration::from_millis(miliseconds_per_frame())
                .checked_sub(duration)
                .unwrap_or_default(),
        );
        tokio::pin!(timeout);

        // Keep processing actions until timeout
        loop {
            // println!("waiting for acrtion");
            // let action = self.action_rx.recv().await.unwrap();
            // println!("action recieved");
            // self.text_store.update(action);

            tokio::select! {
                // Try to receive more actions (will not block if channel is empty)
                biased;

                maybe_action = self.action_rx.recv() => {
                    match maybe_action {
                        Some(action) => {
                            if matches!(action, Action::Exit) {self.exit = true}
                            self.screen_router.update(action);
                        },
                        None => {
                            // Channel is closed, exit loop
                            break;
                        }
                    }
                }
                // If timeout completes, exit the loop
                _ = &mut timeout => {
                    break;
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct App {
    app_store: AppStore,
}

impl App {
    pub fn new(
        dispatcher_tx: UnboundedSender<Action>,
        settings: Rc<RefCell<Settings>>,
    ) -> (Self, UnboundedSender<Action>) {
        let (app_store, action_tx) = AppStore::new(dispatcher_tx, settings);
        (App { app_store }, action_tx)
    }
    /// runs the application's main loop until the user quits
    pub async fn run(&mut self) -> io::Result<()> {
        let mut terminal = ratatui::init();
        self.app_store
            .dispatcher_tx
            .send(Action::StartingApp)
            .unwrap();
        while !self.app_store.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.app_store.time_last_frame = Instant::now();
            self.app_store.update().await;
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.app_store.screen_router.render(area, buf);
    }
}
