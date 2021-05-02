use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::Path;

use encoding::all::WINDOWS_31J;
use encoding::DecoderTrap;
use encoding::Encoding;

use crate::engines::{
    add_to_table, engine_get_info, engine_get_text, engine_get_text2, engine_get_text3,
    DungeonRewardElement, InnerStatics,
};

pub enum SortTarget {
    NAME,
    QTY,
}

pub enum RewardSort {
    Sell,
    Reward,
}

pub fn sort(vec: &mut Vec<(String, isize)>, target: SortTarget, invert: bool) {
    match target {
        SortTarget::NAME => match invert {
            true => vec.sort_by(|a, b| a.0.cmp(&b.0).reverse()),
            false => vec.sort_by(|a, b| a.0.cmp(&b.0)),
        },
        SortTarget::QTY => match invert {
            true => vec.sort_by(|a, b| a.1.cmp(&b.1).reverse()),
            false => vec.sort_by(|a, b| a.1.cmp(&b.1)),
        },
    }
}

pub fn read_from_file<P: AsRef<Path>>(path: P) -> Vec<String> {
    let content = fs::read(path).unwrap();
    let content = content.as_slice();
    let content = WINDOWS_31J.decode(content, DecoderTrap::Ignore).unwrap();
    engine_get_text(&content)
}

pub fn read_from_file2<P: AsRef<Path>>(path: P) -> Vec<String> {
    let content = fs::read(path).unwrap();
    let content = content.as_slice();
    let content = WINDOWS_31J.decode(content, DecoderTrap::Ignore).unwrap();
    engine_get_info(engine_get_text2(&content))
}

pub fn read_from_file3<P: AsRef<Path>>(path: P) -> Vec<String> {
    let content = fs::read(path).unwrap();
    let content = content.as_slice();
    let content = WINDOWS_31J.decode(content, DecoderTrap::Ignore).unwrap();
    engine_get_text3(&content)
}

pub fn connect_hashmap(map0: InnerStatics, map1: InnerStatics) -> InnerStatics {
    let mut new = map0;
    for (item, qty) in map1.iter() {
        match new.get_mut(item) {
            None => {
                new.insert(item.to_string(), *qty);
            }
            Some(value) => {
                *value += *qty;
            }
        }
    }
    new
}

pub fn hashmap_to_vec(map: &InnerStatics) -> Vec<(String, isize)> {
    let mut vector = Vec::new();
    if !map.is_empty() {
        for (key, val) in map.iter() {
            vector.push((key.to_string(), *val));
        }
    }
    vector
}
pub fn load_tsv<P: AsRef<Path>>(path: P) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let mut file = fs::File::open(path).unwrap();
    let mut string = String::new();
    file.read_to_string(&mut string).unwrap();
    let iter = string.split('\n');
    for line in iter {
        let mut iter = line.split('\t');
        let key = iter.next().unwrap().to_string();
        // let key= key.replace("\\t","\t");
        let value = iter.next().unwrap().to_string();
        #[cfg(dewbug_assertions)]
        println!("key :{} value:{}", key, value);
        map.insert(key, value);
    }
    map
}
