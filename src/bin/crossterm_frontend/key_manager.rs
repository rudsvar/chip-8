use super::key_buffer::KeyBuffer;
use crossterm::event::{read, Event, KeyCode};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

pub struct KeyManager {
    stop: Arc<AtomicBool>,
    key_buffer: Arc<KeyBuffer>,
    event_listener: Option<JoinHandle<()>>,
}

/// A struct for managing keypresses that will automatically
/// start a thread that grabs keypresses.
impl KeyManager {
    // Start even listener thread
    pub fn new() -> KeyManager {
        let stop = Arc::new(AtomicBool::new(false));
        let key_buffer = Arc::new(KeyBuffer::new(Duration::from_millis(250)));
        let event_listener = event_listener(stop.clone(), key_buffer.clone());
        KeyManager {
            stop,
            key_buffer,
            event_listener: Some(event_listener),
        }
    }

    /// Get the currently pressed key if one exists
    pub fn get_key(&self) -> Option<KeyCode> {
        self.key_buffer.peek()
    }

    /// Get a key by blocking
    pub fn get_key_blocking(&self) -> KeyCode {
        self.key_buffer.pop_blocking()
    }
}

impl Drop for KeyManager {
    fn drop(&mut self) {
        // Tell the event listener to stop
        self.stop.store(true, Ordering::Relaxed);
        // Wait for it to stop
        if let Some(thread) = self.event_listener.take() {
            thread.join().expect("Could not join event listener thread");
        }
    }
}

/// Starts a thread that listens for key events and pushes them to the key buffer.
fn event_listener(stop: Arc<AtomicBool>, key_buffer: Arc<KeyBuffer>) -> JoinHandle<()> {
    thread::spawn(move || {
        loop {
            let event = read().unwrap();
            log::info!("Got event {:?}", event);

            // Check the shared data, and possibly stop
            if stop.load(Ordering::Relaxed) {
                break;
            }

            // Push keypresses into the key buffer
            if let Event::Key(key_event) = event {
                key_buffer.push(key_event.code);
            }
        }
    })
}
