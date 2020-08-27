pub mod engines {
    use std::collections::HashMap;

    use regex::{Captures, Regex};

    pub fn engine_kill_self(texts: &[String], from: usize) -> HashMap<String, usize> {
        lazy_static! {
        static ref re: Regex=Regex::new(r"\t(?P<name>[^が]+?)を撃破した").unwrap();
        }
        let mut table = HashMap::new();
        let last = texts.len();
        for i in from..last {
            let text = &texts[i];
            match re.captures(text) {
                Some(caps) => {
                    let name = caps.name("name").unwrap().as_str();

                    match table.get(name) {
                        Some(v) => {
                            table.insert(name.to_string(), v + 1);
                        }
                        None => {
                            table.insert(name.to_string(), 1);
                        }
                    };
                }
                None => {}
            }
        }
        table
    }

    pub fn engine_gacha(texts: &[String], from: usize) -> HashMap<String, usize> {
        lazy_static! {
        static ref re:Regex = Regex::new(r"\[(?P<name>.+)] が当たりました！").unwrap();
        }
        let mut table = HashMap::new();
        let last = texts.len();
        for i in from..last {
            let text = &texts[i];
            match re.captures(&text) {
                Some(caps) => {
                    let name = caps.name("name").unwrap().as_str();
                    match table.get(name) {
                        Some(v) => {
                            table.insert(name.to_string(), v + 1);
                        }
                        None => {
                            table.insert(name.to_string(), 1);
                        }
                    };
                }
                None => {}
            }
        }
        table
    }

    pub fn engine_item_use(texts: &[String], from: usize) -> HashMap<String, usize> {
        //[リペアパック2000] を使用した！　
        lazy_static! {
        static ref re:Regex = Regex::new(r"(?P<name>\[.+]) を使用した！").unwrap();
        }
        let mut table = HashMap::new();
        let last = texts.len();
        for i in from..last {
            let text = &texts[i];
            match re.captures(&text) {
                Some(caps) => {
                    let name = caps.name("name").unwrap().as_str();
                    match table.get(name) {
                        Some(v) => {
                            table.insert(name.to_string(), v + 1);
                        }
                        None => {
                            table.insert(name.to_string(), 1);
                        }
                    };
                }
                None => {}
            }
        }
        table
    }

    pub fn engine_rare(texts: &[String], from: usize) -> HashMap<String, usize> {
        lazy_static! {
        static ref re:Regex = Regex::new(r"誰かが \[(?P<name>.+)] をガチャセンターで当てました！").unwrap();
       }
        let mut table = HashMap::new();
        let last = texts.len();
        for i in from..last {
            let text = &texts[i];
            match re.captures(&text) {
                Some(caps) => {
                    let name = caps.name("name").unwrap().as_str();
                    match table.get(name) {
                        Some(v) => {
                            table.insert(name.to_string(), v + 1);
                        }
                        None => {
                            table.insert(name.to_string(), 1);
                        }
                    };
                }
                None => {}
            }
        }
        table
    }

    pub fn engine_labo(texts: &[String], from: usize) -> HashMap<String, usize> {
        //1個も合成に成功しないなら
        // 合成に失敗しました
        //1個でも合成に成功したら
        // (?P<name>) × \d+ の作成に成功しました。有機的な破片 × 4
        lazy_static! {
        static ref re:Regex = Regex::new(r"(?P<name>.+) × (?P<N>.+)").unwrap();
        //先ずはre0でマッチさせてそして新旧判定とHashTableへの登録を行う
        static ref re0:Regex = Regex::new(r"(?P<name>.+) の作成に成功しました。").unwrap();
            }
        let mut table = HashMap::new();
        let last = texts.len();
        for i in from..last {
            let text = &texts[i];
            match re0.captures(text) {
                Some(caps) => {
                    let name = caps.name("name").unwrap().as_str();
                    let caps = re.captures(name);
                    let (k, val) = match caps {
                        None => {//古いバージョンのログ
                            (name, 1)
                        }
                        Some(caps) => {//新しいバージョンのログ
                            let name = caps.name("name").unwrap().as_str();
                            let num = caps.name("N").unwrap().as_str().parse::<usize>().unwrap();
                            (name, num)
                        }
                    };
                    println!("{}:{}", k, val);
                    match table.get(k) {
                        Some(v) => {
                            //tableのエントリーがある場合
                            table.insert(k.to_string(), v + val);
                        }
                        None => {
                            table.insert(k.to_string(), val);
                        }
                    };
                }
                None => {}
            }
        }

        table
    }

    pub fn engine_item_get(texts: &[String], from: usize) -> HashMap<String, usize> {
        lazy_static! {
        static ref re:Regex = Regex::new(r"(?P<name>\[.+]) を (?P<N>\d+)個 取得した！").unwrap();
       }
        let mut table = HashMap::new();
        let last = texts.len();
        for i in from..last {
            let text = &texts[i];
            match re.captures(&text) {
                Some(caps) => {
                    let name = caps.name("name").unwrap().as_str();
                    let num = caps.name("N").unwrap().as_str().parse::<usize>().unwrap();
                    match table.get(name) {
                        Some(v) => {
                            table.insert(name.to_string(), v + num);
                        }
                        None => {
                            table.insert(name.to_string(), num);
                        }
                    };
                }
                None => {}
            }
        }
        table
    }

    pub fn engine_get_part(texts: &[String], from: usize) -> HashMap<String, usize> {
        lazy_static! {
        static ref re:Regex = Regex::new(r"(?P<name>\[.+]) を取得した！").unwrap();
        }
        let mut table = HashMap::new();
        let last = texts.len();
        for i in from..last {
            let text = &texts[i];
            match re.captures(&text) {
                Some(caps) => {
                    let name = caps.name("name").unwrap().as_str();
                    match table.get(name) {
                        Some(v) => {
                            table.insert(name.to_string(), v + 1);
                        }
                        None => {
                            table.insert(name.to_string(), 1);
                        }
                    };
                }
                None => {}
            }
        }
        table
    }

    //フロアゲートの起動を探す.(last)
    pub fn search_floor(texts: &[String], search_from: usize) -> Option<usize> {
        let last = texts.len();
        let mut floor = 0;
        lazy_static! {
        static ref re:Regex = Regex::new(r"(?P<name>.+?)がフロアゲートを起動した！").unwrap();
        }
        for i in search_from..last {
            let text = &texts[i];
            match re.captures(text) {
                None => {}
                Some(_) => {
                    floor = i;
                }
            }
        }
        if floor == 0 {
            None
        } else {
            Some(floor)
        }
    }

    //フロアゲートの起動を探す.(first)
    pub fn search_floor_first(texts: &[String], search_from: usize) -> Option<usize> {
        let last = texts.len();
        let mut floor = 0;
        lazy_static! {
        static ref re:Regex = Regex::new(r"(?P<name>.+?)がフロアゲートを起動した！").unwrap();
        }
        for i in search_from..last {
            let text = &texts[i];
            match re.captures(text) {
                None => {}
                Some(_) => {
                    return Some(floor);
                }
            }
        }
        return None;
    }

    //ダンジョンクリア(last)
    pub fn search_dungeon_clear(texts: &[String], search_from: usize) -> Option<usize> {
        //ダンジョン成功報酬
        let last = texts.len();
        lazy_static! {
        static ref re:Regex = Regex::new(r"ダンジョン成功報酬").unwrap();
        }
        for i in search_from..last {
            let text = &texts[i];
            match re.captures(text) {
                None => {}
                Some(_) => {
                    return Some(i);
                }
            }
        }
        None
    }

    pub fn engine_get_text(text: &str) -> Vec<String> {
        let mut texts = vec![];
        lazy_static! {
        static ref re:Regex = Regex::new(r"\d{4}-\d{2}-\d{2}	\d{2}:\d{2}:\d{2}	\[INFO]	(?P<text>.+)").unwrap();
        }
        for caps in re.captures_iter(text) {
            texts.push("\t".to_string() + caps.name("text").unwrap().as_str());
        }
        texts
    }

    pub fn engine_get_text2(text: &str) -> Vec<String> {
        let mut texts = Vec::new();
        let mut longtext;
        //\r\nを削除する代わりに<ls>をログの区切りとする.
        lazy_static! {
        static ref re:Regex = Regex::new(r"(?P<time>\d{4}-\d{2}-\d{2}	\d{2}:\d{2}:\d{2})").unwrap();
       }
        //<ls>\date\time text
        longtext = re.replace_all(text, |caps: &Captures| {
            format!("<ls>{}", caps.name("time").unwrap().as_str())
        }).replace("\n", "").replace("\r", "");
        for text in longtext.split("<ls>") {
            texts.push(text.to_string());
        }

        texts
    }

    pub fn engine_get_info(texts: Vec<String>) -> Vec<String> {
        let mut vec = Vec::new();
        lazy_static! {
        static ref re:Regex = Regex::new(r"(?P<time>\d{4}-\d{2}-\d{2}	\d{2}:\d{2}:\d{2})\t\[INFO]\t(?P<text>.+)").unwrap();
        }
        for text in texts {
            match re.captures(text.as_str()) {
                None => {}
                Some(caps) => {
                    vec.push(caps.name("text").unwrap().as_str().to_string())
                }
            }
        }
        vec
    }

    pub fn get_time(text: &str) -> String {
        lazy_static! {
        static ref re:Regex = Regex::new(r"(?P<time>\d{4}-\d{2}-\d{2}	\d{2}:\d{2}:\d{2})\t\[INFO]\t(?P<text>.+)").unwrap();
        }
        let captures = re.captures(text);
        match captures {
            None => {
                "No time stamp".to_string()
            }
            Some(caps) => {
                caps.name("time").unwrap().as_str().to_string()
            }
        }
    }
}