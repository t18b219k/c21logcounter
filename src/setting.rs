use std::path::MAIN_SEPARATOR;

use serde::{Deserialize, Serialize};
use sysinfo::{ProcessExt, System, SystemExt};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Setting {
    pub launcher_name: String,
    pub base_path: String,
    pub port: u16,
}

pub enum GetPathError {
    ProcessNotFound,
}
//プロセステーブルを検索してパスを取ってくる.
fn search_process() -> Option<String> {
    let s = System::new_all();
    for process in s.get_processes().values() {
        for arg in process.cmd() {
            if arg.contains("c21.exe") | arg.contains("c21_steam.exe") {
                return Some(arg.clone());
            }
        }
    }
    None
}
pub fn get_path_from_launcher() -> Result<Setting, GetPathError> {
    let executable = search_process();
    let executable = if let Some(exec) = executable {
        exec
    } else {
        return Err(GetPathError::ProcessNotFound);
    };
    //generate wine prefix
    #[cfg(not(target_os = "windows"))]
    let drive_c = {
        let wine_prefix = std::env::var("WINEPREFIX").ok();
        let wine_prefix = wine_prefix.unwrap_or_else(||"~/.wine/".to_string());
        let wine_prefix = wine_prefix.replace("~", &std::env::var("HOME").unwrap());
        let mut drive_c = wine_prefix;
        drive_c.push_str("drive_c/");
        drive_c
    };
    let executable = {
        #[cfg(not(target_os = "windows"))]
        {
            executable.replace("C:\\", &drive_c)
        }
        #[cfg(target_os = "windows")]
        {
            executable
        }
    };
    let executable = executable.replace('\\', "/");
    let file_name_chunks: Vec<&str> = executable.split('/').collect();
    let len = file_name_chunks.len();
    let mut base_path= file_name_chunks.iter().as_slice()[0..len - 1]
        .join(MAIN_SEPARATOR.to_string().as_str());
    base_path.push(MAIN_SEPARATOR);
    let launcher_name = file_name_chunks.iter().last().unwrap();

    let setting = Setting {
        launcher_name: launcher_name.to_string(),
        base_path,
        port: 7878,
    };
    println!("{:#?}", setting);
    Ok(setting)
}
#[test]
fn test_get_path() {
    get_path_from_launcher();
}
