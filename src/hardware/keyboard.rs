use std::error::Error;
use std::io::{Read, stdin};
use std::sync::mpsc::Sender;
use std::thread;
use std::thread::JoinHandle;

fn exit_on_error(e: impl Error) -> ! {
    panic!("Error in keyboard poller: {e}");
}
#[must_use]
pub fn create_keyboard_poller(sender: Sender<u16>) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut buf = [0u8; 1];
        loop {
            match stdin().read(&mut buf) {
                Ok(0) => break,
                Ok(1) => {
                    let send_res = sender.send(u16::from(buf[0]));
                    if let Err(e) = send_res {
                        eprintln!("Error sending from keyboard poller {e}");
                        break;
                    }
                }
                Ok(_) => unreachable!(),
                Err(e) => exit_on_error(e),
            }
        }
    })
}
