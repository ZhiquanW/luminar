use serde;
use std::collections::HashMap;
use std::ops::Deref;
use std::thread::sleep;
use sysinfo::{PidExt, ProcessExt, SystemExt, UserExt};

use crate::cfg::{LuminarRuleFilter, LuminarUserInfo, LuminarRule};
#[derive(Debug, Clone, serde::Deserialize)]
pub struct LuminarProcessUtil {
    pub pid: u32,
    pub commond: Vec<String>,
    pub cpu_usage: f32,
    pub cpu_memory: u64,
    pub device_id: u32,
    pub gpu_usage: f32,
    pub gpu_memory: u64,
}

#[derive(Debug,)]
pub struct LuminarUser {
    pub uid: u32,
    pub name:String,
    pub process_trackers_map: HashMap<u32, LuminarProcessTacker>, // a map from pid to process
    pub rule_trackers_map:HashMap<String, LuminarRuleTracker>,
}

#[derive(Debug)]
pub struct LuminarResManager {
    pub refresh_interval: f32, // in seconds
    sys_reception: LuminarSystemReception,
    pub user_dict: HashMap<u32, LuminarUser>,
    rule_filters: Vec<LuminarRuleFilter>,
}

impl LuminarResManager {
    pub fn new(mut users_info: Vec<LuminarUserInfo>,rule_filters: Vec<LuminarRuleFilter>, common_rules : Vec<LuminarRule>, refresh_interval:f32) -> LuminarResManager {
        let mut users = vec![];
        let sys_reception = LuminarSystemReception::new();
        for user_info in users_info.iter_mut() {
            let Some(uid) = sys_reception.get_uid_by_uname(&user_info.name) else{
                continue;
            };
            // append all common rules to user rules
            user_info.rules.append(&mut common_rules.clone());
            users.push(LuminarUser {
                uid: uid.clone(),
                name: user_info.name.clone(),
                process_trackers_map: HashMap::new(),
                rule_trackers_map: user_info.rules.iter().map(|r|(r.name.clone(), LuminarRuleTracker{
                    name: r.name.clone(),
                    rule:r.clone(),
                    cpu_core_time: 0.0,
                    gpu_device_time:0.0,
                    tracked_pids: Vec::new()
                })).collect(),
            });
        }
        LuminarResManager {
            refresh_interval: refresh_interval,
            sys_reception: sys_reception,
            user_dict: users
                .into_iter()
                .map(|user| (user.uid.clone(), user))
                .collect(),
            rule_filters: rule_filters,
        }
    }

    pub fn launch(&mut self) {
        loop {
            let uid_process_info_map  = self.sys_reception.get_uid_process_info_map();
            let uid_process_info_map = self.global_filter_rules(uid_process_info_map);
            // iterate through eahc luminar user and process their process and rules
            for (uid, luminar_user) in self.user_dict.iter_mut(){
                let Some(luminar_processes_util) = uid_process_info_map.get(uid) else{
                    continue;
                };
                // clear all tracked pids for the next step to push
                for (_, rule_tracker) in luminar_user.rule_trackers_map.iter_mut(){
                    rule_tracker.tracked_pids.clear();
                }
                // iterate through each process, track the running status and address it by corresponding rule
                for luminar_process_util in luminar_processes_util.iter(){
                    let pid = luminar_process_util.pid;
                  
                    // find the corresponding rule of the process, update the rule consumption map
                    // iterate through user rules, leave the matched rules
                    let mut matched_rules:Vec<LuminarRule> = luminar_user.rule_trackers_map.iter().map(
                        |(_,rt)| rt.rule.clone()
                    ).filter_map(|r|  r.match_process(luminar_process_util).then_some(r.clone())).collect();
                    matched_rules.sort_by(|a,b| b.priority.cmp(&a.priority));
                    // if no rules is matched, this process is illegal, kill
                    if matched_rules.is_empty(){
                        if self.sys_reception.kill(&pid){
                            println!("process {:?}, out of any rules. Killed",luminar_process_util.pid);
                        }else{
                            println!("failed to kill process {:?}", luminar_process_util.pid);
                        }
                    }else{
                        // if a rule is matched, track the process to perform extra action
                        // if the process has been tracked before, update its running status
                        let delta_cpu_core_time = luminar_process_util.cpu_usage * self.refresh_interval;
                        let delta_gpu_device_time = luminar_process_util.gpu_usage * self.refresh_interval;
                        let mut process_tracker = luminar_user.process_trackers_map.entry(pid).or_insert( LuminarProcessTacker {
                            pid: pid,
                            utility: luminar_process_util.clone(),
                            cpu_core_time: 0.0,
                            gpu_device_time:0.0,
                            start_time: std::time::Instant::now(),
                            running_time: std::time::Instant::now() - std::time::Instant::now()
                        });
                        process_tracker.utility = luminar_process_util.clone();
                        process_tracker.running_time = std::time::Instant::now() - process_tracker.start_time;
                        process_tracker.cpu_core_time += delta_cpu_core_time;
                        process_tracker.gpu_device_time += delta_gpu_device_time;
                        // check pid tracker if the computing resources of the rule is all consumed, kill
                        // select the rule with the highest priority and 
                        // update process tracker information
                        let target_rule_name = String::from(matched_rules[0].name.clone());
                        let mut rule_tracker = luminar_user.rule_trackers_map.get_mut(&target_rule_name).expect(format!("failed to get rule from rule tracker with rule name {}",target_rule_name.as_str()).as_str());
                        rule_tracker.cpu_core_time += delta_cpu_core_time;
                        rule_tracker.gpu_device_time += delta_gpu_device_time;
                        rule_tracker.tracked_pids.push(pid);
                    }
                   
                }
                println!("user name: {:?}", luminar_user.name);

                // iterate through all rule trackers, take actions on the pids it tracks
                for (name, rule_tracker) in luminar_user.rule_trackers_map.iter_mut(){
                    println!("rule: {:?} pids: {:?} cpu core time: {:?}, gpu_device_time: {:?}",name, rule_tracker.tracked_pids, rule_tracker.cpu_core_time,rule_tracker.gpu_device_time);
                    println!("consumed: {:?}", rule_tracker.is_consumed());
                    if rule_tracker.is_consumed(){
                        println!("in consume");
                        for pid in rule_tracker.tracked_pids.iter(){
                            self.sys_reception.kill(pid);
                        }
                        println!("Rule {:?} has been consumed, tracked pids are killed {:?}", name, rule_tracker.tracked_pids);
                    }
                }

            }
            print!("\x1B[2J\x1B[1;1H");
            sleep(std::time::Duration::from_secs_f32(self.refresh_interval));
        }
    }
    fn global_filter_rules(&self,mut uid_process_info_map: HashMap<u32,Vec<LuminarProcessUtil>>)->HashMap<u32,Vec<LuminarProcessUtil>>{
        for (_, processes) in uid_process_info_map.iter_mut(){
            processes.retain(|p| self.rule_filters.iter().all(|filter|  filter.retain(&p)));

        }
        uid_process_info_map
    }

}

#[derive(Debug)]
pub struct LuminarSystemReception {
    // assistant variable for fetching process info
    sys: sysinfo::System,     // for CPU info
    nvml: nvml_wrapper::Nvml, // for GPU info
}
impl LuminarSystemReception {
    pub fn new() -> LuminarSystemReception {
        let sys = sysinfo::System::new_all();
        let nvml = nvml_wrapper::Nvml::init().expect("failed to init nvml");
        let mut sys_reception = LuminarSystemReception {
            sys: sys,
            nvml: nvml,
        };
        sys_reception.get_pid_uid_map();
        sys_reception
        // sys_reception
        // return l;
    }
    // Dec,31,2022: win the combat with Zixun Yu about using str and String
    // choose to use String here
    // TODO: remove borrow in return value, change to u32
    pub fn get_uid_by_uname(&self, uname: &String) -> Option<&u32> {
        self.sys
            .users()
            .iter()
            .find(|u| u.name() == uname.as_str())
            .map(|u| u.id().deref())
    }

    // the method get gpu info in a tuple of (gpu_instance_id, smutil, memory) and return a HashMap of pid to this tuple.
    pub fn get_pid_gpu_info_map(&mut self) -> HashMap<u32, (u32, u32, u64)> {
        let Ok(gpu_device_count) = self.nvml.device_count() else{
            return HashMap::new();
        };
        let gpu_devices = (0..gpu_device_count)
            .filter_map(|i| self.nvml.device_by_index(i).ok())
            .collect::<Vec<nvml_wrapper::Device>>();
        let mut gpu_info_map = HashMap::new();
        for device in gpu_devices.iter() {
            let Ok(device_idx) = device.index() else{
                continue;
            };
            let Ok(compute_processes) = device.running_compute_processes() else{
                continue;
            };
            for compute_process in compute_processes.iter() {
                use nvml_wrapper::enums::device::UsedGpuMemory::Used;
                let Used(gpu_memory) = compute_process.used_gpu_memory else{
                    continue;
                };
                gpu_info_map.insert(compute_process.pid, (device_idx, 0, gpu_memory));
            }
            let Ok(processes_sample) = device.process_utilization_stats(None) else{
                continue; 
            };
            for process_sample in processes_sample.iter() {
                if let Some(process_info) = gpu_info_map.get_mut(&process_sample.pid) {
                    (*process_info).1 = process_sample.sm_util;
                }
            }
        }
        gpu_info_map
    }
    // pub fn get_process_usage_by_pid(&self, pid: &u32) -> Option<LuminarProcessUtil> {}
    pub fn get_pid_uid_map(&mut self) -> HashMap<u32, u32> {
        self.sys.refresh_all();
        // processes can vanish, appear or maintain in the system. To make the logic clear, init all related values when refresh to update
        // clear pid_uid_map to init all values from strach when refresh
        self
            .sys
            .processes()
            .iter()
            .filter(|pair| Some(pair.1).is_some())
            .map(|(pid, process)| (pid.as_u32(), process.user_id().expect("[alert: this error should not occur] failed to get user id from process").deref().clone()))
            .collect::<HashMap<_, _>>()
    }

    pub fn get_uid_process_info_map(&mut self) -> HashMap<u32, Vec<LuminarProcessUtil>> {
        self.sys.refresh_all();
        let pid_gpu_info_map = self.get_pid_gpu_info_map();
        // clear uid_process_map to init all values from strach when refresh
        let mut uid_process_info_map: HashMap<u32, Vec<LuminarProcessUtil>> = HashMap::new();
        for (pid, process) in self.sys.processes() {
            let pid_u32 = pid.as_u32();
            let Some(uid_u32) = process.user_id().map(|uid| uid.deref().clone()) else {
                continue;
            };
            let gpu_info = pid_gpu_info_map.get(&pid_u32).cloned().unwrap_or((0, 0, 0));
            uid_process_info_map
                .entry(uid_u32)
                .or_default()
                .push(LuminarProcessUtil {
                    pid: pid_u32,
                    commond: process.cmd().to_vec(),
                    cpu_usage: process.cpu_usage() / 100.0f32 , // convert from f32 [0,100] to f32 [0, 1].
                    cpu_memory: process.memory()/1e6 as u64, // from bytes to mb
                    device_id: gpu_info.0,
                    gpu_usage: gpu_info.1 as f32 / 100.0f32, // convert from u32 [0, 100] to f32 [0, 1].
                    gpu_memory: gpu_info.2 / 1e6 as u64,
                });
        }
        uid_process_info_map
    }

    pub fn kill(&self,pid:&u32)->bool{
        println!("in kill");
        if let Some(process) = self.sys.process(sysinfo::Pid::from_u32(pid.clone())){
            return process.kill();
        }
        println!("failed to get process from pid {:?}", pid);
        false
    }
}

#[derive(Debug)]
pub struct LuminarProcessTacker{
    pub pid:u32,
    pub utility: LuminarProcessUtil,
    pub start_time: std::time::Instant,
    pub running_time:std::time::Duration,
    pub cpu_core_time:f32, // in seconds
    pub gpu_device_time:f32, // in seconds
}



#[derive(Debug)]
pub struct LuminarRuleTracker{
    pub name: String,
    pub rule: LuminarRule,
    pub cpu_core_time:f32, // in seconds
    pub gpu_device_time:f32, // in seconds
    pub tracked_pids: Vec<u32>,
}

impl LuminarRuleTracker{
    // the 
    fn is_consumed(&self)->bool{
        println!("{:?}",self);
        println!("{:?} {:?}", (self.cpu_core_time/60.0f32 ) as u64 ,self.rule.max_cpu_core_time);
        println!("{:?} ", (self.cpu_core_time/60.0f32 ) as u64 >= self.rule.max_cpu_core_time);
        println!("{:?} {:?}",(self.gpu_device_time / 60.0f32 ) as u64 , self.rule.max_gpu_device_time);
        (self.cpu_core_time/60.0f32 ) as u64 >= self.rule.max_cpu_core_time || (self.gpu_device_time / 60.0f32 ) as u64 >= self.rule.max_gpu_device_time
    }

}