use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;

use serde_json::Value;

use crate::cfg::LuminarGlobalConfig;
use crate::cfg::LuminarRule;
use crate::cfg::LuminarRuleFilter;
use crate::cfg::LuminarUserInfo;

pub fn ini_luminar_cfg_file(cfg_path: &Path) -> Result<(), std::io::Error> {
    // create file if not exist
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(cfg_path)
        .expect("try to create default configuration file: failed to open file");
    let global_cfg = LuminarGlobalConfig::default();
    let mut default_cfg: HashMap<&str, Value> = HashMap::new();
    default_cfg.insert(
        "global_config",
        serde_json::to_value(&global_cfg)
            .expect("failed to convert LuminarGlobalConfig to serde_json::Value"),
    );
    default_cfg.insert(
        "user_infos",
        serde_json::to_value(Vec::<LuminarUserInfo>::new())
            .expect("failed to converte empty user_infos to serde_json::Value"),
    );
    file.write_all(
        serde_json::to_string_pretty(&default_cfg)
            .unwrap()
            .as_str()
            .as_bytes(),
    )
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
    println!("read configuration file from {:?}", cfg_path.as_os_str(),);
    // 1. load all data in json file
    let mut luminar_config: HashMap<String, serde_json::Value> =
        serde_json::from_str(&config_string).expect("failed to parse configuration file");
    // 2. parse all user infos, which is user_config in json
    let luminar_user_info = serde_json::from_value(
        luminar_config
            .remove("user_infos")
            .expect("failed to convert Value to LuminarUserInfo"),
    )
    .unwrap_or(Vec::new());
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
