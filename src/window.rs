use std::{
    sync::{Arc, Mutex, MutexGuard},
    thread::{self, JoinHandle},
};

use minifb::{Key, Window, WindowOptions};

// const PERIOD_60HZ_US: u64 = 1_000_000 / 60;

pub struct WindowHandle {
    join_handle: JoinHandle<()>,
    display_buffer: Arc<Mutex<Vec<u32>>>,
    display_dirty: Arc<Mutex<bool>>,
    keys: Arc<Mutex<Option<Vec<Key>>>>,
    keys_pressed: Arc<Mutex<Option<Vec<Key>>>>,
    title_update: Arc<Mutex<Option<String>>>,
    updated: Arc<Mutex<bool>>,
    closing: Arc<Mutex<bool>>,
}

impl WindowHandle {
    pub fn is_closing(&self) -> bool {
        *self.closing.lock().unwrap()
    }

    pub fn take_updated(&self) -> bool {
        let mut updated = self.updated.lock().unwrap();
        if *updated {
            *updated = false;
            true
        } else {
            false
        }
    }

    pub fn get_keys(&self) -> Option<Vec<Key>> {
        self.keys.lock().unwrap().clone()
    }

    pub fn get_keys_pressed(&self) -> Option<Vec<Key>> {
        self.keys_pressed.lock().unwrap().take()
    }

    pub fn get_display_buffer_mut(&mut self) -> MutexGuard<Vec<u32>> {
        let display_buffer_guard = self.display_buffer.lock().unwrap();

        *self.display_dirty.lock().unwrap() = true;

        display_buffer_guard
    }

    pub fn set_title(&mut self, title: String) {
        self.title_update.lock().unwrap().replace(title);
    }

    pub fn close(self) {
        *self.closing.lock().unwrap() = true;
        self.join_handle.join().unwrap();
    }
}

struct WindowSharedData {
    display_buffer: Arc<Mutex<Vec<u32>>>,
    display_dirty: Arc<Mutex<bool>>,
    keys: Arc<Mutex<Option<Vec<Key>>>>,
    keys_pressed: Arc<Mutex<Option<Vec<Key>>>>,
    title_update: Arc<Mutex<Option<String>>>,
    updated: Arc<Mutex<bool>>,
    closing: Arc<Mutex<bool>>,
}

pub fn spawn(title: String, width: usize, height: usize) -> WindowHandle {
    let display_buffer = Arc::new(Mutex::new((0..width * height).map(|_| 0).collect()));
    let display_dirty = Arc::new(Mutex::new(true));
    let keys = Arc::new(Mutex::new(None));
    let keys_pressed = Arc::new(Mutex::new(None));
    let title_update = Arc::new(Mutex::new(None));
    let updated = Arc::new(Mutex::new(false));
    let closing = Arc::new(Mutex::new(false));

    let shared_data = WindowSharedData {
        display_buffer: display_buffer.clone(),
        display_dirty: display_dirty.clone(),
        keys: keys.clone(),
        keys_pressed: keys_pressed.clone(),
        title_update: title_update.clone(),
        updated: updated.clone(),
        closing: closing.clone(),
    };

    let join_handle = thread::spawn(move || {
        let opts = WindowOptions {
            resize: true,
            scale: minifb::Scale::X16,
            scale_mode: minifb::ScaleMode::AspectRatioStretch,
            ..Default::default()
        };

        let mut window = Window::new(&title, width, height, opts).unwrap();

        // window.limit_update_rate(Some(Duration::from_micros(PERIOD_60HZ_US)));

        while !*shared_data.closing.lock().unwrap() && window.is_open() {
            if *shared_data.display_dirty.lock().unwrap() {
                let buffer = shared_data.display_buffer.lock().unwrap();
                window.update_with_buffer(&buffer, width, height).unwrap();
            } else {
                window.update();
            }

            {
                let mut keys = shared_data.keys.lock().unwrap();

                if let Some(new_keys) = window.get_keys() {
                    (*keys).replace(new_keys);
                } else {
                    (*keys).take();
                }
            }

            {
                let mut keys_pressed = shared_data.keys_pressed.lock().unwrap();

                if let Some(new_keys) = window.get_keys_pressed(minifb::KeyRepeat::Yes) {
                    (*keys_pressed).replace(new_keys);
                } else {
                    (*keys_pressed).take();
                }
            }

            if let Some(new_title) = shared_data.title_update.lock().unwrap().take() {
                window.set_title(&new_title);
            }

            *shared_data.updated.lock().unwrap() = true;
        }

        *shared_data.closing.lock().unwrap() = true;
    });

    WindowHandle {
        join_handle,
        display_buffer,
        display_dirty,
        keys,
        keys_pressed,
        title_update,
        updated,
        closing,
    }
}
