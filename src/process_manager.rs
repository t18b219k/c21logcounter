use crate::setting::Setting;
use std::path::Path;
use std::process::{Child, Command};
use std::sync::mpsc::Sender;

//manage c21 process
pub enum ProcessRequest {
    Kill,
    Launch,
}
//start launcher
pub fn update(setting: &Setting) -> Option<Child> {
    #[cfg(not(windows))]
    let process = Command::new("wine")
        .current_dir(&setting.base_path)
        .arg(&setting.launcher_name)
        .spawn()
        .ok();

    #[cfg(windows)]
    let process = Command::new(&setting.launcher_name)
        .current_dir(&setting.base_path)
        .spawn()
        .ok();
    process
}
//create launcher
pub fn construct_launcher<P: AsRef<Path> + Send + 'static>(path: P) -> Sender<ProcessRequest> {
    let (tx, rx) = std::sync::mpsc::channel();

    let _thread_id = std::thread::spawn(move || {
        let mut child: Option<Child> = None;
        for request in rx.iter() {
            match request {
                ProcessRequest::Kill => {
                    if let Some(ref mut child) = child {
                        child.kill().ok();
                    }
                }
                ProcessRequest::Launch => {
                    let log_output = std::fs::File::create("spawned_process_output.txt").unwrap();
                    child = if cfg!(windows) {
                        Command::new("programs/cosmic.exe")
                            .current_dir(path.as_ref())
                            .arg("-launch")
                            .spawn()
                            .ok()
                    } else {
                        Command::new("wine")
                            .current_dir(path.as_ref())
                            .arg("programs/cosmic.exe")
                            .arg("-launch")
                            .stdout(log_output)
                            .spawn()
                            .ok()
                    };
                }
            }
            println!("Request Received");
        }
    });

    tx
}
#[test]
fn test_launch() {
    let base_path = "/home/rustacean/.wine/drive_c/CyberStep/C21/";
    let ch = construct_launcher(base_path);
    ch.send(ProcessRequest::Launch);
    std::thread::sleep(Duration::from_secs(100));
}
#[test]
fn test_kill() {}
