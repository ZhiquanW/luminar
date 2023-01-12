use crate::core::LuminarProcessUtil;
use serde;

#[derive(Debug, Clone, Default, serde::Deserialize)]
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
#[derive(Debug, serde::Deserialize)]
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
#[derive(Debug, Clone, serde::Deserialize)]
pub struct LuminarUserInfo {
    pub name: String,
    pub rules: Vec<LuminarRule>,
}
