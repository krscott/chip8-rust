use minifb::{Key, Window, WindowOptions};
use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

#[derive(Debug)]
enum WindowCmd {
    Update,
    UpdateBuffer(Arc<Mutex<Vec<u32>>>),
    PollKeys,
    Close,
    SetTitle(String),
}

pub struct WindowHandle {
    cmd_tx: mpsc::Sender<WindowCmd>,
    msg_keys_rx: mpsc::Receiver<Vec<Key>>,
    join_handle: thread::JoinHandle<()>,
}

impl WindowHandle {
    pub fn set_title(&self, title: String) -> anyhow::Result<()> {
        self.cmd_tx.send(WindowCmd::SetTitle(title))?;
        Ok(())
    }

    pub fn update(&self) -> anyhow::Result<()> {
        self.cmd_tx.send(WindowCmd::Update)?;
        Ok(())
    }

    pub fn update_buffer(&self, buffer: Arc<Mutex<Vec<u32>>>) -> anyhow::Result<()> {
        self.cmd_tx.send(WindowCmd::UpdateBuffer(buffer))?;
        Ok(())
    }

    pub fn get_keys(&self) -> anyhow::Result<Vec<Key>> {
        self.cmd_tx.send(WindowCmd::PollKeys)?;
        let keys = self.msg_keys_rx.recv()?;
        Ok(keys)
    }

    pub fn close(self) -> anyhow::Result<()> {
        self.cmd_tx.send(WindowCmd::Close)?;
        self.join_handle.join().unwrap();
        Ok(())
    }
}

pub fn spawn<S: Into<String>>(
    name: S,
    width: usize,
    height: usize,
    options: WindowOptions,
) -> anyhow::Result<WindowHandle> {
    let name: String = name.into();

    let (cmd_tx, cmd_rx) = mpsc::channel();
    let (msg_keys_tx, msg_keys_rx) = mpsc::channel();

    let join_handle = thread::spawn(move || {
        let mut window = Window::new(&name, width, height, options).unwrap();

        // Limit to max ~60 fps update rate
        window.limit_update_rate(Some(std::time::Duration::from_micros(1_000_000 / 60)));

        while window.is_open() {
            match cmd_rx.recv() {
                Err(_) => break,
                Ok(cmd) => match cmd {
                    WindowCmd::Update => window.update(),
                    WindowCmd::UpdateBuffer(buffer) => window
                        .update_with_buffer(&buffer.lock().unwrap(), width, height)
                        .unwrap(),
                    WindowCmd::PollKeys => {
                        let keys = window.get_keys().unwrap_or_default();
                        msg_keys_tx.send(keys).unwrap();
                    }
                    WindowCmd::Close => {
                        break;
                    }
                    WindowCmd::SetTitle(title) => window.set_title(&title),
                },
            }
        }
    });

    let handle = WindowHandle {
        cmd_tx,
        msg_keys_rx,
        join_handle,
    };

    handle.update()?;

    Ok(handle)
}
