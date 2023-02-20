use serde;

use crate::backend::LuminarProcessUtil;
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
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
            refresh_interval: 2.0f32,
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

/// unstable
///
/// todo: add cpu/gpu memory time
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
    #[allow(dead_code)]
    pub fn match_process(&self, process_util: &LuminarProcessUtil) -> bool {
        process_util.cpu_memory < self.max_cpu_memory
            && process_util.gpu_memory < self.max_gpu_memory
    }
}
/// define the process should be filtered so it will not be managed by luminar
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
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
