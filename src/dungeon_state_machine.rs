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
use chrono::{Local, NaiveDateTime};
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
    sync: bool,
}
#[derive(Debug)]
pub struct DungeonOutPut {
    pub rewards: InnerStatics,
    pub sells: InnerStatics,
    pub statics: Vec<InnerStatics>,
    pub lap_time: Option<chrono::Duration>,
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
            sync: false,
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

        let lap_time = match self.state {
            DungeonState::OutOfDungeon => None,
            DungeonState::Dungeon => {
                let time = self.get_current_time();

                Some(time - self.start_time.unwrap())
            }
            _ => Some(self.clear_time.unwrap() - self.start_time.unwrap()),
        };
        Some(DungeonOutPut {
            rewards: rewards.0,
            sells: rewards.1,
            statics: vec![items, item_use, parts, kill],
            lap_time,
            reward_exp: 0,
            reward_guild_pint: 0,
            reward_dollar: 0,
        })
    }
    fn get_current_time(&self) -> NaiveDateTime {
        if self.sync {
            chrono::Local::now().naive_local()
        } else {
            self.time_stamps[self.current_line]
        }
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

                    let current_time = self.time_stamps[self.current_line];
                    let world_current_time = chrono::Local::now().naive_local();
                    if (world_current_time - current_time) < chrono::Duration::seconds(6) {
                        #[cfg(debug_assertions)]
                        println!("set sync");
                        self.sync = true;
                    }
                }
            }
            DungeonState::Dungeon => {
                let clear = search_dungeon_clear_first(&self.texts, self.current_line);
                if let Some(clear) = clear {
                    self.state = DungeonState::Clear;
                    self.current_line = clear;
                    let current_time = self.time_stamps[self.current_line];
                    self.clear_time.replace(current_time);
                    #[cfg(debug_assertions)]
                    println!("clear time {:?}", self.clear_time)
                } else {
                    self.current_line = self.texts.len();
                }
            }
            DungeonState::Clear => {
                //同期しているならcurrent_time をシステムの時間にセットする
                let current_time = self.get_current_time();
                if (current_time - self.clear_time.unwrap()) > chrono::Duration::seconds(120) {
                    self.state = DungeonState::OutOfDungeon;
                    self.dungeon_range.replace(self.from..self.current_line);
                    self.current_line = self.texts.len();
                    self.from = self.texts.len();
                    return;
                }

                let reward_start = search_reward_first(&self.texts, self.current_line);
                while self.state == DungeonState::Clear && (self.current_line < self.texts.len()) {
                    if let Some(reward_start) = reward_start {
                        self.state = DungeonState::Reward;
                        self.current_line = reward_start;
                    } else {
                        self.current_line += 1;
                    }
                }
            }
            DungeonState::Reward => {
                // check time out
                let current_time = self.get_current_time();
                if (current_time - self.clear_time.unwrap()) > chrono::Duration::seconds(120) {
                    self.state = DungeonState::OutOfDungeon;
                    self.dungeon_range.replace(self.from..self.current_line);
                    self.current_line = self.texts.len();
                    self.from = self.texts.len();
                    return;
                }
                while self.state == DungeonState::Reward && (self.current_line < self.texts.len()) {
                    //同期しているならcurrent_time をシステムの時間にセットする

                    let activate_floor_gate = search_floor_first(&self.texts, self.current_line);
                    let sell_start = search_reward_sell_first(&self.texts, self.current_line);
                    #[cfg(debug_assertions)]
                    println!("elapsed time {}", current_time - self.clear_time.unwrap());
                    if let Some(floor_gate) = activate_floor_gate {
                        self.state = DungeonState::Dungeon;
                        self.start_time.replace(self.time_stamps[self.current_line]);
                        self.current_line = floor_gate;
                        self.dungeon_range.replace(self.from..self.current_line);
                        self.from = self.current_line;
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
