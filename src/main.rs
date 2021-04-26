#![windows_subsystem = "windows"]
#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate toml;

use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener};
use std::path::Path;
use std::sync::mpsc::Sender;

use regex::Regex;

use engines::engine_item_use;
use engines::engine_kill_self;
use engines::engine_labo;
use utils::connect_hashmap;
use utils::hashmap_to_vec;
use utils::sort;

use crate::engines::{
    engine_gacha, engine_get_part, engine_item_get, engine_reward_dungeon, engine_tsv_match,
    InnerStatics, search_dungeon_clear, search_floor,DungeonRewardElement,
};
use crate::Method::{CONNECT, DELETE, GET, HEAD, POST, PUT, TRACE};
use crate::process_manager::{construct_launcher, ProcessRequest, update};
use crate::setting::{get_path_from_launcher, Setting};
use crate::utils::{
    connect_hashmap_drs, hashmap_to_vec_drs, load_tsv, read_from_file, read_from_file2,
    read_from_file3, RewardSort, sort_drs, SortTarget,
};

mod engines;
mod mesa_inject;
mod process_manager;
mod setting;
mod utils;

#[derive(Clone)]
struct Statics {
    cache_list: HashSet<String>,
    statics: InnerStatics,
    last: usize,
}

#[derive(Debug)]
enum Method {
    GET,
    POST,
    PUT,
    HEAD,
    DELETE,
    OPTIONS,
    TRACE,
    CONNECT,
}

#[derive(Debug)]
struct HttpRequest<'a> {
    method: Method,
    uri: String,
    queries: Vec<(Cow<'a, str>, Option<Cow<'a, str>>)>,
    version: f32,
}
struct DungeonRewardStatics {
    cache_list: HashSet<String>,
    statics: HashMap<String, DungeonRewardElement>,
    last: usize,
}

impl DungeonRewardStatics {
    fn new() -> Self {
        Self {
            cache_list: Default::default(),
            statics: Default::default(),
            last: 0,
        }
    }
    //不足しているキャッシュのリストを返す
    //新しいエントリをキャッシュ済みに追加する
    fn query_cache(&mut self, list: &[String]) -> Vec<String> {
        let mut require = Vec::new();
        for entry in list {
            match self.cache_list.get(entry) {
                None => {
                    require.push(entry.clone());
                    self.cache_list.insert(entry.to_string());
                }
                //エントリーがあるならばキャッシュされている
                Some(_) => {}
            }
        }
        require
    }
    // 統計データを取得
    fn get_statics(&self) -> HashMap<String,DungeonRewardElement> {
        self.statics.clone()
    }
    //統計データを更新
    fn update_statics(&mut self, data: HashMap<String, DungeonRewardElement>) {
        for entry in data {
            let (name, qty) = entry;

            match self.statics.get_mut(&name) {
                None => {
                    self.statics.insert(name, qty);
                }
                Some(value) => {
                    *value = *value + qty;
                }
            }
        }
    }
    fn rewrite_statics(&mut self, data: HashMap<String, DungeonRewardElement>) {
        self.statics = data;
    }
    fn set_last(&mut self, last: usize) {
        self.last = last;
    }
    fn get_last(&self) -> usize {
        self.last
    }
    fn blank(&mut self) {
        self.statics.clear();
    }
}

impl Statics {
    fn new() -> Self {
        Self {
            cache_list: Default::default(),
            statics: Default::default(),
            last: 0,
        }
    }
    //不足しているキャッシュのリストを返す
    //新しいエントリをキャッシュ済みに追加する
    fn query_cache(&mut self, list: &[String]) -> Vec<String> {
        let mut require = Vec::new();
        for entry in list {
            match self.cache_list.get(entry) {
                None => {
                    require.push(entry.clone());
                    self.cache_list.insert(entry.to_string());
                }
                //エントリーがあるならばキャッシュされている
                Some(_) => {}
            }
        }
        require
    }
    // 統計データを取得
    fn get_statics(&self) -> InnerStatics {
        self.statics.clone()
    }
    //統計データを更新
    fn update_statics(&mut self, data: InnerStatics) {
        for entry in data {
            let (name, qty) = entry;
            match self.statics.get_mut(&name) {
                None => {
                    self.statics.insert(name, qty);
                }
                Some(value) => {
                    *value += qty;
                }
            }
        }
    }
    fn rewrite_statics(&mut self, data: InnerStatics) {
        self.statics = data;
    }
    fn set_last(&mut self, last: usize) {
        self.last = last;
    }
    fn get_last(&self) -> usize {
        self.last
    }
    fn blank(&mut self) {
        self.statics.clear();
    }
}

fn main() {
    //item part kill labo use gacha dungeon_item dungeon_part dungeon_kill dungeon_use burst dungeon mission shuttle dungeon_reward
    //0     1     2   3    4    5          6           7           8             9       10     11       12     13         14
    let mut statics = vec![Statics::new(); 14];
    let mut dungeon_reward_statics = DungeonRewardStatics::new();
    let config_text = fs::read_to_string("Settings.toml");
    let mut config: Option<Setting> = None;
    let mut launcher: Option<Sender<ProcessRequest>> = None;
    let mut port = 7878;

    if let Ok(config_text) = config_text {
        config = toml::from_str(&config_text).ok();
        launcher.replace(construct_launcher(config.clone().unwrap().base_path));
        port = config.as_ref().unwrap().port;
    }
    let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), port)).unwrap();
    webbrowser::open(&format!("http://localhost:{}/", port));
    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        let mut buffer = [0; 1024];
        let bytes = stream.read(&mut buffer).unwrap();
        let text = String::from_utf8_lossy(&buffer[0..bytes]);
        let request = request_parse(text.as_ref());

        match request {
            None => {}
            Some(request) => {
                let response = make_response(
                    &mut config.clone(),
                    request,
                    &mut statics,
                    &mut dungeon_reward_statics,
                    &mut launcher,
                );
                stream.write(&response);
            }
        }
    }
}

fn request_parse(text: &str) -> Option<HttpRequest> {
    let re = Regex::new(r"(.+?) (.+?) HTTP/(.+?)\r\n").unwrap();
    let cap = re.captures(text);
    match cap {
        Some(cap) => {
            let method = &cap[1];
            let uri = String::from(".");
            let uri = uri + &cap[2];
            let uri = uri.as_str();
            let mut uri_chunks = uri.split('?');
            let uri = uri_chunks.next().unwrap();
            let queries: Vec<(Cow<str>, Option<Cow<str>>)> = uri_chunks
                .map(|s| {
                    let mut query = s.split('=');
                    (
                        Cow::Owned(query.next().unwrap().to_string()),
                        query.next().map(|v| Cow::Owned(v.to_string())),
                    )
                })
                .collect();
            let uri = match uri {
                "./" => "./index.html",
                _ => &uri,
            };
            let method = match method {
                "GET" => GET,
                "POST" => POST,
                "PUT" => PUT,
                "HEAD" => HEAD,
                "DELETE" => DELETE,
                "OPTIONS" => HEAD,
                "TRACE" => TRACE,
                "CONNECT" => CONNECT,
                _ => GET,
            };

            Some(HttpRequest {
                method,
                uri: uri.to_string(),
                queries,
                version: 1.1,
            })
        }
        None => None,
    }
}

//Httpレスポンスを作成
fn make_response(
    config: &mut Option<Setting>,
    request: HttpRequest,
    statics: &mut [Statics],
    drs: &mut DungeonRewardStatics,
    launcher: &mut Option<Sender<ProcessRequest>>,
) -> Vec<u8> {
    if let Some(ref mut config) = config {
        // process query
        if let Some(query) = request.queries.get(0) {
            match query.0.as_ref() {
                "generate_config" => {
                    if let Ok(setting) = get_path_from_launcher() {
                        println!("Setting generated");
                        let config_file_content = toml::to_string(&setting).unwrap();
                        std::fs::write("./Settings.toml", config_file_content);
                    }
                }
                "inject_mesa" => {
                    println!("Injecting mesa");
                    let mut programs_path = config.base_path.clone();
                    programs_path.push_str("programs");
                    mesa_inject::inject_mesa(programs_path);
                }
                "update_c21" => {
                    let _process = update(config);
                }
                "launch_cosmic" => {
                    if let Some(ref sender) = launcher {
                        sender
                            .send(ProcessRequest::LaunchMain)
                            .expect("Failed to send launch message");
                    }
                }
                "kill_cosmic" => {
                    if let Some(ref sender) = launcher {
                        sender
                            .send(ProcessRequest::KillMain)
                            .expect("Failed to send kill message");
                    }
                }
                "launch_stage_editor" => {
                    if let Some(ref sender) = launcher {
                        sender
                            .send(ProcessRequest::LaunchStageEditor)
                            .expect("Failed to send kill message");
                    }
                }
                "kill_stage_editor" => {
                    if let Some(ref sender) = launcher {
                        sender
                            .send(ProcessRequest::KillStageEditor)
                            .expect("Failed to send kill message");
                    }
                }
                "exit" => {
                    std::process::exit(0);
                }
                _ => {}
            }
        }
        let file = File::open(&request.uri);
        println!("request:{:#?}", request);
        let mut header = Vec::from("HTTP/1.1 200 OK\r\n\r\n");
        lazy_static! {
            static ref DICTIONARIES: Vec<HashMap<String, String>> = {
                let shuttle_tsv = load_tsv("./shuttle.tsv");
                let dungeon_tsv = load_tsv("./dungeon.tsv");
                let mission_tsv = load_tsv("./mission.tsv");
                let burst_tsv = load_tsv("./burst.tsv");
                vec![burst_tsv, dungeon_tsv, mission_tsv, shuttle_tsv]
            };
        }
        let mut chat_path = config.base_path.clone();
        chat_path.push_str("chat/");
        let chat_dir_path = Path::new(&chat_path);
        let mut payload = {
            match file {
                //ファイルが存在
                Ok(mut file) => {
                    let mut buf = Vec::with_capacity(4096);
                    file.read_to_end(&mut buf);
                    buf
                }
                //存在しない
                Err(err) => {
                    eprintln!("{}", err);
                    let uri = request.uri;
                    match uri.as_str() {
                        //RTLCの機能はCGIとして実装
                        "./dungeon_reward" => {
                            let mut paths = Vec::new();//パスのリスト
                            let c21_chat_file_list = fs::read_dir(chat_dir_path).unwrap();
                            for dir_entry in c21_chat_file_list {
                                match dir_entry {
                                    Ok(dir_entry) => {
                                        let path = dir_entry.path();
                                        let path = path.to_str().unwrap();
                                        let path = path.to_string();
                                        let update = std::fs::metadata(&path).unwrap();
                                        let update = update.modified().unwrap();

                                        println!("{},{:#?}", path, update);
                                        paths.push((path, update));
                                    }
                                    Err(error) => { eprintln!("{}", error) }
                                }
                            }
                            paths.sort_by(|a, b| { a.1.cmp(&b.1) });
                            let last = paths.pop().unwrap();//最新
                            let paths: Vec<String> = paths.iter().map(|item| { item.0.clone() }).collect();

                            let need_to_load = drs.query_cache(&paths);
                            use std::sync::{Arc, Mutex};
                            let (tx, rx) = std::sync::mpsc::channel();
                            let tx = Arc::new(Mutex::new(tx));
                            let ntl = need_to_load.clone();
                            println!("{:#?}", ntl);

                            for path in ntl {
                                use std::thread;
                                let tx = tx.clone();
                                thread::spawn(move || {
                                    let texts = read_from_file(&path);
                                    let data = engine_reward_dungeon(&texts, 0);
                                    tx.lock().unwrap().send(data);
                                }
                                );
                            }
                            if !need_to_load.is_empty() {
                                for (id, rcv) in rx.iter().enumerate() {
                                    println!("id: {}, len: {}", id + 1, need_to_load.len());
                                    if id + 1 == need_to_load.len() {
                                        break;
                                    }
                                    drs.update_statics(rcv);
                                }
                            }
                            let (items, lds) = {
                                let texts = read_from_file(last.0);
                                (drs.get_statics(), engine_reward_dungeon(&texts, 0))
                            };
                            let set = connect_hashmap_drs(items, lds);
                            //ITEMSとLDSを統合して出力
                            let mut droptable = "<!DOCTYPE html><html><head>
                        <script>
        function reload() {
        location.reload(true);
        }
          setTimeout(reload, 30000);
          </script><meta charset=\"UTF-8\"> <title>C21Counter_rs</title></head><body><table border=\"1\" cellspacing=\"0\" cellpadding=\"5\" bordercolor=\"#333333\">".to_string();
                            let table_row = "<tr><th>名前</th><th>報酬</th><th>売却</th></tr>";
                            droptable.push_str(table_row);
                            let mut vector = hashmap_to_vec_drs(&set);
                            sort_drs(&mut vector, RewardSort::Reward,SortTarget::NAME, true);
                            for row in vector {
                                let (key, val) = row;
                                let row_string = format!("<tr><td>{}</td><td>{}</td><td>{}</td></tr>\r\n", key, val.0,val.1);
                                droptable.push_str(&row_string);
                            }
                            droptable.push_str("</table></body></html>");
                            droptable.into_bytes()
                        }
                        "./items" | "./parts" | "./kills" | "./labo" | "./use" | "./gacha" | "./dungeon_clear" | "./burst" | "./mission" | "./shuttle" => {
                            let mut paths = Vec::new();//パスのリスト
                            let c21_chat_file_list = fs::read_dir(chat_dir_path).unwrap();
                            for dir_entry in c21_chat_file_list {
                                match dir_entry {
                                    Ok(dir_entry) => {
                                        let path = dir_entry.path();
                                        let path = path.to_str().unwrap();
                                        let path = path.to_string();
                                        let update = std::fs::metadata(&path).unwrap();
                                        let update = update.modified().unwrap();

                                        println!("{},{:#?}", path, update);
                                        paths.push((path, update));
                                    }
                                    Err(error) => { eprintln!("{}", error) }
                                }
                            }
                            paths.sort_by(|a, b| { a.1.cmp(&b.1) });
                            let last = paths.pop().unwrap();//最新
                            let paths: Vec<String> = paths.iter().map(|item| { item.0.clone() }).collect();
                            //ここはuriによってふるまいをかえる
                            let need_to_load = match uri.as_str() {
                                "./items" => { statics[0].query_cache(&paths) }
                                "./parts" => { statics[1].query_cache(&paths) }
                                "./kills" => { statics[2].query_cache(&paths) }
                                "./labo" => { statics[3].query_cache(&paths) }
                                "./use" => { statics[4].query_cache(&paths) }
                                "./gacha" => { statics[5].query_cache(&paths) }

                                "./burst" => { statics[10].query_cache(&paths) }
                                "./dungeon_clear" => { statics[11].query_cache(&paths) }
                                "./mission" => { statics[12].query_cache(&paths) }
                                "./shuttle" => { statics[13].query_cache(&paths) }
                                _ => { statics[0].query_cache(&paths) }
                            };

                            //更新が必要なものをリストアップ

                            use std::sync::{Arc, Mutex};
                            let (tx, rx) = std::sync::mpsc::channel();
                            let tx = Arc::new(Mutex::new(tx));
                            let ntl = need_to_load.clone();
                            println!("{:#?}", ntl);

                            for path in ntl {
                                use std::thread;
                                let tx = tx.clone();
                                let uri = uri.clone();
                                thread::spawn(move || {
                                    match uri.as_str() {
                                        "./items" => {
                                            let texts = read_from_file(path);
                                            let data = engine_item_get(&texts, 0);
                                            tx.lock().unwrap().send(data);
                                        }
                                        "./parts" => {
                                            let texts = read_from_file(path);
                                            let data = engine_get_part(&texts, 0);
                                            tx.lock().unwrap().send(data);
                                        }
                                        "./kills" => {
                                            let texts = read_from_file(path);
                                            let data = engine_kill_self(&texts, 0);
                                            tx.lock().unwrap().send(data);
                                        }
                                        "./labo" => {
                                            let texts = read_from_file2(path);
                                            let data = engine_labo(&texts, 0);
                                            tx.lock().unwrap().send(data);
                                        }
                                        "./use" => {
                                            let texts = read_from_file(path);
                                            let data = engine_item_use(&texts, 0);
                                            tx.lock().unwrap().send(data);
                                        }
                                        "./gacha" => {
                                            let texts = read_from_file(path);
                                            let data = engine_gacha(&texts, 0);
                                            tx.lock().unwrap().send(data);
                                        }
                                        "./burst" => {
                                            let texts = read_from_file3(path);
                                            let data = engine_tsv_match(&texts, &DICTIONARIES[0], 0);
                                            tx.lock().unwrap().send(data);
                                        }
                                        "./dungeon_clear" => {
                                            let texts = read_from_file3(path);
                                            let data = engine_tsv_match(&texts, &DICTIONARIES[1], 0);
                                            tx.lock().unwrap().send(data);
                                        }
                                        "./mission" => {
                                            let texts = read_from_file3(path);
                                            let data = engine_tsv_match(&texts, &DICTIONARIES[2], 0);
                                            tx.lock().unwrap().send(data);
                                        }
                                        "./shuttle" => {
                                            let texts = read_from_file3(path);
                                            let data = engine_tsv_match(&texts, &DICTIONARIES[3], 0);
                                            tx.lock().unwrap().send(data);
                                        }
                                        _ => {}
                                    };
                                });
                            }
                            if !need_to_load.is_empty() {
                                for (id, rcv) in rx.iter().enumerate() {
                                    println!("id: {}, len: {}", id + 1, need_to_load.len());
                                    if id + 1 == need_to_load.len() {
                                        break;
                                    }
                                    match uri.as_str() {
                                        "./items" => {
                                            statics[0].update_statics(rcv);
                                        }
                                        "./parts" => {
                                            statics[1].update_statics(rcv);
                                        }
                                        "./kills" => {
                                            statics[2].update_statics(rcv);
                                        }
                                        "./labo" => {
                                            statics[3].update_statics(rcv);
                                        }
                                        "./use" => {
                                            statics[4].update_statics(rcv);
                                        }
                                        "./gacha" => {
                                            statics[5].update_statics(rcv);
                                        }
                                        "./burst" => {
                                            statics[10].update_statics(rcv);
                                        }
                                        "./dungeon_clear" => {
                                            statics[11].update_statics(rcv);
                                        }
                                        "./mission" => {
                                            statics[12].update_statics(rcv);
                                        }
                                        "./shuttle" => {
                                            statics[13].update_statics(rcv);
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            let (items, lds) = match uri.as_str() {
                                "./items" => {
                                    let texts = read_from_file(last.0);
                                    (statics[0].get_statics(), engine_item_get(&texts, 0))
                                }
                                "./parts" => {
                                    let texts = read_from_file(last.0);
                                    (statics[1].get_statics(), engine_get_part(&texts, 0))
                                }
                                "./kills" => {
                                    let texts = read_from_file(last.0);
                                    (statics[2].get_statics(), engine_kill_self(&texts, 0))
                                }
                                "./labo" => {
                                    let texts = read_from_file2(last.0);
                                    (statics[3].get_statics(), engine_labo(&texts, 0))
                                }
                                "./use" => {
                                    let texts = read_from_file(last.0);
                                    (statics[4].get_statics(), engine_item_use(&texts, 0))
                                }
                                "./gacha" => {
                                    let texts = read_from_file(last.0);
                                    (statics[5].get_statics(), engine_gacha(&texts, 0))
                                }
                                "./burst" => {
                                    let texts = read_from_file3(last.0);
                                    (statics[10].get_statics(), engine_tsv_match(&texts, &DICTIONARIES[0], 0))
                                }
                                "./dungeon_clear" => {
                                    let texts = read_from_file3(last.0);
                                    (statics[11].get_statics(), engine_tsv_match(&texts, &DICTIONARIES[1], 0))
                                }
                                "./mission" => {
                                    let texts = read_from_file3(last.0);
                                    (statics[12].get_statics(), engine_tsv_match(&texts, &DICTIONARIES[2], 0))
                                }
                                "./shuttle" => {
                                    let texts = read_from_file3(last.0);
                                    (statics[13].get_statics(), engine_tsv_match(&texts, &DICTIONARIES[3], 0))
                                }
                                _ => {
                                    let texts = read_from_file(last.0);
                                    (statics[0].get_statics(), engine_item_get(&texts, 0))
                                }
                            };
                            let set = connect_hashmap(items, lds);
                            //ITEMSとLDSを統合して出力
                            let mut droptable = "<!DOCTYPE html><html><head>
                        <script>
        function reload() {
        location.reload(true);
        }
          setTimeout(reload, 30000);
          </script><meta charset=\"UTF-8\"> <title>C21Counter_rs</title></head><body><table border=\"1\" width=\"200\" cellspacing=\"0\" cellpadding=\"5\" bordercolor=\"#333333\">".to_string();
                            let table_row = "<tr><th>名前</th><th>個数</th></tr>";
                            droptable.push_str(table_row);
                            let mut vector = hashmap_to_vec(&set);
                            sort(&mut vector, SortTarget::NAME, true);
                            for row in vector {
                                let (key, val) = row;
                                let row_string = format!("<tr><td>{}</td><td>{}</td></tr>\r\n", key, val);
                                droptable.push_str(&row_string);
                            }
                            droptable.push_str("</table></body></html>");
                            droptable.into_bytes()
                        }
                        "./dungeon" => unsafe {
                            let mut paths = Vec::new();//パスのリスト
                            static mut ENTERED: bool = false;
                            static mut LAST_FLOOR_GATE: usize = 0;
                            static mut LAST_CLEAR: usize = 0;
                            static mut DONE_LINE: Option<usize> = None;
                            let c21_chat_file_list = fs::read_dir(chat_dir_path).unwrap();
                            for dir_entry in c21_chat_file_list {
                                match dir_entry {
                                    Ok(dir_entry) => {
                                        let path = dir_entry.path();
                                        let path = path.to_str().unwrap();
                                        let path = path.to_string();
                                        let update = std::fs::metadata(&path).unwrap();
                                        let update = update.modified().unwrap();

                                        //   println!("{},{:#?}", path, update);
                                        paths.push((path, update));
                                    }
                                    Err(error) => { eprintln!("{}", error) }
                                }
                            }
                            paths.sort_by(|a, b| { a.1.cmp(&b.1) });
                            let last = paths.pop().unwrap();//最新

                            let texts = read_from_file(&last.0);
                            if DONE_LINE.is_none() {
                                DONE_LINE = Some(texts.len());
                            }
                            let from = search_floor(&texts, DONE_LINE.unwrap());
                            let last_clear_stack = search_dungeon_clear(&texts, DONE_LINE.unwrap());
                            //ダンジョン侵入判定
                            if (last_clear_stack < from) & !ENTERED {
                                ENTERED = true;
                                LAST_FLOOR_GATE = from.unwrap();
                            }

                            match last_clear_stack {
                                None => {}
                                Some(line) => {
                                    LAST_CLEAR = line
                                }
                            }


                            if (LAST_CLEAR > LAST_FLOOR_GATE) & ENTERED {
                                // dump current try
                                // generate file name and create
                                let path = Path::new(&last.0);
                                let stem = path.file_stem().unwrap();
                                let stem = stem.to_str().unwrap();

                                if !Path::new("./dungeon_statics").exists() {
                                    std::fs::create_dir("./dungeon_statics");
                                }
                                let file_name = format!("./dungeon_statics/{}@{}_{}.html", stem, LAST_FLOOR_GATE, LAST_CLEAR);
                                println!("write to  {}", file_name);
                                let mut file = std::fs::File::create(file_name).unwrap();
                                // generate  html
                                let mut table = "<!DOCTYPE html>\n\
                                <html>\n\
                                    <head>\n\
                                        <meta charset=\"UTF-8\">\n\
                                        <link href=\"./style.css\" rel=\"stylesheet\" type=\"text/css\">\n
                                    </head>\n
                                    <body>\n
                                        <h1>ダンジョン内カウント</h1>\n".to_string();
                                let mut captions = vec!["アイテム取得", "パーツ取得", "キル", "アイテム使用"];
                                captions.reverse();
                                for data in &statics[6..10] {
                                    let table_row = "   <div class=\"hbox\">\
                                        <table border=\"1\" width=\"200\" cellspacing=\"0\" cellpadding=\"5\" bordercolor=\"#333333\">";
                                    let caption = format!("      <caption>{}</caption>\n", captions.pop().unwrap());
                                    table.push_str(table_row);
                                    table.push_str(&caption);
                                    table.push_str("            <tr><th>名前</th><th>個数</th></tr>\n");
                                    let mut vector = hashmap_to_vec(&data.statics);
                                    sort(&mut vector, SortTarget::NAME, true);
                                    for row in vector {
                                        let (key, val) = row;
                                        let row_string = format!("        <tr><td>{}</td><td>{}</td></tr>\n", key, val);
                                        table.push_str(&row_string);
                                    }
                                    table.push_str("        </table>\
                                    </div>");
                                }
                                table.push_str("\
                                </body>\
                            </html>");
                                file.write_all(table.as_bytes()).unwrap();
                                file.flush().unwrap();
                                //
                                ENTERED = false;
                                statics[6].blank();
                                statics[7].blank();
                                statics[8].blank();
                                statics[9].blank();
                            }

                            if ENTERED {
                                println!("Rewrite");
                                statics[6].rewrite_statics(engine_item_get(&texts, LAST_FLOOR_GATE));
                                statics[7].rewrite_statics(engine_get_part(&texts, LAST_FLOOR_GATE));
                                statics[8].rewrite_statics(engine_kill_self(&texts, LAST_FLOOR_GATE));
                                statics[9].rewrite_statics(engine_item_use(&texts, LAST_FLOOR_GATE));
                                DONE_LINE = Some(texts.len());
                            }
                            println!("ENTERED: {} DONE_LINE: {} LAST_FLOOR_GATE: {} LAST_CLEAR: {}", ENTERED, DONE_LINE.unwrap(), LAST_FLOOR_GATE, LAST_CLEAR);
                            let mut table = "<!DOCTYPE html><html><head><meta charset=\"UTF-8\">\
<link href=\"./style.css\" rel=\"stylesheet\" type=\"text/css\">
<script>
        function reload() {
        location.reload(true);
        }
          setTimeout(reload, 3000);
          </script>

          </head><body><h1>ダンジョン内カウント</h1>".to_string();
                            let mut captions = vec!["アイテム取得", "パーツ取得", "キル", "アイテム使用"];
                            captions.reverse();
                            for data in &statics[6..10] {
                                let table_row = "<div class=\"hbox\"><table border=\"1\" width=\"200\" cellspacing=\"0\" cellpadding=\"5\" bordercolor=\"#333333\">";
                                let caption = format!("<caption>{}</caption>", captions.pop().unwrap());
                                table.push_str(table_row);
                                table.push_str(&caption);
                                table.push_str("<tr><th>名前</th><th>個数</th></tr>");
                                let mut vector = hashmap_to_vec(&data.statics);
                                sort(&mut vector, SortTarget::NAME, true);
                                for row in vector {
                                    let (key, val) = row;
                                    let row_string = format!("<tr><td>{}</td><td>{}</td></tr>", key, val);
                                    table.push_str(&row_string);
                                }
                                table.push_str("</table></div>");
                            }
                            table.push_str("</body></html>");
                            table.into_bytes()
                        }

                        "./floor" => {
                            let mut paths = Vec::new();//パスのリスト
                            let c21_chat_file_list = fs::read_dir(chat_dir_path).unwrap();
                            for dir_entry in c21_chat_file_list {
                                match dir_entry {
                                    Ok(dir_entry) => {
                                        let path = dir_entry.path();
                                        let path = path.to_str().unwrap();
                                        let path = path.to_string();
                                        let update = std::fs::metadata(&path).unwrap();
                                        let update = update.modified().unwrap();

                                        //     println!("{},{:#?}", path, update);
                                        paths.push((path, update));
                                    }
                                    Err(error) => { eprintln!("{}", error) }
                                }
                            }
                            paths.sort_by(|a, b| { a.1.cmp(&b.1) });
                            let last = paths.pop().unwrap();//最新
                            let texts = read_from_file(last.0);
                            let from = search_floor(&texts, 0);
                            match from {
                                None => {
                                    Vec::from("<!DOCTYPE html><html><head><script>
        function reload() {
        location.reload(true);
            }
          setTimeout(reload, 3000);
          </script><meta charset=\"UTF-8\">\
<link href=\"./style.css\" rel=\"stylesheet\" type=\"text/css\">
          </head><body>
          <h1>フロア内カウント</h1>
          <h1>まだダンジョンなどに入っていません</h1>
          </body>")
                                }
                                Some(from) => {
                                    let mut lds = Vec::new();
                                    lds.push(engine_item_get(&texts, from));
                                    lds.push(engine_get_part(&texts, from));
                                    lds.push(engine_item_use(&texts, from));
                                    lds.push(engine_kill_self(&texts, from));
                                    let mut table = "<!DOCTYPE html><html><head><script>
        function reload() {
        location.reload(true);
            }
          setTimeout(reload, 3000);
          </script><meta charset=\"UTF-8\">\
<link href=\"./style.css\" rel=\"stylesheet\" type=\"text/css\">
          </head><body><h1>フロア内カウント</h1>".to_string();
                                    let mut captions = vec!["アイテム取得", "パーツ取得", "アイテム使用", "キル"];
                                    captions.reverse();
                                    for data in lds {
                                        let table_row = "<div class=\"hbox\"><table border=\"1\" width=\"200\" cellspacing=\"0\" cellpadding=\"5\" bordercolor=\"#333333\">";
                                        let caption = format!("<caption>{}</caption>", captions.pop().unwrap());
                                        table.push_str(table_row);
                                        table.push_str(&caption);
                                        table.push_str("<tr><th>名前</th><th>個数</th></tr>");
                                        let mut vector = hashmap_to_vec(&data);
                                        sort(&mut vector, SortTarget::NAME, true);
                                        for row in vector {
                                            let (key, val) = row;
                                            let row_string = format!("<tr><td>{}</td><td>{}</td></tr>", key, val);
                                            table.push_str(&row_string);
                                        }
                                        table.push_str("</table></div>");
                                    }
                                    table.push_str("</body></html>");
                                    table.into_bytes()
                                }
                            }
                        }
                        "./config" => {
                            Vec::from("<html><title>Now Constructing</title><body><h1>Now Constructing</h1></body></html>")
                        }
                        _ => Vec::from(
                           include_str!("not_found.html")
                        ),
                    }
                }
            }
        };
        let mut page = Vec::with_capacity(4096);
        page.append(&mut header);
        page.append(&mut payload);
        page
    } else {
        //設定ファイルが読めないとき
        let mut header = Vec::from("HTTP/1.1 200 OK\r\n\r\n");
        let mut buffer = Vec::with_capacity(512);

        match get_path_from_launcher() {
            Ok(setting) => {
                //write setting
                println!("Setting generated");
                let config_file_content = toml::to_string(&setting).unwrap();
                std::fs::write("./Settings.toml", config_file_content);
                //config.replace(setting);
                let mut old_position = Some(setting);
                std::mem::swap(config, &mut old_position);
                println!("{:#?}", old_position);
                println!("{:#?}", config);
                let mut config_not_found_page = File::open("config_not_found.html").unwrap();
                config_not_found_page.read_to_end(&mut buffer);
            }
            Err(_) => {
                let mut please_start_launcher_page =
                    File::open("please_start_launcher.html").unwrap();
                please_start_launcher_page.read_to_end(&mut buffer);
            }
        }

        header.append(&mut buffer);

        header
    }
}
