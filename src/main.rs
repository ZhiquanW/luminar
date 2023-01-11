use nvml_wrapper::error::NvmlError;
use serde::de::value;
use serde_json::{self, Value};
use std::collections::HashMap;
use std::ops::Deref;
use std::{fs, path};

use crate::core::{
    LuminarManager, LuminarProcessUtil, LuminarRule, LuminarRuleFilter, LuminarSystemReception,
    LuminarUser, LuminarUserInfo,
};
mod core;

fn load_luminar_configuration(
    cfg_path: &str,
) -> (
    Vec<LuminarUserInfo>,
    Vec<LuminarRuleFilter>,
    Vec<LuminarRule>,
) {
    let config_string = fs::read_to_string(cfg_path).expect("failed to read configuration file");
    let mut luminar_config: HashMap<String, serde_json::Value> =
        serde_json::from_str(&config_string).expect("failed to parse configuration file");
    let luminar_user_info: Vec<LuminarUserInfo> = serde_json::from_value(
        luminar_config
            .remove("user_config")
            .expect("no user_config in luminar_config"),
    )
    .expect("failed to convert Value to LuminarUserInfo");
    let luminar_global_config: Value = serde_json::from_value(
        luminar_config
            .remove("global_config")
            .expect("failed to read global_config from json file"),
    )
    .expect("failed to extract global config");
    let luminar_rule_filters: Vec<LuminarRuleFilter> = serde_json::from_value(
        luminar_global_config
            .as_object()
            .expect("failed to convert value to object")
            .to_owned()
            .remove("rule_filters")
            .expect("failed to get rule_filters from Value"),
    )
    .expect("failed to load rule filters");
    let luminar_common_rule: Vec<LuminarRule> = serde_json::from_value(
        luminar_global_config
            .as_object()
            .expect("failed to convert to object")
            .to_owned()
            .remove("common_rules")
            .expect("failed to get common_rules from luminar_global_config"),
    )
    .expect("faield to read luminar rules from json");
    // .expect("failed to translate json rule_filters to Vec<RuleFilter>");
    println!("{luminar_common_rule:#?}");
    return (luminar_user_info, luminar_rule_filters, luminar_common_rule);
}

fn nvml_test() -> Result<(), NvmlError> {
    use nvml_wrapper::Nvml;
    let nvml = Nvml::init()?;
    // Get the first `Device` (GPU) in the system
    let device = nvml.device_by_index(0)?;
    loop {
        // let samples = device.process_utilization_stats(None).unwrap();
        // for sample in samples.iter() {
        //     println!(
        //         "pid: {:?},{:?}, {:?}",
        //         sample.pid, sample.sm_util, sample.timestamp
        //     );
        // }
        let processes_info = device.running_compute_processes();
        for process in processes_info {
            println!("{process:?}");
        }
    }

    let brand = device.brand()?; // GeForce on my system
    let fan_speed = device.fan_speed(0)?; // Currently 17% on my system
    let power_limit = device.enforced_power_limit()?; // 275k milliwatts on my system
    let encoder_util = device.encoder_utilization()?; // Currently 0 on my system; Not encoding anything
    let memory_info = device.memory_info()?; // Currently 1.63/6.37 GB used on my system
    println!("{memory_info:?}");
    Ok(())
}
fn main() -> Result<(), NvmlError> {
    // nvml_test();
    // initial
    let (luminar_users_info, luminar_rule_filters, luminar_common_rules) =
        load_luminar_configuration("luminar_users_conf.json");
    let mut luminar_manager = LuminarManager::new(
        luminar_users_info,
        luminar_rule_filters,
        luminar_common_rules,
        0.5,
    );
    println!("{:#?}", luminar_manager.user_dict);
    luminar_manager.launch();
    // // search all processes executed by active users
    // for (pid, sys_process) in sys.processes() {
    //     let Some(process_uid) = sys_process.user_id() else {
    //         continue;
    //     };
    //     let Some(process_uname) = uid_name_map.get(process_uid) else {
    //         continue;
    //     };
    //     // if the process is executed by luminar users , record the process and put it into his dashboard
    //     if luminar_user_dict.contains_key(process_uname.as_str()) {
    //         let Some(active_user) = active_luminar_users.get_mut(process_uid) else{
    //             continue;
    //         };
    //         let p_res = ProcessRes {
    //             cpu_usage: sys_process.cpu_usage(),
    //             cpu_memory: sys_process.memory(),
    //             gpu_usage: 0f32,
    //             gpu_memory: 0,
    //         };
    //         active_user.process_map.insert(pid.clone(), p_res);
    //     }
    // }

    // println!("{:?}", active_luminar_users);
    Ok(())
}
