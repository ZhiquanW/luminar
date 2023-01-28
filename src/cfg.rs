use std::fs;
use std::{collections::HashMap, path::Path};

use serde;
use serde_json::{Map, Value};

use crate::backend::LuminarProcessUtil;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct LuminarGlobalConfig {
    pub port: u32,
    pub refresh_interval: f32,
    pub rule_filters: Vec<LuminarRuleFilter>,
    pub common_rules: Vec<LuminarRule>,
}

impl LuminarGlobalConfig {
    pub fn default() -> LuminarGlobalConfig {
        return LuminarGlobalConfig {
            port: 3114,
            refresh_interval: 0.5f32,
            rule_filters: vec![LuminarRuleFilter {
                name: "tiny_process".to_string(),
                max_cpu_usage: 0.1,
                max_cpu_memory: 512,
                max_gpu_usage: 0.1,
                max_gpu_memory: 512,
            }],
            common_rules: vec![LuminarRule {
                name: "all".to_string(),
                priority: -99,
                max_cpu_core_time: 64,
                max_cpu_memory: 16384,
                max_gpu_device_time: 64,
                max_gpu_memory: 16384,
            }],
        };
    }
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct LuminarRule {
    pub name: String,
    pub priority: i32,
    pub max_cpu_core_time: u64,   // in minutes
    pub max_cpu_memory: u64,      // in MB
    pub max_gpu_device_time: u64, // in minutes
    pub max_gpu_memory: u64,      // in MB
}

impl LuminarRule {
    pub fn match_process(&self, process_util: &LuminarProcessUtil) -> bool {
        process_util.cpu_memory < self.max_cpu_memory
            && process_util.gpu_memory < self.max_gpu_memory
    }
}
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct LuminarRuleFilter {
    pub name: String,
    pub max_cpu_usage: f32,
    pub max_cpu_memory: u64,
    pub max_gpu_usage: f32,
    pub max_gpu_memory: u64,
}

impl LuminarRuleFilter {
    pub fn retain(&self, rule: &LuminarProcessUtil) -> bool {
        // rule.cpu_memory > self.max_cpu_memory
        rule.cpu_usage > self.max_cpu_usage
            || rule.cpu_memory > self.max_cpu_memory
            || rule.gpu_usage > self.max_gpu_usage
            || rule.gpu_memory > self.max_gpu_memory
    }
}
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct LuminarUserInfo {
    pub name: String,
    pub rules: Vec<LuminarRule>,
}

pub fn load_luminar_configuration(
    cfg_path: &Path,
) -> (
    Vec<LuminarUserInfo>,
    Vec<LuminarRuleFilter>,
    Vec<LuminarRule>,
    u32,
    f32,
) {
    let config_string = fs::read_to_string(cfg_path).expect(
        format!(
            "failed to read configuration file from {:?}",
            cfg_path.to_str()
        )
        .as_str(),
    );
    // 1. load all data in json file
    let mut luminar_config: HashMap<String, serde_json::Value> =
        serde_json::from_str(&config_string).expect("failed to parse configuration file");
    // 2. parse all user infos, which is user_config in json
    let Ok(luminar_user_info) =
        serde_json::from_value(luminar_config.remove("user_config").expect("failed to convert Value to LuminarUserInfo"))
             else{
                 luminar_user_info = Vec::new();
            };
    // 3. get global config
    let luminar_global_config: LuminarGlobalConfig = serde_json::from_value(
        luminar_config
            .remove("global_config")
            .expect("failed to read global_config from json file"),
    )
    .expect("failed to extract global config");
    return (
        luminar_user_info,
        luminar_global_config.rule_filters,
        luminar_global_config.common_rules,
        luminar_global_config.port,
        luminar_global_config.refresh_interval,
    );
}
