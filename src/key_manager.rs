use std::thread::{self, JoinHandle};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use std::sync::mpsc::{channel, Sender, Receiver};
use termion::event::Key;
use termion::input::TermRead;

const TIMEOUT: Duration = Duration::from_millis(250);

struct SharedData {
    key_pressed: Option<(Key, SystemTime)>,
    stop: bool,
    waiting_for_key: bool,
}

impl SharedData {
    fn new() -> Self {
        SharedData {
            key_pressed: None,
            stop: false,
            waiting_for_key: false
        }
    }
}

pub struct KeyManager {
    shared_data: Arc<Mutex<SharedData>>,
    receiver: Receiver<(Key, SystemTime)>,
    event_listener: JoinHandle<()>,
}

impl KeyManager {
    
    // Start even listener thread
    pub fn new() -> KeyManager {
        let (sender, receiver) = channel();
        let shared_data = Arc::new(Mutex::new(SharedData::new()));
        let event_listener = event_listener(sender, shared_data.clone());
        KeyManager {
            shared_data,
            receiver,
            event_listener
        }
    }

    // Get the currently pressed key if one exists
    pub fn get_key(&self) -> Option<Key> {
        let key_pressed = self.shared_data.lock().unwrap().key_pressed;
        key_pressed
            .filter(|(_, timestamp)| timestamp.elapsed().unwrap() < TIMEOUT )
            .map(|(key, _)| key)
    }

    // Tell the event listener to send the next key here
    pub fn get_key_blocking(&self) -> Key {
        let mut guard = self.shared_data.lock().unwrap();
        // Tell the listener that we are waiting
        guard.waiting_for_key = true;
        let mut received_key = Key::Null;
        // Find a fresh keypress
        for (key, timestamp) in self.receiver.iter() {
            if timestamp.elapsed().unwrap() < TIMEOUT {
                received_key = key;
            }
        }
        // Tell the listener that we are done waiting
        guard.waiting_for_key = false;
        received_key
    }
}

impl Drop for KeyManager {
    fn drop(&mut self) {
        // Tell the event listener to stop
        self.shared_data.lock().unwrap().stop = true;
        // TODO: Wait for it?
    }
}

fn event_listener(sender: Sender<(Key, SystemTime)>, shared_data: Arc<Mutex<SharedData>>) -> JoinHandle<()> {
    thread::spawn(move || {
        let stdin = std::io::stdin();
        for event in stdin.keys() {

            // Check the shared data, and possibly stop
            let mut shared_data = shared_data.lock().unwrap();
            if shared_data.stop {
                break;
            }

            // Investiage the event
            match event {
                Ok(key) => {
                    // Either send it 
                    let key_with_timestamp = (key, SystemTime::now());
                    if shared_data.waiting_for_key {
                        sender.send(key_with_timestamp).expect("Could not send via channel");
                    } else {
                        shared_data.key_pressed = Some(key_with_timestamp);
                    }
                }
                _ => {}
            }
        }
    })
}