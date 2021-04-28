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
    search_dungeon_clear, search_floor, DungeonRewardElement, InnerStatics,
};
use crate::process_manager::{construct_launcher, update, ProcessRequest};
use crate::setting::{get_path_from_launcher, Setting};
use crate::utils::{
    connect_hashmap_drs, hashmap_to_vec_drs, load_tsv, read_from_file, read_from_file2,
    read_from_file3, sort_drs, RewardSort, SortTarget,
};
use crate::Method::{CONNECT, DELETE, GET, HEAD, POST, PUT, TRACE};

use sailfish::TemplateOnce;

#[derive(TemplateOnce)]
#[template(path = "general.stpl")]
struct GeneralStaticsTemplate {
    name: String,
    statics: Vec<(String, isize)>,
}

#[derive(TemplateOnce)]
#[template(path = "dungeon_reward.stpl")]
struct GenerateDungeonRewardStaticsTemplate {
    name: String,
    statics: Vec<(String, DungeonRewardElement)>,
}
#[derive(TemplateOnce)]
#[template(path = "inner_floor.stpl")]
struct InFloorStaticsTemplate {
    name: String,
    set_of_statics: Vec<GeneralStaticsTemplate>,
}
mod engines;
mod mesa_inject;
mod process_manager;
mod setting;
mod statics_address;
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
#[derive(Default)]
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
    fn get_statics(&self) -> HashMap<String, DungeonRewardElement> {
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

struct DungeonContext {
    last_gate: usize,
    entered: bool,
    last_clear: usize,
    done_line: Option<usize>,
}

/// contain all statics log contents
/// and configs
/// logs are splatted to each line

struct Context {
    config: Option<Setting>,
    launcher: Option<Sender<ProcessRequest>>,
    log_cache: HashMap<String, Vec<String>>,
    general_statics: Vec<Statics>,
    dungeon_reward_statics: DungeonRewardStatics,
    dungeon_save: DungeonContext,
    current_updating_file: String,
    port: u16,
}

use crate::statics_address::StaticsAddress;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use std::sync::Mutex;

fn main() {
    let config_text = fs::read_to_string("Settings.toml");
    let mut context = Context {
        config: None,
        launcher: None,
        log_cache: Default::default(),
        general_statics: vec![Statics::new(); 14],
        dungeon_reward_statics:Default::default(),
        dungeon_save: DungeonContext {
            last_gate: 0,
            entered: false,
            last_clear: 0,
            done_line: None,
        },
        current_updating_file: "".to_string(),
        port: 7878,
    };
    //設定読み込み
    if let Ok(config_text) = config_text {
        context.config = toml::from_str(&config_text).ok();
        context.launcher.replace(construct_launcher(
            context.config.clone().unwrap().base_path,
        ));
        context.port = context.config.as_ref().unwrap().port;
    }
    let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), context.port)).unwrap();
    webbrowser::open(&format!("http://localhost:{}/", context.port));
    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        let mut buffer = [0; 1024];
        let bytes = stream.read(&mut buffer).unwrap();
        let text = String::from_utf8_lossy(&buffer[0..bytes]);
        let request = request_parse(text.as_ref());

        match request {
            None => {}
            Some(request) => {
                let response = make_response(request, &mut context);
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

fn search_latest_log_file<P: AsRef<Path>>(chat_dir_path: P) -> (String, Vec<String>) {
    let mut paths = Vec::new(); //パスのリスト
    let c21_chat_file_list = fs::read_dir(chat_dir_path).unwrap();
    for dir_entry in c21_chat_file_list {
        match dir_entry {
            Ok(dir_entry) => {
                let path = dir_entry.path();
                let path = path.to_str().unwrap();
                let path = path.to_string();
                let update = std::fs::metadata(&path).unwrap();
                let update = update.modified().unwrap();
                #[cfg(debug_assertions)]
                println!("{},{:#?}", path, update);
                paths.push((path, update));
            }
            Err(error) => {
                eprintln!("{}", error)
            }
        }
    }
    paths.sort_by(|a, b| a.1.cmp(&b.1));
    let last = paths.pop().unwrap(); //最新
    let paths: Vec<String> = paths.iter().map(|item| item.0.clone()).collect();
    (last.0, paths)
}

//Httpレスポンスを作成
fn make_response(request: HttpRequest, context: &mut Context) -> Vec<u8> {
    if let Some(ref mut config) = context.config {
        // process query
        if let Some(query) = request.queries.get(0) {
            match query.0.as_ref() {
                "generate_config" => {
                    if let Ok(setting) = get_path_from_launcher() {
                        #[cfg(debug_assertions)]
                        println!("Setting generated");
                        let config_file_content = toml::to_string(&setting).unwrap();
                        std::fs::write("./Settings.toml", config_file_content);
                    }
                }
                "inject_mesa" => {
                    #[cfg(debug_assertions)]
                    println!("Injecting mesa");
                    let mut programs_path = config.base_path.clone();
                    programs_path.push_str("programs");
                    mesa_inject::inject_mesa(programs_path);
                }
                "update_c21" => {
                    let _process = update(config);
                }
                "launch_cosmic" => {
                    if let Some(ref mut sender) = context.launcher {
                        sender
                            .send(ProcessRequest::LaunchMain)
                            .expect("Failed to send launch message");
                    }
                }
                "kill_cosmic" => {
                    if let Some(ref mut sender) = context.launcher {
                        sender
                            .send(ProcessRequest::KillMain)
                            .expect("Failed to send kill message");
                    }
                }
                "launch_stage_editor" => {
                    if let Some(ref mut sender) = context.launcher {
                        sender
                            .send(ProcessRequest::LaunchStageEditor)
                            .expect("Failed to send kill message");
                    }
                }
                "kill_stage_editor" => {
                    if let Some(ref mut sender) = context.launcher {
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
        #[cfg(debug_assertions)]
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
                        //機能はCGIとして実装
                        "./dungeon_reward" => {
                            let (last, paths) = search_latest_log_file(chat_dir_path);
                            let need_to_load = context.dungeon_reward_statics.query_cache(&paths);
                            use std::sync::Arc;
                            let (tx, rx) = std::sync::mpsc::channel();
                            let tx = Arc::new(Mutex::new(tx));
                            let ntl = need_to_load.clone();
                            #[cfg(debug_assertions)]
                            println!("{:#?}", ntl);

                            for path in ntl {
                                use std::thread;
                                let tx = tx.clone();
                                thread::spawn(move || {
                                    let texts = read_from_file(&path);
                                    let data = engine_reward_dungeon(&texts, 0);
                                    tx.lock().unwrap().send(data);
                                });
                            }
                            if !need_to_load.is_empty() {
                                for (id, rcv) in rx.iter().enumerate() {
                                    #[cfg(debug_assertions)]
                                    println!("id: {}, len: {}", id + 1, need_to_load.len());
                                    if id + 1 == need_to_load.len() {
                                        break;
                                    }
                                    context.dungeon_reward_statics.update_statics(rcv);
                                }
                            }
                            let (items, lds) = {
                                let texts = read_from_file(last);
                                (context.dungeon_reward_statics.get_statics(), engine_reward_dungeon(&texts, 0))
                            };
                            let set = connect_hashmap_drs(items, lds);
                            let mut vector = hashmap_to_vec_drs(&set);
                            sort_drs(&mut vector, RewardSort::Reward, SortTarget::NAME, true);
                            //ITEMSとLDSを統合して出力
                            let ctx = GenerateDungeonRewardStaticsTemplate {
                                name: "ダンジョン報酬".to_string(),
                                statics: vector,
                            };
                            let text = ctx.render_once().unwrap();
                            text.into_bytes()
                        }
                        "./items" | "./parts" | "./kills" | "./labo" | "./use" | "./gacha"
                        | "./dungeon_clear" | "./burst" | "./mission" | "./shuttle" => {
                            let (last, paths) = search_latest_log_file(chat_dir_path);
                            let statics_address = StaticsAddress::from_url(uri.as_str()).unwrap();
                            let need_to_load =
                                context.general_statics[statics_address.as_uint()].query_cache(&paths);
                            //更新が必要なものをリストアップ

                            use std::sync::Arc;
                            let (tx, rx) = std::sync::mpsc::channel();
                            let tx = Arc::new(Mutex::new(tx));
                            let ntl = need_to_load.clone();
                            #[cfg(debug_assertions)]
                            println!("{:#?}", ntl);

                            for path in ntl {
                                use std::thread;
                                let tx = tx.clone();
                                let uri = uri.clone();
                                thread::spawn(move || match statics_address {
                                    StaticsAddress::Item => {
                                        let texts = read_from_file(path);
                                        let data = engine_item_get(&texts, 0);
                                        tx.lock().unwrap().send(data);
                                    }
                                    StaticsAddress::ItemUse => {
                                        let texts = read_from_file(path);
                                        let data = engine_item_use(&texts, 0);
                                        tx.lock().unwrap().send(data);
                                    }
                                    StaticsAddress::Parts => {
                                        let texts = read_from_file(path);
                                        let data = engine_get_part(&texts, 0);
                                        tx.lock().unwrap().send(data);
                                    }
                                    StaticsAddress::Kill => {
                                        let texts = read_from_file(path);
                                        let data = engine_kill_self(&texts, 0);
                                        tx.lock().unwrap().send(data);
                                    }
                                    StaticsAddress::Burst
                                    | StaticsAddress::Mission
                                    | StaticsAddress::DungeonClear
                                    | StaticsAddress::Shuttle => {
                                        let texts = read_from_file3(path);
                                        let data = engine_tsv_match(
                                            &texts,
                                            &DICTIONARIES
                                                [statics_address.as_dictionary_index().unwrap()],
                                            0,
                                        );
                                        tx.lock().unwrap().send(data);
                                    }
                                    StaticsAddress::Lab => {
                                        let texts = read_from_file2(path);
                                        let data = engine_labo(&texts, 0);
                                        tx.lock().unwrap().send(data);
                                    }
                                    StaticsAddress::Gacha => {
                                        let texts = read_from_file(path);
                                        let data = engine_gacha(&texts, 0);
                                        tx.lock().unwrap().send(data);
                                    }
                                    _ => {}
                                });
                            }
                            if !need_to_load.is_empty() {
                                for (id, rcv) in rx.iter().enumerate() {
                                    #[cfg(debug_assertions)]
                                    println!("id: {}, len: {}", id + 1, need_to_load.len());
                                    if id + 1 == need_to_load.len() {
                                        break;
                                    }
                                    context.general_statics[statics_address.as_uint()].update_statics(rcv);
                                }
                            }
                            let items = context.general_statics[statics_address.as_uint()].get_statics();
                            let updating = match statics_address {
                                StaticsAddress::Item => {
                                    let texts = read_from_file(last);
                                    engine_item_get(&texts, 0)
                                }
                                StaticsAddress::ItemUse => {
                                    let texts = read_from_file(last);
                                    engine_item_use(&texts, 0)
                                }
                                StaticsAddress::Parts => {
                                    let texts = read_from_file(last);
                                    engine_get_part(&texts, 0)
                                }
                                StaticsAddress::Kill => {
                                    let texts = read_from_file(last);
                                    engine_kill_self(&texts, 0)
                                }
                                StaticsAddress::Burst
                                | StaticsAddress::Mission
                                | StaticsAddress::DungeonClear
                                | StaticsAddress::Shuttle => {
                                    let texts = read_from_file3(last);
                                    engine_tsv_match(
                                        &texts,
                                        &DICTIONARIES
                                            [statics_address.as_dictionary_index().unwrap()],
                                        0,
                                    )
                                }
                                StaticsAddress::Lab => {
                                    let texts = read_from_file2(last);
                                    engine_labo(&texts, 0)
                                }
                                StaticsAddress::Gacha => {
                                    let texts = read_from_file(last);
                                    engine_gacha(&texts, 0)
                                }
                                _ => {
                                    unreachable!()
                                }
                            };

                            //ITEMSとLDSを統合して出力
                            let set = connect_hashmap(items, updating);
                            let mut vector = hashmap_to_vec(&set);
                            sort(&mut vector, SortTarget::NAME, true);
                            let ctx = GeneralStaticsTemplate {
                                name: statics_address.to_string(),
                                statics: vector,
                            };
                            ctx.render_once().unwrap().into_bytes()
                        }
                        "./dungeon" => {
                            let (last, paths) = search_latest_log_file(chat_dir_path);
                            let texts = read_from_file(&last);
                            let from = search_floor(&texts, context.dungeon_save.done_line.unwrap_or(0));

                            let last_clear_stack = search_dungeon_clear(
                                &texts,
                                context.dungeon_save.done_line.unwrap_or(0),
                            );
                            //ダンジョン侵入判定
                            //侵入してない状態で最新のダンジョンクリアよりフロアゲートの起動があとならば侵入したと判定する
                            if (last_clear_stack < from) & !context.dungeon_save.entered {
                                context.dungeon_save.entered = true;
                                context.dungeon_save.last_gate = from.unwrap();
                            }

                            //LAST_CLEARをlast_clear_stackがSomeならば更新する
                            if let Some(line) = last_clear_stack {
                                context.dungeon_save.last_clear = line;
                            }
                            //write_statics 関数の定義
                            let write_statics = |statics: &[Statics]| -> String {
                                let ctx = InFloorStaticsTemplate {
                                    name: "ダンジョン内カウント".to_string(),
                                    set_of_statics: vec![
                                        GeneralStaticsTemplate {
                                            name: "アイテム取得".to_string(),
                                            statics: {
                                                let mut vector = hashmap_to_vec(
                                                    &statics[StaticsAddress::ItemDungeon.as_uint()]
                                                        .statics,
                                                );
                                                sort(&mut vector, SortTarget::NAME, true);
                                                vector
                                            },
                                        },
                                        GeneralStaticsTemplate {
                                            name: "パーツ取得".to_string(),
                                            statics: {
                                                let mut vector = hashmap_to_vec(
                                                    &statics[StaticsAddress::PartDungeon.as_uint()]
                                                        .statics,
                                                );
                                                sort(&mut vector, SortTarget::NAME, true);
                                                vector
                                            },
                                        },
                                        GeneralStaticsTemplate {
                                            name: "キル".to_string(),
                                            statics: {
                                                let mut vector = hashmap_to_vec(
                                                    &statics[StaticsAddress::KillDungeon.as_uint()]
                                                        .statics,
                                                );
                                                sort(&mut vector, SortTarget::NAME, true);
                                                vector
                                            },
                                        },
                                        GeneralStaticsTemplate {
                                            name: "アイテム使用".to_string(),
                                            statics: {
                                                let mut vector = hashmap_to_vec(
                                                    &statics
                                                        [StaticsAddress::ItemUseDungeon.as_uint()]
                                                    .statics,
                                                );
                                                sort(&mut vector, SortTarget::NAME, true);
                                                vector
                                            },
                                        },
                                    ],
                                };
                                ctx.render_once().unwrap()
                            };

                            //侵入した状態ならば統計の更新処理を行う
                            if context.dungeon_save.entered {
                                #[cfg(debug_assertions)]
                                println!("Rewrite");

                                if let Some(done_line) = context.dungeon_save.done_line {
                                    context.general_statics[StaticsAddress::ItemDungeon.as_uint()]
                                        .update_statics(engine_item_get(&texts, done_line));
                                    context.general_statics[StaticsAddress::PartDungeon.as_uint()]
                                        .update_statics(engine_get_part(&texts, done_line));
                                    context.general_statics[StaticsAddress::KillDungeon.as_uint()]
                                        .update_statics(engine_kill_self(&texts, done_line));
                                    context.general_statics[StaticsAddress::ItemUseDungeon.as_uint()]
                                        .update_statics(engine_item_use(&texts, done_line));
                                    context.dungeon_save.done_line = Some(texts.len());
                                }
                            }
                            //侵入した状態で最後のフロアゲートの起動よりクリアのほうがあとならば報酬を受け取っていると判定できる.
                            //ここまでの統計をファイルに書き出す
                            //ダンジョンからの退出処理を行う
                            if (context.dungeon_save.last_clear > context.dungeon_save.last_gate)
                                & context.dungeon_save.entered
                            {
                                let path = Path::new(&last);
                                let stem = path.file_stem().unwrap();
                                let stem = stem.to_str().unwrap();

                                if !Path::new("./dungeon_statics").exists() {
                                    std::fs::create_dir("./dungeon_statics");
                                }
                                let file_name = format!(
                                    "./dungeon_statics/{}@{}_{}.html",
                                    stem, context.dungeon_save.last_gate, context.dungeon_save.last_clear
                                );

                                #[cfg(debug_assertions)]
                                println!("write to  {}", file_name);
                                let mut file = std::fs::File::create(file_name).unwrap();

                                let table = write_statics(&context.general_statics);
                                file.write_all(table.as_bytes()).unwrap();
                                file.flush().unwrap();
                                context.dungeon_save.entered = false;
                                context.general_statics[StaticsAddress::ItemDungeon.as_uint()].blank();
                                context.general_statics[StaticsAddress::ItemUseDungeon.as_uint()].blank();
                                context.general_statics[StaticsAddress::PartDungeon.as_uint()].blank();
                                context.general_statics[StaticsAddress::KillDungeon.as_uint()].blank();
                            }
                            #[cfg(debug_assertions)]
                            println!(
                                "entered: {} done_line: {:?} last_floor_gate: {}  last_clear: {}",
                                context.dungeon_save.entered,
                                context.dungeon_save.done_line,
                                context.dungeon_save.last_gate,
                                context.dungeon_save.last_clear
                            );
                            let table = write_statics(& context.general_statics);
                            //どこまでのテキストを処理したか記録する
                            context.dungeon_save.done_line.replace(texts.len());
                            table.into_bytes()
                        }

                        "./floor" => {
                            let (last, texts) = search_latest_log_file(chat_dir_path);
                            let texts = read_from_file(last);
                            let from = search_floor(&texts, 0);
                            match from {
                                None => Vec::from(include_str!("not_entered.html")),
                                Some(from) => {
                                    let mut lds = Vec::new();
                                    lds.push(engine_item_get(&texts, from));
                                    lds.push(engine_get_part(&texts, from));
                                    lds.push(engine_item_use(&texts, from));
                                    lds.push(engine_kill_self(&texts, from));

                                    let ctx = InFloorStaticsTemplate {
                                        name: "フロア内カウント".to_string(),
                                        set_of_statics: vec![
                                            GeneralStaticsTemplate {
                                                name: "アイテム取得".to_string(),
                                                statics: {
                                                    let mut vector = hashmap_to_vec(&lds[0]);
                                                    sort(&mut vector, SortTarget::NAME, true);
                                                    vector
                                                },
                                            },
                                            GeneralStaticsTemplate {
                                                name: "パーツ取得".to_string(),
                                                statics: {
                                                    let mut vector = hashmap_to_vec(&lds[1]);
                                                    sort(&mut vector, SortTarget::NAME, true);
                                                    vector
                                                },
                                            },
                                            GeneralStaticsTemplate {
                                                name: "アイテム使用".to_string(),
                                                statics: {
                                                    let mut vector = hashmap_to_vec(&lds[2]);
                                                    sort(&mut vector, SortTarget::NAME, true);
                                                    vector
                                                },
                                            },
                                            GeneralStaticsTemplate {
                                                name: "キル".to_string(),
                                                statics: {
                                                    let mut vector = hashmap_to_vec(&lds[3]);
                                                    sort(&mut vector, SortTarget::NAME, true);
                                                    vector
                                                },
                                            },
                                        ],
                                    };
                                    let table = ctx.render_once().unwrap();
                                    table.into_bytes()
                                }
                            }
                        }

                        _ => Vec::from(include_str!("not_found.html")),
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
                #[cfg(debug_assertions)]
                println!("Setting generated");
                let config_file_content = toml::to_string(&setting).unwrap();
                std::fs::write("./Settings.toml", config_file_content);
                //config.replace(setting);
                let mut old_position = Some(setting);
                std::mem::swap(&mut context.config, &mut old_position);
                #[cfg(debug_assertions)]
                println!("{:#?}", old_position);
                #[cfg(debug_assertions)]
                println!("{:#?}", context.config);
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
