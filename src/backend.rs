use chrono;
use chrono::Timelike;
use serde;
use std::fs;
use std::io::Write;
use std::ops::Deref;
use std::path::PathBuf;
use std::{collections::HashMap, path::Path};
use sysinfo::{PidExt, ProcessExt, SystemExt, UserExt};

use crate::cfg::{LuminarRule, LuminarRuleFilter, LuminarUserInfo};
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

/// # track the computing resource of a process with accumulated resource usage
///
/// - this struct is created when a new process is deteced and added to user_process_tracker in LuminarResManager.
/// - it is used for computing the LumianrRuleTracker and Lumainr Timeline for logging.
#[derive(Debug)]
pub struct LuminarProcessTacker {
    pub pid: u32,
    // LumianrProcessUtil stores the current resource usage of the process
    pub utility: LuminarProcessUtil,
    pub start_time: std::time::Instant,
    pub running_time: std::time::Duration,
    pub cpu_core_time: f64,    // in num_core *seconds
    pub cpu_memory_time: u128, // in mb * seconds
    pub gpu_device_time: f64,  // in num_device * seconds
    pub gpu_memory_time: u128, // in mb * seconds
}
/// # track the computing resource of all the processes under this rule
///
#[derive(Debug, Default)]
pub struct LuminarRuleTracker {
    pub name: String, // the name of the rule, defined in the configuraton file
    pub rule: LuminarRule,
    pub cpu_core_time: f64,    // in num_core * seconds
    pub cpu_memory_time: u128, // in mb * seconds
    pub gpu_device_time: f64,  // in num_device * seconds
    pub gpu_memory_time: u128, // in mb * seconds
    pub tracked_pids: Vec<u32>,
}

/// manage the logging evernt at specific time and record previous logging info
///
#[derive(Debug)]
pub struct LuminarLoggingSystem {
    log_dir: PathBuf,
    pub next_log_time: chrono::DateTime<chrono::Local>,
    pub user_tracker_backups: HashMap<u32, LuminarUserTrackerBackup>,
}
impl LuminarLoggingSystem {
    fn should_log(&self) -> bool {
        return chrono::Local::now() > self.next_log_time;
    }
    fn log(&mut self) {
        if self.should_log() {
            if !self.log_dir.exists() {
                // Recursively create a directory and all of its parent components if they are missing.
                std::fs::create_dir_all(self.log_dir.as_path())
                    .expect(format!("failed to create folder: {:?}", self.log_dir).as_str());
            }

            let file_path = self
                .log_dir
                .join(format!("{}", chrono::Local::now().format("%Y-%m-%d_%H:%M")));

            // create file if not exist
            let mut file = fs::OpenOptions::new()
                .write(true)
                .create(true)
                .open(&file_path)
                .expect(&format!(
                    "failed to create log file: {:?}",
                    file_path.as_path()
                ));
            file.write_all(
                serde_json::to_string_pretty(&self.user_tracker_backups)
                    .expect("failed to convert user_timeline to string")
                    .as_str()
                    .as_bytes(),
            )
            .expect("failed to log");
            for (_uid, tracker_backup) in self.user_tracker_backups.iter_mut() {
                tracker_backup.reset();
            }
            self.next_log_time += chrono::Duration::days(1);
        }
    }
}
/// # record the computing resource of a user during a specific duraion
#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct LuminarUserTrackerBackup {
    pub name: String,          // user name
    pub cpu_core_time: f64,    // in num_core * seconds
    pub cpu_memory_time: u128, // in mb * seconds
    pub gpu_device_time: f64,  // in num_device * seconds
    pub gpu_memory_time: u128, // in mb * seconds
}

impl LuminarUserTrackerBackup {
    pub fn reset(&mut self) {
        self.cpu_core_time = 0.0;
        self.cpu_memory_time = 0;
        self.gpu_device_time = 0.0;
        self.gpu_memory_time = 0;
    }
}

impl LuminarRuleTracker {
    #[allow(dead_code)]
    fn is_consumed(&self) -> bool {
        println!("{:?}", self);
        println!(
            "{:?} {:?}",
            (self.cpu_core_time / 60.0f64) as u64,
            self.rule.max_cpu_core_time
        );
        println!(
            "{:?} ",
            (self.cpu_core_time / 60.0f64) as u64 >= self.rule.max_cpu_core_time
        );
        println!(
            "{:?} {:?}",
            (self.gpu_device_time / 60.0f64) as u64,
            self.rule.max_gpu_device_time
        );
        (self.cpu_core_time / 60.0f64) as u64 >= self.rule.max_cpu_core_time
            || (self.gpu_device_time / 60.0f64) as u64 >= self.rule.max_gpu_device_time
    }
}
#[derive(Debug, Default)]
pub struct LuminarUser {
    pub luminar: bool, // True, if this user should be managed by luminar
    pub uid: u32,
    pub name: String,
    pub process_trackers_map: HashMap<u32, LuminarProcessTacker>, // a map from pid to process
    pub rule_trackers_map: HashMap<String, LuminarRuleTracker>,   // map rule name -> rule tracker
}

#[derive(Debug)]
pub struct LuminarResManager {
    pub refresh_interval: f32, // in seconds
    sys_reception: LuminarSystemReception,
    pub user_process_tracker: HashMap<u32, LuminarUser>, // store uid -> Luminar User Process Tracking data, for all users on the machine
    pub logging_system: LuminarLoggingSystem,
    #[allow(dead_code)]
    rule_filters: Vec<LuminarRuleFilter>, // store global rule filters, not suppose to be update frequently
}

impl LuminarResManager {
    pub fn new<P: AsRef<Path>>(
        working_dir: P,
        mut users_info: Vec<LuminarUserInfo>,
        rule_filters: Vec<LuminarRuleFilter>,
        common_rules: Vec<LuminarRule>,
        refresh_interval: f32,
    ) -> LuminarResManager {
        let sys_reception = LuminarSystemReception::new();
        // init user process tracker with empty maps
        let mut user_process_tracker = sys_reception
            .get_uid_uname_map()
            .iter()
            .map(|(uid, uname)| {
                (
                    uid.clone(),
                    LuminarUser {
                        uid: uid.clone(),
                        name: String::from(uname.clone()),
                        ..Default::default()
                    },
                )
            })
            .collect::<HashMap<u32, LuminarUser>>();
        for user_info in users_info.iter_mut() {
            // get user, ignore this user if uid does not exist
            let uid = match sys_reception.get_uid_by_uname(&user_info.name) {
                Some(uname) => uname,
                None => continue,
            };
            let luminar_user = match user_process_tracker.get_mut(uid) {
                Some(user) => user,
                None => continue,
            };
            luminar_user.luminar = true;
            // append all common rules to user rules
            user_info.rules.extend(common_rules.iter().cloned());
            luminar_user.rule_trackers_map = user_info
                .rules
                .iter()
                .map(|rule| {
                    (
                        rule.name.clone(),
                        LuminarRuleTracker {
                            name: rule.name.clone(),
                            rule: rule.clone(),
                            ..Default::default()
                        },
                    )
                })
                .collect();
        }
        let log_time = chrono::Local::now()
            .with_hour(23)
            .unwrap()
            .with_minute(59)
            .unwrap()
            .with_second(59)
            .unwrap();
        let logging_system = LuminarLoggingSystem {
            log_dir: working_dir.as_ref().to_owned(),
            next_log_time: log_time,
            user_tracker_backups: sys_reception
                .get_uid_uname_map()
                .iter()
                .map(|(uid, uname)| {
                    (
                        uid.clone(),
                        LuminarUserTrackerBackup {
                            name: String::from(uname.deref()),
                            ..Default::default()
                        },
                    )
                })
                .collect(),
        };
        LuminarResManager {
            refresh_interval: refresh_interval,
            sys_reception: sys_reception,
            logging_system: logging_system,
            user_process_tracker: user_process_tracker,
            rule_filters: rule_filters,
        }
    }

    pub fn monitor_update(&mut self) {
        let uid_process_info_map = self.sys_reception.get_uid_process_info_map();
        // update each user's processes utility
        for (uid, luminar_user) in self.user_process_tracker.iter_mut() {
            let luminar_process_util = match uid_process_info_map.get(uid) {
                Some(process_utils) => process_utils,
                None => continue,
            }; // update process backup
            let user_backup = self
                .logging_system
                .user_tracker_backups
                .get_mut(uid)
                .expect("failed to get user from logging system");
            // iterate through process util of the user to record its inforamtion
            for luminar_process_util in luminar_process_util.iter() {
                // track the process
                // if the process has been tracked before, update its running status
                let delta_cpu_core_time =
                    (luminar_process_util.cpu_usage * self.refresh_interval) as f64;
                let delta_gpu_device_time =
                    (luminar_process_util.gpu_usage * self.refresh_interval) as f64;
                let delta_cpu_memory_time =
                    (luminar_process_util.cpu_memory as f32 * self.refresh_interval) as u128;
                let delta_gpu_memory_time =
                    (luminar_process_util.gpu_memory as f32 * self.refresh_interval) as u128;
                let mut process_tracker = luminar_user
                    .process_trackers_map
                    .entry(luminar_process_util.pid)
                    .or_insert(LuminarProcessTacker {
                        pid: luminar_process_util.pid,
                        utility: luminar_process_util.clone(),
                        cpu_core_time: 0.0,
                        cpu_memory_time: 0,
                        gpu_device_time: 0.0,
                        gpu_memory_time: 0,
                        start_time: std::time::Instant::now(),
                        running_time: std::time::Instant::now() - std::time::Instant::now(),
                    });
                process_tracker.utility = luminar_process_util.clone();
                process_tracker.running_time =
                    std::time::Instant::now() - process_tracker.start_time;
                process_tracker.cpu_core_time += delta_cpu_core_time;
                process_tracker.cpu_memory_time += delta_cpu_memory_time;
                process_tracker.gpu_device_time += delta_gpu_device_time;
                process_tracker.gpu_memory_time += delta_gpu_memory_time;

                user_backup.cpu_core_time += delta_cpu_core_time;
                user_backup.cpu_memory_time += delta_cpu_memory_time;
                user_backup.gpu_device_time += delta_gpu_device_time;
                user_backup.gpu_memory_time += delta_gpu_memory_time;
            }
        }
    }
    pub fn log_update(&mut self) {
        self.logging_system.log();
    }
    pub fn display_update(&self) {
        // history usage print

        use prettytable::Table;
        let mut usage_table = Table::new();
        usage_table.add_row(row![
            "User Name",
            "CPU Usage\n(core * minutes)",
            "Memory \n(MB * minutes)",
            "GPU Usage\n(device * minutes)",
            "GPU Memory \n(MB * minutes)"
        ]);
        for (_uid, backup) in self.logging_system.user_tracker_backups.iter() {
            usage_table.add_row(row![
                backup.name,
                format!("{:.3}", backup.cpu_core_time / 60.0).as_str(),
                format!("{:.3}", backup.cpu_memory_time / 60).as_str(),
                format!("{:.3}", backup.gpu_device_time / 60.0).as_str(),
                format!("{:.3}", backup.gpu_memory_time / 60).as_str()
            ]);
        }
        // Print the table to stdout
        print!("\x1B[2J\x1B[1;1H");

        println!("version {}", env!("CARGO_PKG_VERSION"));
        println!(
            "next log time: {}",
            self.logging_system
                .next_log_time
                .format("%Y-%m-%d %H:%M:%S")
        );
        usage_table.printstd();

        // backup usage print
    }
    // pub fn update(&mut self) {
    //     let uid_process_info_map = self.sys_reception.get_uid_process_info_map();
    //     let uid_process_info_map = self.global_filter_rules(uid_process_info_map);
    //     // iterate through each luminar user and process their process and ß
    //         let Some(luminar_processes_util) = uid_process_info_map.get(uid) else{
    //         continue;
    //         };
    //         // clear all tracked pids for the next step to push
    //         for (_, rule_tracker) in luminar_user.rule_trackers_map.iter_mut() {
    //             rule_tracker.tracked_pids.clear();
    //         }
    //         // iterate through each process, track the running status and address it by corresponding rule
    //         for luminar_process_util in luminar_processes_util.iter() {
    //             let pid = luminar_process_util.pid;

    //             // find the corresponding rule of the process, update the rule consumption map
    //             // iterate through user rules, leave the matched rules
    //             let mut matched_rules: Vec<LuminarRule> = luminar_user
    //                 .rule_trackers_map
    //                 .iter()
    //                 .map(|(_, rt)| rt.rule.clone())
    //                 .filter_map(|r| r.match_process(luminar_process_util).then_some(r.clone()))
    //                 .collect();
    //             matched_rules.sort_by(|a, b| b.priority.cmp(&a.priority));
    //             // if no rules is matched, this process is illegal, kill
    //             if matched_rules.is_empty() {
    //                 if self.sys_reception.kill(&pid) {
    //                     println!(
    //                         "process {:?}, out of any rules. Killed",
    //                         luminar_process_util.pid
    //                     );
    //                 } else {
    //                     println!("failed to kill process {:?}", luminar_process_util.pid);
    //                 }
    //             } else {
    //                 // if a rule is matched, track the process to perform extra action
    //                 // if the process has been tracked before, update its running status
    //                 let delta_cpu_core_time =
    //                     luminar_process_util.cpu_usage * self.refresh_interval;
    //                 let delta_gpu_device_time =
    //                     luminar_process_util.gpu_usage * self.refresh_interval;
    //                 let mut process_tracker = luminar_user
    //                     .process_trackers_map
    //                     .entry(pid)
    //                     .or_insert(LuminarProcessTacker {
    //                         pid: pid,
    //                         utility: luminar_process_util.clone(),
    //                         cpu_core_time: 0.0,
    //                         cpu_memory_time: 0,
    //                         gpu_device_time: 0.0,
    //                         gpu_memory_time: 0,
    //                         start_time: std::time::Instant::now(),
    //                         running_time: std::time::Instant::now() - std::time::Instant::now(),
    //                     });
    //                 process_tracker.utility = luminar_process_util.clone();
    //                 process_tracker.running_time =
    //                     std::time::Instant::now() - process_tracker.start_time;
    //                 process_tracker.cpu_core_time += delta_cpu_core_time;
    //                 process_tracker.gpu_device_time += delta_gpu_device_time;
    //                 // check pid tracker if the computing resources of the rule is all consumed, kill
    //                 // select the rule with the highest priority and
    //                 // update process tracker information
    //                 let target_rule_name = String::from(matched_rules[0].name.clone());
    //                 let mut rule_tracker = luminar_user
    //                     .rule_trackers_map
    //                     .get_mut(&target_rule_name)
    //                     .expect(
    //                         format!(
    //                             "failed to get rule from rule tracker with rule name {}",
    //                             target_rule_name.as_str()
    //                         )
    //                         .as_str(),
    //                     );
    //                 rule_tracker.cpu_core_time += delta_cpu_core_time;
    //                 rule_tracker.gpu_device_time += delta_gpu_device_time;
    //                 rule_tracker.tracked_pids.push(pid);
    //             }
    //         }
    //         println!("user name: {:?}", luminar_user.name);

    //         // iterate through all rule trackers, take actions on the pids it tracks
    //         for (name, rule_tracker) in luminar_user.rule_trackers_map.iter_mut() {
    //             println!(
    //                 "rule: {:?} pids: {:?} cpu core time: {:?}, gpu_device_time: {:?}",
    //                 name,
    //                 rule_tracker.tracked_pids,
    //                 rule_tracker.cpu_core_time,
    //                 rule_tracker.gpu_device_time
    //             );
    //             println!("consumed: {:?}", rule_tracker.is_consumed());
    //             if rule_tracker.is_consumed() {
    //                 println!("in consume");
    //                 for pid in rule_tracker.tracked_pids.iter() {
    //                     self.sys_reception.kill(pid);
    //                 }
    //                 println!(
    //                     "Rule {:?} has been consumed, tracked pids are killed {:?}",
    //                     name, rule_tracker.tracked_pids
    //                 );
    //             }
    //         }
    //     }
    // }
    // pub fn launch(&mut self) {
    //     loop {
    //         self.update();
    //         print!("\x1B[2J\x1B[1;1H");
    //         sleep(std::time::Duration::from_secs_f32(self.refresh_interval));
    //     }
    // }
    #[allow(dead_code)]
    fn global_filter_rules(
        &self,
        mut uid_process_info_map: HashMap<u32, Vec<LuminarProcessUtil>>,
    ) -> HashMap<u32, Vec<LuminarProcessUtil>> {
        for (_, processes) in uid_process_info_map.iter_mut() {
            processes.retain(|p| self.rule_filters.iter().all(|filter| filter.retain(&p)));
        }
        uid_process_info_map
    }
}

/// a wrapper class for sysinfo and nvml_wrapper to get cpu and gpu usage information easily
///
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
        let sys_reception = LuminarSystemReception {
            sys: sys,
            nvml: nvml,
        };
        // sys_reception.get_pid_uid_map();
        sys_reception
    }
    // TODO: remove borrow in return value, change to u32
    pub fn get_uid_by_uname(&self, uname: &String) -> Option<&u32> {
        self.sys
            .users()
            .iter()
            .find(|u| u.name() == uname.as_str())
            .map(|u| u.id().deref())
    }
    pub fn get_uid_uname_map(&self) -> HashMap<u32, &str> {
        self.sys
            .users()
            .iter()
            .map(|user| (user.id().deref().clone(), user.name()))
            .collect::<HashMap<_, _>>()
    }
    // the method get gpu info in a tuple of (gpu_instance_id, smutil, memory) and return a HashMap of pid to this tuple.
    pub fn get_pid_gpu_info_map(&mut self) -> HashMap<u32, (u32, u32, u64)> {
        // let Ok(gpu_device_count) = self.nvml.device_count() else{
        //     return HashMap::new();
        // };
        let gpu_device_count = match self.nvml.device_count() {
            Ok(count) => count,
            Err(_) => return HashMap::new(),
        };
        let gpu_devices = (0..gpu_device_count)
            .filter_map(|i| self.nvml.device_by_index(i).ok())
            .collect::<Vec<nvml_wrapper::Device>>();
        let mut gpu_info_map = HashMap::new();
        for device in gpu_devices.iter() {
            let device_idx = match device.index() {
                Ok(device_idx) => device_idx,
                Err(_) => continue,
            };
            let compute_processes = match device.running_compute_processes() {
                Ok(ps) => ps,
                Err(_) => continue,
            };

            for compute_process in compute_processes.iter() {
                use nvml_wrapper::enums::device::UsedGpuMemory::Used;
                let Used(gpu_memory) = compute_process.used_gpu_memory else{
                    continue;
                };
                gpu_info_map.insert(compute_process.pid, (device_idx, 0, gpu_memory));
            }

            let processes_sample = match device.process_utilization_stats(None) {
                Ok(ps_sample) => ps_sample,
                Err(_) => continue,
            };
            for process_sample in processes_sample.iter() {
                if let Some(process_info) = gpu_info_map.get_mut(&process_sample.pid) {
                    (*process_info).1 = process_sample.sm_util;
                }
            }
        }
        gpu_info_map
    }

    pub fn get_uid_process_info_map(&mut self) -> HashMap<u32, Vec<LuminarProcessUtil>> {
        self.sys.refresh_all();
        let pid_gpu_info_map = self.get_pid_gpu_info_map();
        // clear uid_process_map to init all values from strach when refresh
        let mut uid_process_info_map: HashMap<u32, Vec<LuminarProcessUtil>> = HashMap::new();
        for (pid, process) in self.sys.processes() {
            let pid_u32 = pid.as_u32();
            // let Some(uid_u32) = process.user_id().map(|uid| uid.deref().clone()) else {
            //     continue;
            // };
            let uid_u32 = match process.user_id().map(|uid| uid.deref().clone()) {
                Some(uid) => uid,
                None => continue,
            };
            let gpu_info = pid_gpu_info_map.get(&pid_u32).cloned().unwrap_or((0, 0, 0));
            uid_process_info_map
                .entry(uid_u32)
                .or_default()
                .push(LuminarProcessUtil {
                    pid: pid_u32,
                    commond: process.cmd().to_vec(),
                    cpu_usage: process.cpu_usage() / 100.0, // convert from f32 [0,100] to f32 [0, 1].
                    cpu_memory: process.memory() / 1e6 as u64, // from bytes to mb
                    device_id: gpu_info.0,
                    gpu_usage: gpu_info.1 as f32 / 100.0, // convert from u32 [0, 100] to f32 [0, 1].
                    gpu_memory: gpu_info.2 / 1e6 as u64,  // from bytes to mb,
                });
        }
        uid_process_info_map
    }
    #[allow(dead_code)]
    pub fn kill(&self, pid: &u32) -> bool {
        println!("in kill");
        if let Some(process) = self.sys.process(sysinfo::Pid::from_u32(pid.clone())) {
            return process.kill();
        }
        println!("failed to get process from pid {:?}", pid);
        false
    }
}
