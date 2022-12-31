use std::fs;
use std::str::FromStr;
use nvml_wrapper::enum_wrappers::device::{Clock, TemperatureSensor};
use nvml_wrapper::error::NvmlError;
use nvml_wrapper::{cuda_driver_version_major, cuda_driver_version_minor, Nvml};
use pretty_bytes::converter::convert;
use std::collections::HashMap;
use sysinfo::{Pid, Process, ProcessExt, System, SystemExt, Uid, UserExt};
use serde_json;

use crate::core::{ProcessRes, LaminarUser, LaminarRule};
mod core;

fn main() -> Result<(), NvmlError> {

    // initial 
    let a = ["aa","bb"];
    // read configuration file &create laminar users based on json file
    let config_string = fs::read_to_string("laminar_users_conf.json").expect("failed to read configuration file");
    let binding = serde_json::from_str::<serde_json::Value>(&config_string).expect("failed to parse configuration file");
    let config_json = binding.as_object().expect("failed to parse configuration file");
    let mut laminar_user_dict: HashMap<String, core::LaminarUser> = HashMap::new();
    for (key,value) in config_json.into_iter(){
        let binding= value.get("rules").expect("illegal key name");
        let rules_data = binding.as_object().expect("failed to unwrap from rules data binding");
        let mut rules = Vec::new();
        for (rule_key,rule_value) in rules_data.into_iter(){
            rules.push(core::LaminarRule{
                name:String::from(rule_key.clone()), 
                time:rule_value.get("core_time").expect("failed to get time from rule").as_f64().expect("failed to transfer value to f64"),
                max_gpu_memory: rule_value.get("max_gpu_memory").expect("failed to get max gpu memory").as_u64().expect("failed to transfer value to u64"),
            });
        }
        laminar_user_dict.insert(key.clone(), core::LaminarUser{
            name: key.clone(),
            uid: Uid::from_str("0").unwrap(),
            rules: rules,
            process_map: HashMap::new(),
        });
    }
    println!("{laminar_user_dict:?}");
    // println!("config_string: {}", config_string);
    // perform each time step
    // create ExcUsers
    let sys = System::new_all();
    println!("System Total Memory: {} bytes", sys.total_memory());
    // create a map from uid to name
    let mut uid_name_map: HashMap<Uid, String> = HashMap::new();
    for user in sys.users() {
        uid_name_map.insert(user.id().clone(), user.name().to_string());
    }
    // create excusers
    let mut active_laminar_users: HashMap<Uid, core::LaminarUser> = HashMap::new();
    for (uid, uname) in uid_name_map.iter() {
        // vec contains.(&&str)
        // dict contains_key(&str)
        let b = a.contains(&uname.as_str());
        if laminar_user_dict.contains_key(uname.as_str()) {
            let active_user = core::LaminarUser {
                name: uname.clone(),
                uid: uid.clone(),
                rules: Vec::new(),
                process_map: HashMap::new(),
            };
            active_laminar_users.insert(uid.clone(), active_user);
        }
    }

    // search all processes executed by active users
    for (pid, sys_process) in sys.processes() {
        let Some(process_uid) = sys_process.user_id() else {
            continue;
        };
        let Some(process_uname) = uid_name_map.get(process_uid) else {
            continue;
        };
        // if the process is executed by laminar users , record the process and put it into his dashboard
        if laminar_user_dict.contains_key(process_uname.as_str()) {
            let Some(active_user) = active_laminar_users.get_mut(process_uid) else{
                continue;
            };
            let p_res = ProcessRes {
                cpu_usage:sys_process.cpu_usage(),
                cpu_memory: sys_process.memory(),
                gpu_usage: 0f32,
                gpu_memory: 0,
            };
            active_user.process_map.insert(pid.clone(), p_res);
        }
    }

    // println!("{:?}", active_laminar_users);
    Ok(())
}
