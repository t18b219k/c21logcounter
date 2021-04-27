#[derive(Clone, Copy)]
pub enum StaticsAddress {
    Item,
    ItemDungeon,
    ItemUse,
    ItemUseDungeon,
    Parts,
    PartDungeon,
    Kill,
    KillDungeon,
    Burst,
    Mission,
    DungeonClear,
    Shuttle,
    Lab,
    Gacha,
}
impl StaticsAddress {
    pub fn from_url(url: &str) -> Option<Self> {
        match url {
            "./items" => Some(Self::Item),
            "./parts" => Some(Self::Parts),
            "./use" => Some(Self::ItemUse),
            "./kills" => Some(Self::Kill),
            "./labo" => Some(Self::Lab),
            "./gacha" => Some(Self::Gacha),
            "./dungeon_clear" => Some(Self::DungeonClear),
            "./burst" => Some(Self::Burst),
            "./mission" => Some(Self::Mission),
            "./shuttle" => Some(Self::Shuttle),
            _ => None,
        }
    }
    pub fn as_uint(&self) -> usize {
        match self {
            StaticsAddress::Item => 0,
            StaticsAddress::ItemDungeon => 1,
            StaticsAddress::ItemUse => 2,
            StaticsAddress::ItemUseDungeon => 3,
            StaticsAddress::Parts => 4,
            StaticsAddress::PartDungeon => 5,
            StaticsAddress::Kill => 6,
            StaticsAddress::KillDungeon => 7,
            StaticsAddress::Burst => 8,
            StaticsAddress::Mission => 9,
            StaticsAddress::DungeonClear => 10,
            StaticsAddress::Shuttle => 11,
            StaticsAddress::Lab => 12,
            StaticsAddress::Gacha => 13,
        }
    }
    pub fn as_dictionary_index(&self) -> Option<usize> {
        //bdms
        match self {
            StaticsAddress::Burst => Some(0),
            StaticsAddress::DungeonClear => Some(1),
            StaticsAddress::Mission => Some(2),
            StaticsAddress::Shuttle => Some(3),
            _ => None,
        }
    }
}
impl ToString for StaticsAddress {
    fn to_string(&self) -> String {
        let text = match self {
            StaticsAddress::Item => "アイテム取得",
            StaticsAddress::ItemDungeon => "アイテム取得",
            StaticsAddress::ItemUse => "アイテム使用",
            StaticsAddress::ItemUseDungeon => "アイテム使用",
            StaticsAddress::Parts => "パーツ",
            StaticsAddress::PartDungeon => "パーツ",
            StaticsAddress::Kill => "キル",
            StaticsAddress::KillDungeon => "キル",
            StaticsAddress::Burst => "突発",
            StaticsAddress::Mission => "ミッション",
            StaticsAddress::DungeonClear => "ダンジョンクリア",
            StaticsAddress::Shuttle => "シャトル",
            StaticsAddress::Lab => "合成",
            StaticsAddress::Gacha => "ガチャ",
        };
        text.to_owned()
    }
}
