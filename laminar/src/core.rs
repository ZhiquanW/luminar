use std::collections::HashMap;
use sysinfo::{Uid, Pid};
use serde::{Deserialize, Serialize};
use serde;
#[derive(Debug)]
pub struct ProcessRes {
    pub cpu_usage: f32,
    pub cpu_memory: u64,
    pub gpu_usage: f32,
    pub gpu_memory: u64,
}
// public to external files
#[derive(Debug)]
pub struct LaminarRule{
    pub name: String,
    pub time: f64,
    pub max_gpu_memory: u64,
}
#[derive(Debug)]
pub struct LaminarUser {
    pub uid: Uid,
    pub name: String,
    pub rules: Vec<LaminarRule>,
    pub process_map: HashMap<Pid, ProcessRes>,
}
// #[derive(Deserialize)]
// pub struct LaminarUserMap {
//     #[serde(flatten)]
//     pub laminar_users: HashMap<Uid, LaminarUser>,
// }

// impl Into<HashMap<String,LaminarUser>> for LaminarUserMap {
//     fn into(self) -> HashMap<String,LaminarUser> {
//         let mut user_map: HashMap<String,LaminarUser> = HashMap::new();
//         for (uid, user) in self.laminar_users.iter() {
//             user_map.insert(user.name.clone(), user.clone());
//         }
//         return user_map;
//     }
// }