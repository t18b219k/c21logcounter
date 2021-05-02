/*
state_machine
    out_of_dungeon ←-------------
        ↓ floor_gate            |
     dungeon                    |
       | |  ↑floor_gate         |
       | ----                   |
       ↓ dungeon_clear_reward   |
      clear     time_out        |
        ↓　|--------------------|
      reward                    |
        | | ↑ reward_item       |
        | ---                   |
|-------|                       |
↓sell   |time_out|floor_gate    |
sell    |---------------------- |
| |  ↑sell_item                 |
| ----                          |
| summary_of_sell               |
---------------------------------
*/
use crate::engines::{
    engine_get_part, engine_item_get, engine_item_use, engine_kill_self, engine_reward_dungeon,
    search_dungeon_clear_first, search_floor_first, search_reward_first, search_reward_sell_first,
    InnerStatics,
};
use chrono::NaiveDateTime;
use std::borrow::Borrow;
use std::ops::Range;
use std::option::Option::Some;
#[derive(Debug)]
pub struct DungeonStateMachine {
    state: DungeonState,
    texts: Vec<String>,
    time_stamps: Vec<NaiveDateTime>,
    from: usize,
    current_line: usize,
    clear_time: Option<chrono::NaiveDateTime>,
    start_time: Option<chrono::NaiveDateTime>,
    dungeon_range: Option<Range<usize>>,
    emit: bool,
}
#[derive(Debug)]
pub struct DungeonOutPut {
    pub rewards: InnerStatics,
    pub sells: InnerStatics,
    pub statics: Vec<InnerStatics>,
    reward_exp: usize,
    reward_guild_pint: usize,
    reward_dollar: usize,
}
impl DungeonStateMachine {
    pub fn init(texts: Vec<String>, time_stamps: Vec<NaiveDateTime>, from: usize) -> Self {
        Self {
            state: DungeonState::OutOfDungeon,
            texts,
            time_stamps,
            from,
            current_line: from,
            clear_time: None,
            start_time: None,
            dungeon_range: None,
            emit: false,
        }
    }
    pub fn statics(&mut self) -> Option<DungeonOutPut> {
        let range = if let Some(dungeon_range) = self.dungeon_range.clone() {
            dungeon_range
        } else {
            self.from..self.current_line
        };

        let texts = &self.texts[range];
        let items = engine_item_get(texts, 0);
        let item_use = engine_item_use(texts, 0);
        let parts = engine_get_part(texts, 0);
        let kill = engine_kill_self(texts, 0);
        let rewards = engine_reward_dungeon(texts, 0);

        Some(DungeonOutPut {
            rewards: rewards.0,
            sells: rewards.1,
            statics: vec![items, item_use, parts, kill],
            reward_exp: 0,
            reward_guild_pint: 0,
            reward_dollar: 0,
        })
    }
    pub fn state_change(&mut self) {
        match self.state {
            DungeonState::OutOfDungeon => {
                let floor_gate = search_floor_first(&self.texts, self.current_line);
                if let Some(floor_gate) = floor_gate {
                    self.from = floor_gate;
                    self.state = DungeonState::Dungeon;
                    self.current_line = floor_gate;
                    self.start_time.replace(self.time_stamps[self.current_line]);
                }
            }
            DungeonState::Dungeon => {
                let clear = search_dungeon_clear_first(&self.texts, self.current_line);
                if let Some(clear) = clear {
                    self.state = DungeonState::Clear;
                    self.current_line = clear;
                    self.clear_time.replace(self.time_stamps[self.current_line]);
                } else {
                    self.current_line = self.texts.len();
                }
            }
            DungeonState::Clear => {
                let reward_start = search_reward_first(&self.texts, self.current_line);
                let current_time = self.time_stamps[self.current_line];
                if let Some(reward_start) = reward_start {
                    self.state = DungeonState::Reward;
                    self.current_line = reward_start;
                } else if (current_time - self.clear_time.unwrap()) > chrono::Duration::seconds(120)
                {
                    self.state = DungeonState::OutOfDungeon;
                    self.emit = true;
                }
            }
            DungeonState::Reward => {
                while self.state == DungeonState::Reward && (self.current_line < self.texts.len()) {
                    let current_time = self.time_stamps[self.current_line];
                    let activate_floor_gate = search_floor_first(&self.texts, self.current_line);
                    let sell_start = search_reward_sell_first(&self.texts, self.current_line);
                    //check time out
                    if (current_time - self.clear_time.unwrap()) > chrono::Duration::seconds(120) {
                        self.state = DungeonState::OutOfDungeon;
                        self.emit = true;
                    } else if let Some(floor_gate) = activate_floor_gate {
                        self.state = DungeonState::Dungeon;
                        self.start_time.replace(self.time_stamps[self.current_line]);
                        self.current_line = floor_gate;
                        self.dungeon_range.replace(self.from..self.current_line);
                        self.from = self.current_line;
                        self.emit = true;
                    } else if let Some(sell) = sell_start {
                        self.state = DungeonState::Sell;
                        self.current_line = sell;
                    } else {
                        self.current_line += 1;
                    }
                }
                //ログが足りなくなるとここに来る
            }
            DungeonState::Sell => {
                while self.state == DungeonState::Sell && (self.current_line < self.texts.len()) {
                    if self.texts[self.current_line].contains("報酬売却計") {
                        self.state = DungeonState::OutOfDungeon;
                        //save
                        self.dungeon_range.replace(self.from..self.current_line);
                        self.from = self.current_line;
                        self.emit = true;
                    } else {
                        self.current_line += 1;
                    }
                }
            }
        }
    }
    pub fn inspect_state(&self) -> &DungeonState {
        self.state.borrow()
    }
    pub fn supply_text(&mut self, other: (&[NaiveDateTime], &[String])) {
        self.texts.extend_from_slice(other.1);
        self.time_stamps.extend_from_slice(other.0);
    }
    pub fn get_current_text_len(&self) -> usize {
        self.texts.len()
    }
    pub fn query_dungeon_range(&mut self) -> Option<Range<usize>> {
        self.dungeon_range.take()
    }
}
///ダンジョンの状態
#[derive(Debug, Eq, PartialEq)]
pub enum DungeonState {
    OutOfDungeon,
    Dungeon,
    Clear,
    Reward,
    Sell,
}
