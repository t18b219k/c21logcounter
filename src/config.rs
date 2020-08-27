use serde_derive::Deserialize;

mod config {
    struct Config {
        log_folder: Option<String>,
        //ログが格納されているフォルダー
        check_period: Option<u32>,
        //更新間隔
        port: Option<u32>,//サーバのポート
    }
}
