use std::collections::HashMap;

use regex::{Captures, Regex};
use std::ops::Add;

pub type InnerStatics = HashMap<String, isize>;

pub fn engine_kill_self(texts: &[String], from: usize) -> InnerStatics {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\t(?P<name>[^が]+?)を撃破した").unwrap();
    }
    let mut table = HashMap::new();
    let last = texts.len();
    for text in &texts[from..last] {
        if let Some(caps) = RE.captures(text) {
            let name = caps.name("name").unwrap().as_str();
            match table.get_mut(name) {
                None => {
                    table.insert(name.to_string(), 1);
                }
                Some(value) => {
                    *value += 1;
                }
            }
        }
    }
    table
}

pub fn engine_gacha(texts: &[String], from: usize) -> InnerStatics {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\[(?P<name>.+)] が当たりました！").unwrap();
    }
    let mut table: HashMap<String, isize> = HashMap::new();
    let last = texts.len();
    for text in &texts[from..last] {
        if let Some(caps) = RE.captures(&text) {
            let name = caps.name("name").unwrap().as_str();
            add_to_table(&mut table, name, 1);
        }
    }
    table
}

pub(crate) fn add_to_table<V: Add + Copy + std::ops::Add<Output = V>>(
    table: &mut HashMap<String, V>,
    key: impl ToString,
    value: V,
) {
    let key = key.to_string();
    match table.get_mut(&key) {
        None => {
            table.insert(key, value);
        }
        Some(old) => {
            *old = *old + value;
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
///(reward,sell)
pub(crate) struct DungeonRewardElement(pub(crate) isize, pub(crate) isize);

impl Add for DungeonRewardElement {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0, self.1 + rhs.1)
    }
}

pub fn engine_item_use(texts: &[String], from: usize) -> InnerStatics {
    //[リペアパック2000] を使用した！
    lazy_static! {
        static ref RE: Regex = Regex::new(r"(?P<name>\[.+]) を使用した！").unwrap();
    }
    let mut table = HashMap::new();
    let last = texts.len();
    for text in &texts[from..last] {
        if let Some(caps) = RE.captures(&text) {
            let name = caps.name("name").unwrap().as_str();
            add_to_table(&mut table, name, 1);
        }
    }
    table
}

//this is not normal format
//so i use dedicated format
// (reward,sells)
pub(crate) fn engine_reward_dungeon(
    texts: &[String],
    from: usize,
) -> HashMap<String, DungeonRewardElement> {
    lazy_static! {
    //	報酬－ ENパック2000 x 1
    static ref RE2:Regex=Regex::new(r"報酬－ (?P<name>.+) x (?P<N>\d+)").unwrap();
    static ref RESELL:Regex=Regex::new(r"報酬売却－ (?P<name>.+) x (?P<N>\d+)").unwrap();
    }
    let mut table: HashMap<String, DungeonRewardElement> = HashMap::new();
    let last = texts.len();
    for text in &texts[from..last] {
        match RE2.captures(&text) {
            Some(caps) => {
                let name = caps.name("name").unwrap().as_str();
                let num = caps.name("N").unwrap().as_str().parse::<isize>().unwrap();
                let cell = DungeonRewardElement(num, 0);
                add_to_table(&mut table, name, cell);
            }
            None => {
                println!("{}", &text)
            }
        }
        match RESELL.captures(&text) {
            Some(caps) => {
                let name = caps.name("name").unwrap().as_str();
                let num = caps.name("N").unwrap().as_str().parse::<isize>().unwrap();
                let cell = DungeonRewardElement(0, num);
                add_to_table(&mut table, name, cell);
            }
            None => {
                println!("{}", &text)
            }
        }
    }
    table
}

pub fn engine_rare(texts: &[String], from: usize) -> InnerStatics {
    lazy_static! {
        static ref RE: Regex =
            Regex::new(r"誰かが \[(?P<name>.+)] をガチャセンターで当てました！").unwrap();
    }
    let mut table = HashMap::new();
    let last = texts.len();
    for text in &texts[from..last] {
        if let Some(caps) = RE.captures(&text) {
            let name = caps.name("name").unwrap().as_str();
            add_to_table(&mut table, name, 1);
        }
    }
    table
}

pub fn engine_labo(texts: &[String], from: usize) -> InnerStatics {
    //1個も合成に成功しないなら
    // 合成に失敗しました
    //1個でも合成に成功したら
    // (?P<name>) × \d+ の作成に成功しました。有機的な破片 × 4
    lazy_static! {
    static ref RE:Regex = Regex::new(r"(?P<name>.+) × (?P<N>[0-9]+)").unwrap();
    //先ずはre0でマッチさせてそして新旧判定とHashTableへの登録を行う
    static ref RE0:Regex = Regex::new(r"(?P<name>.+) の作成に成功しました。").unwrap();
        }
    let mut table = HashMap::new();
    let last = texts.len();
    for text in &texts[from..last] {
        if let Some(caps) = RE0.captures(text) {
            let name = caps.name("name").unwrap().as_str();
            let caps = RE.captures(name);
            let (k, val) = match caps {
                None => {
                    //古いバージョンのログ
                    (name, 1)
                }
                Some(caps) => {
                    //新しいバージョンのログ
                    let name = caps.name("name").unwrap().as_str();
                    let num = caps.name("N").unwrap().as_str().parse::<isize>().unwrap();
                    (name, num)
                }
            };
            println!("{}:{}", k, val);
            add_to_table(&mut table, k, val);
        }
    }
    table
}

pub fn engine_tsv_match(
    texts: &[String],
    dictionary: &HashMap<String, String>,
    from: usize,
) -> InnerStatics {
    let mut table = HashMap::new();
    let last = texts.len();
    for text in &texts[from..last] {
        let mut text = text.clone();
        text = text.replace("\r", "");
        text = text.replace("\t", "\\t");
        text.remove(0);
        text.remove(0);
        let name = dictionary.get(&text);
        if let Some(name) = name {
            add_to_table(&mut table, name, 1);
        }
    }
    table
}

pub fn engine_item_get(texts: &[String], from: usize) -> InnerStatics {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"(?P<name>\[.+]) を (?P<N>\d+)個 取得した！").unwrap();
    }
    let mut table = HashMap::new();
    let last = texts.len();
    for text in &texts[from..last] {
        if let Some(caps) = RE.captures(&text) {
            let name = caps.name("name").unwrap().as_str();
            let num = caps.name("N").unwrap().as_str().parse::<isize>().unwrap();
            add_to_table(&mut table, name, num);
        }
    }
    table
}

pub fn engine_get_part(texts: &[String], from: usize) -> InnerStatics {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"(?P<name>\[.+]) を取得した！").unwrap();
    }
    let mut table = HashMap::new();
    let last = texts.len();
    for text in &texts[from..last] {
        if let Some(caps) = RE.captures(&text) {
            let name = caps.name("name").unwrap().as_str();
            add_to_table(&mut table, name, 1);
        }
    }
    table
}

///フロアゲートの起動を探す.(last)
pub fn search_floor(texts: &[String], search_from: usize) -> Option<usize> {
    let last = texts.len();
    let mut floor = None;
    if search_from > last {
        return None;
    }
    for (offset, text) in texts[search_from..last].iter().enumerate() {
        println!("offset {} ", offset);
        if text.contains("がフロアゲートを起動した！") {
            floor = Some(search_from + offset);
        }
    }
    floor
}

///フロアゲートの起動を探す.(first)
pub fn search_floor_first(texts: &[String], search_from: usize) -> Option<usize> {
    let last = texts.len();
    lazy_static! {
        static ref RE: Regex = Regex::new(r"(?P<name>.+?)がフロアゲートを起動した！").unwrap();
    }
    for (offset, text) in texts[search_from..last].iter().enumerate() {
        if RE.captures(text).is_some() {
            return Some(offset + search_from);
        }
    }
    None
}

///ダンジョンクリア(last)
pub fn search_dungeon_clear(texts: &[String], search_from: usize) -> Option<usize> {
    //ダンジョン成功報酬
    let last = texts.len();
    for (offset, text) in texts[search_from..last].iter().enumerate() {
        if text.contains(r"ダンジョン成功報酬") {
            return Some(search_from + offset);
        }
    }
    None
}

pub fn engine_get_text(text: &str) -> Vec<String> {
    let mut texts = vec![];
    lazy_static! {
        static ref RE: Regex =
            Regex::new(r"\d{4}-\d{2}-\d{2}	\d{2}:\d{2}:\d{2}	\[INFO]	(?P<text>.+)").unwrap();
    }
    for caps in RE.captures_iter(text) {
        texts.push("\t".to_string() + caps.name("text").unwrap().as_str());
    }
    texts
}

pub fn engine_get_text3(text: &str) -> Vec<String> {
    let mut texts = vec![];
    lazy_static! {
        static ref RE: Regex =
            Regex::new(r"\d{4}-\d{2}-\d{2}	\d{2}:\d{2}:\d{2}	(?P<text>.+)").unwrap();
    }
    for caps in RE.captures_iter(text) {
        texts.push("\t".to_string() + caps.name("text").unwrap().as_str());
    }
    texts
}

pub fn engine_get_text2(text: &str) -> Vec<String> {
    let mut texts = Vec::new();
    let longtext;
    //\r\nを削除する代わりに<ls>をログの区切りとする.
    lazy_static! {
        static ref RE: Regex = Regex::new(r"(?P<time>\d{4}-\d{2}-\d{2}	\d{2}:\d{2}:\d{2})").unwrap();
    }
    //<ls>\date\time text
    longtext = RE
        .replace_all(text, |caps: &Captures| {
            format!("<ls>{}", caps.name("time").unwrap().as_str())
        })
        .replace("\n", "")
        .replace("\r", "");
    for text in longtext.split("<ls>") {
        texts.push(text.to_string());
    }

    texts
}

pub fn engine_get_info(texts: Vec<String>) -> Vec<String> {
    let mut vec = Vec::new();
    lazy_static! {
        static ref RE: Regex =
            Regex::new(r"(?P<time>\d{4}-\d{2}-\d{2}	\d{2}:\d{2}:\d{2})\t\[INFO]\t(?P<text>.+)")
                .unwrap();
    }
    for text in texts {
        match RE.captures(text.as_str()) {
            None => {}
            Some(caps) => vec.push(caps.name("text").unwrap().as_str().to_string()),
        }
    }
    vec
}

pub fn get_time(text: &str) -> String {
    lazy_static! {
        static ref RE: Regex =
            Regex::new(r"(?P<time>\d{4}-\d{2}-\d{2}	\d{2}:\d{2}:\d{2})\t\[INFO]\t(?P<text>.+)")
                .unwrap();
    }
    let captures = RE.captures(text);
    match captures {
        None => "No time stamp".to_string(),
        Some(caps) => caps.name("time").unwrap().as_str().to_string(),
    }
}
