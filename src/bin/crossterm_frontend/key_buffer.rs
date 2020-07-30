use crossterm::event::KeyCode;
use std::{
    time::{Duration, SystemTime},
    sync::{Mutex, Condvar},
    collections::VecDeque
};

/// A thread-safe buffer for storing keys and timestamps.
/// For use with a producers and consumers of keys.
/// Wrap it in an `std::sync::Arc` and you are good to go.
pub struct KeyBuffer {
    timeout: Duration,
    buffer: Mutex<VecDeque<(KeyCode, SystemTime)>>,
    condvar: Condvar
}

impl KeyBuffer {

    /// Create a new `KeyBuffer`, but don't return keypresses that are older than `timeout`.
    pub fn new(timeout: Duration) -> KeyBuffer {
        KeyBuffer {
            timeout,
            buffer: Mutex::new(VecDeque::new()),
            condvar: Condvar::new()
        }
    }

    // Filter out old keypresses
    fn clean(&self) {
        let mut buffer_guard = self.buffer.lock().unwrap();
        // Filter out old values
        *buffer_guard = buffer_guard.iter()
            .filter(|(_, ts)| ts.elapsed().unwrap() < self.timeout)
            .map(|(a, b)| (*a, *b))
            .collect();
    }

    /// Push a new keypress to the buffer.
    pub fn push(&self, key_code: KeyCode) {
        self.clean();
        self.buffer.lock().unwrap()
            .push_back((key_code, SystemTime::now()));
        self.condvar.notify_one();
    }

    /// Peek at the current keypress
    pub fn peek(&self) -> Option<KeyCode> {
        self.clean();
        // Select the keycode component
        self.buffer.lock().unwrap()
            .front()
            .map(|(kc, _)| *kc)
    }

    /// Pop a keypress from the buffer if a fresh enough one exists.
    pub fn pop(&self) -> Option<KeyCode> {
        let mut buffer_guard = self.buffer.lock().unwrap();
        buffer_guard.pop_front()
            .filter(|(_, ts)| ts.elapsed().unwrap() < self.timeout)
            .map(|(kc, _)| kc)
    }

    /// Pop a keypress from the buffer, even if it requires some waiting.
    pub fn pop_blocking(&self) -> KeyCode {
        let mut buffer_guard = self.buffer.lock().unwrap();
        loop {
            match buffer_guard.pop_front() {
                Some((key_code, timestamp)) => {
                    if timestamp.elapsed().unwrap() < self.timeout {
                        return key_code;
                    }
                }
                None => {}
            }
            buffer_guard = self.condvar.wait(buffer_guard).unwrap();
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::{sync::Arc, thread};

    #[test]
    fn push_and_pop_blocking() {
        let kb = Arc::new(KeyBuffer::new(Duration::from_millis(100)));
        
        let kb_c1 = kb.clone();
        let kb_c2 = kb.clone();
        let input = KeyCode::Null;

        let consumer = thread::spawn(move || {
            kb_c2.pop_blocking()
        });

        let producer = thread::spawn(move || {
            thread::sleep(Duration::from_millis(10));
            kb_c1.push(input)
        });

        // Allow the consumer to arrive first
        let output = consumer.join().unwrap();
        producer.join().unwrap();
        assert_eq!(output, input);
    }
}