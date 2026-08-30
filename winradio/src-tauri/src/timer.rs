use std::sync::Arc;
use parking_lot::Mutex;
use tokio::time::{sleep, Duration};
use tokio::sync::oneshot;
use serde::Serialize;
use tauri::{AppHandle, Manager};
use crate::audio::RadioPlayer;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SleepTimerArmedPayload {
    minutes: u32,
}

/// Emits `event` (with `payload`) through `app_handle` if one has been set.
/// Free function (not a `&self` method) so it can be called both from
/// `SleepTimer` itself and from inside the detached `tokio::spawn`ed task
/// below, which only holds a cloned `Arc<Mutex<Option<AppHandle>>>`, not a
/// `&SleepTimer`.
fn emit<S: Serialize + Clone>(app_handle: &Arc<Mutex<Option<AppHandle>>>, event: &str, payload: S) {
    if let Some(handle) = app_handle.lock().as_ref() {
        let _ = handle.emit_all(event, payload);
    }
}

#[derive(Clone)]
pub struct SleepTimer {
    timer_handle: Arc<Mutex<Option<oneshot::Sender<()>>>>,
    player: Arc<RadioPlayer>,
    app_handle: Arc<Mutex<Option<AppHandle>>>,
}

impl SleepTimer {
    pub fn new(player: Arc<RadioPlayer>) -> Self {
        Self {
            timer_handle: Arc::new(Mutex::new(None)),
            player,
            app_handle: Arc::new(Mutex::new(None)),
        }
    }

    pub fn set_app_handle(&self, handle: AppHandle) {
        *self.app_handle.lock() = Some(handle);
    }

    /// Arms a new timer for `minutes` minutes, or cancels the current one
    /// when `minutes == 0` (the existing `set_sleep_timer` command's
    /// semantics). Re-arming while already armed supersedes the old timer —
    /// its `select!` just exits quietly, since this call already emits its
    /// own `sleep-timer-armed` event and the frontend only cares about the
    /// latest state.
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
            emit(&self.app_handle, "sleep-timer-cleared", ());
            return;
        }

        emit(&self.app_handle, "sleep-timer-armed", SleepTimerArmedPayload { minutes });

        let player = self.player.clone();
        let app_handle = self.app_handle.clone();
        let timer_handle = self.timer_handle.clone();
        tokio::spawn(async move {
            tokio::select! {
                _ = sleep(Duration::from_secs(minutes as u64 * 60)) => {
                    let _ = player.stop().await;
                    // Reaching this branch means this task's `select!` won
                    // the race against its own `rx`, i.e. it was never
                    // cancelled/superseded — so `timer_handle` still holds
                    // (at most) this task's own sender. Clear it so a future
                    // "is a timer currently armed?" check on `timer_handle`
                    // doesn't see a stale `Some` for a timer that already
                    // fired (harmless today only because nothing queries
                    // this field yet — it self-corrects on the next `set()`
                    // call regardless).
                    *timer_handle.lock() = None;
                    emit(&app_handle, "sleep-timer-cleared", ());
                }
                _ = rx => {
                    // Timer cancelled or superseded by a newer `set()` call —
                    // that call already emitted its own event, so nothing to
                    // do here.
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Pins the wire shape of `sleep-timer-armed`'s payload: the frontend
    // reads `event.payload.minutes` (playback.ts) against a hand-typed
    // literal in its own test, so nothing there would catch a Rust-side
    // field rename (e.g. `minutes` -> `duration_minutes`, which `rename_all
    // = "camelCase"` would silently turn into `durationMinutes`). This test
    // is the one place that would fail if that ever happened.
    #[test]
    fn sleep_timer_armed_payload_serializes_to_camel_case_minutes() {
        let payload = SleepTimerArmedPayload { minutes: 30 };

        let json = serde_json::to_value(&payload).unwrap();

        assert_eq!(json, serde_json::json!({ "minutes": 30 }));
    }
}
