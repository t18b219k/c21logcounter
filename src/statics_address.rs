#[derive(Clone, Copy)]
pub enum StaticsAddress {
    Item,
    ItemUse,
    Parts,
    Kill,
    Burst,
    Mission,
    Shuttle,
    Lab,
    Gacha,
    DungeonReward,
    DungeonSell,
    DungeonClear,
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
            StaticsAddress::ItemUse => 1,
            StaticsAddress::Parts => 2,
            StaticsAddress::Kill => 3,
            StaticsAddress::Burst => 4,
            StaticsAddress::Mission => 5,
            StaticsAddress::DungeonClear => 6,
            StaticsAddress::Shuttle => 7,
            StaticsAddress::Lab => 8,
            StaticsAddress::Gacha => 9,
            StaticsAddress::DungeonReward => 10,
            StaticsAddress::DungeonSell => 11,
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
            StaticsAddress::ItemUse => "アイテム使用",
            StaticsAddress::Parts => "パーツ",
            StaticsAddress::Kill => "キル",
            StaticsAddress::Burst => "突発",
            StaticsAddress::Mission => "ミッション",
            StaticsAddress::DungeonClear => "ダンジョンクリア",
            StaticsAddress::Shuttle => "シャトル",
            StaticsAddress::Lab => "合成",
            StaticsAddress::Gacha => "ガチャ",
            StaticsAddress::DungeonReward => "ダンジョン報酬",
            StaticsAddress::DungeonSell => "ダンジョン報酬売却",
        };
        text.to_owned()
    }
}
