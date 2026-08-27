use std::sync::Arc;
use parking_lot::Mutex;
use tokio::time::{sleep, Duration};
use tokio::sync::oneshot;
use crate::audio::RadioPlayer;

pub struct SleepTimer {
    timer_handle: Arc<Mutex<Option<oneshot::Sender<()>>>>,
    player: Arc<RadioPlayer>,
}

impl SleepTimer {
    pub fn new(player: Arc<RadioPlayer>) -> Self {
        Self {
            timer_handle: Arc::new(Mutex::new(None)),
            player,
        }
    }

    pub async fn set(&self, minutes: u32) {
        let (tx, rx) = oneshot::channel();

        {
            let mut handle = self.timer_handle.lock();
            if let Some(old_tx) = handle.take() {
                let _ = old_tx.send(());
            }
            *handle = Some(tx);
        }

        if minutes == 0 {
            return;
        }

        let player = self.player.clone();
        tokio::spawn(async move {
            tokio::select! {
                _ = sleep(Duration::from_secs(minutes as u64 * 60)) => {
                    let _ = player.stop().await;
                }
                _ = rx => {
                    // Timer cancelled
                }
            }
        });
    }
}