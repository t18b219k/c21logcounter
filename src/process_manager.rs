use std::process::{Child, Command};
use std::sync::mpsc::Sender;

//manage c21 process
enum ProcessRequest {
    Kill,
    Launch,
}
//start launcher
fn update() -> Option<Child> {
    Command::new("wine").arg("c21.exe").spawn().ok()
}
//create launcher
fn construct_launcher() -> Sender<ProcessRequest> {
    let (tx, rx) = std::sync::mpsc::channel();
    let thread_id = std::thread::spawn(move || {
        let mut child: Option<Child> = None;
        for request in rx.iter() {
            match request {
                ProcessRequest::Kill => {
                    if let Some(ref mut child) = child {
                        child.kill().ok();
                    }
                }
                ProcessRequest::Launch => {
                    child = Command::new("wine")
                        .arg("programs/cosmic.exe")
                        .arg("-launch")
                        .spawn()
                        .ok();
                }
            }
        }
    });

    tx
}
