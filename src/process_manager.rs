use std::path::Path;
use std::process::{Child, Command};
use std::sync::mpsc::Sender;

use crate::setting::Setting;

//manage c21 process
pub enum ProcessRequest {
    KillMain,
    LaunchMain,
    KillStageEditor,
    LaunchStageEditor,
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
        let mut child_main: Option<Child> = None;
        let mut child_stage_editor: Option<Child> = None;
        for request in rx.iter() {
            match request {
                ProcessRequest::KillMain => {
                    if let Some(ref mut child) = child_main {
                        child.kill().ok();
                    }
                }
                ProcessRequest::LaunchMain => {
                    child_main = if cfg!(windows) {
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
                            .spawn()
                            .ok()
                    };
                }
                ProcessRequest::KillStageEditor => {
                    if let Some(ref mut child) = child_stage_editor {
                        child.kill().ok();
                    }
                }
                ProcessRequest::LaunchStageEditor => {
                    child_stage_editor = if cfg!(windows) {
                        Command::new("programs/cosmic.exe")
                            .current_dir(path.as_ref())
                            .arg("-launch")
                            .arg("-stageedit")
                            .spawn()
                            .ok()
                    } else {
                        Command::new("wine")
                            .current_dir(path.as_ref())
                            .arg("programs/cosmic.exe")
                            .arg("-launch")
                            .arg("-stageedit")
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

#[cfg(test)]
mod process_manager_test {
    use std::time::Duration;

    use crate::process_manager::{construct_launcher, ProcessRequest};

    #[test]
    fn test_launch() {
        let base_path = "/home/rustacean/.wine/drive_c/CyberStep/C21/";
        let ch = construct_launcher(base_path);
        ch.send(ProcessRequest::LaunchMain);
        std::thread::sleep(Duration::from_secs(100));
    }
}
