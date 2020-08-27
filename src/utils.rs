pub mod utils {
    use std::collections::HashMap;
    use std::fs;
    use std::path::Path;

    use encoding::all::WINDOWS_31J;
    use encoding::DecoderTrap;
    use encoding::Encoding;

    use crate::engines::engines::{engine_get_info, engine_get_text, engine_get_text2};

    pub enum SortTarget {
        NAME,
        QTY,
    }

    pub fn sort(vec: &mut Vec<(String, usize)>, target: SortTarget, invert: bool) {
        match target {
            SortTarget::NAME => {
                match invert {
                    true => {
                        vec.sort_by(|a, b| {
                            a.0.cmp(&b.0).reverse()
                        })
                    }
                    false => {
                        vec.sort_by(|a, b| {
                            a.0.cmp(&b.0)
                        })
                    }
                }
            }
            SortTarget::QTY => {
                match invert {
                    true => {
                        vec.sort_by(|a, b| {
                            a.1.cmp(&b.1).reverse()
                        })
                    }
                    false => {
                        vec.sort_by(|a, b| {
                            a.1.cmp(&b.1)
                        })
                    }
                }
            }
        }
    }

    pub fn read_from_file<P: AsRef<Path>>(path: P) -> Vec<String> {
        let content = fs::read(path).unwrap();
        let content = content.as_slice();
        let content = WINDOWS_31J.decode(content, DecoderTrap::Ignore).unwrap();
        let texts = engine_get_text(&content);
        texts
    }

    pub fn read_from_file2<P: AsRef<Path>>(path: P) -> Vec<String> {
        let content = fs::read(path).unwrap();
        let content = content.as_slice();
        let content = WINDOWS_31J.decode(content, DecoderTrap::Ignore).unwrap();
        let texts = engine_get_info(engine_get_text2(&content));
        texts
    }

    pub fn connect_hashmap(map0: HashMap<String, usize>, map1: HashMap<String, usize>) -> HashMap<String, usize> {
        let mut new = map0.clone();
        for (item, qty) in map1.iter() {
            match new.get(item) {
                Some(old) => {
                    let qty = old.clone() + *qty;
                    new.insert(item.to_string(), qty);
                }
                None => {
                    new.insert(item.to_string(), *qty);
                }
            }
        }
        new
    }

    pub fn hashmap_to_vec(map: &HashMap<String, usize>) -> Vec<(String, usize)> {
        let mut vector = Vec::new();
        if !map.is_empty() {
            for (key, val) in map.iter() {
                vector.push((key.to_string(), *val));
            }
        }
        vector
    }
}