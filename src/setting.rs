use std::fs::File;
use std::io::Write;
use serde::{Serialize,Deserialize};
#[derive(Debug, Serialize,Deserialize)]
struct Setting {
    launcher_path: String,
    chat_path: String,
    base_path: String,
}
///Write default setting
#[test]
fn write_default_setting() {

    let setting_unix = Setting {
        launcher_path: ".wine/drive_c/CyberStep/c21/c21.exe".to_string(),
        chat_path: ".wine/drive_c/CyberStep/c21/chat".to_string(),
        base_path: ".wine/drive_c/CyberStep/c21/".to_string(),
    };

    let setting_windows = Setting {
        launcher_path: "C:\\CyberStep\\C21\\c21.exe".to_string(),
        chat_path: "C:\\CyberStep\\C21\\chat".to_string(),
        base_path: "C:\\CyberStep\\C21\\".to_string(),
    };
    let mut file_windows = File::create("Setting_windows.toml").unwrap();
    let mut file_unix=File::create("Setting_unix.toml").unwrap();
    let toml_unix= toml::to_string(&setting_unix).unwrap();
    let toml_windows=toml::to_string(&setting_windows).unwrap();
    write!(file_unix,"{}",toml_unix);
    write!(file_windows,"{}",toml_windows);
    file_windows.flush();
    file_unix.flush();
}

//
fn get_path_from_launcher() {
    //generate wine prefix
    #[cfg(not(target_os = "windows"))]
    let drive_c = {
        let wine_prefix = option_env!("WINEPREFIX");
        let wine_prefix = wine_prefix.unwrap_or("~/.wine/");
        let wine_prefix = wine_prefix.replace("~", option_env!("HOME").unwrap());
        let mut drive_c = wine_prefix.to_string();
        drive_c.push_str("drive_c/");
        drive_c
    };
}
