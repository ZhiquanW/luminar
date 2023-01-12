use crate::cfg::{LuminarRule, LuminarRuleFilter, LuminarUserInfo};
use crate::core::LuminarManager;
use serde_json::{self, Value};
use std::collections::HashMap;
use std::fs;
use structopt::StructOpt;
mod argparse;
mod cfg;
mod core;
use std::env;

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

fn main() {
    let opt = argparse::LuminarArgs::from_args();
    println!("{:?}", opt.cmd);
    // let (luminar_users_info, luminar_rule_filters, luminar_common_rules) =
    //     load_luminar_configuration("luminar_users_conf.json");
    // let mut luminar_manager = LuminarManager::new(
    //     luminar_users_info,
    //     luminar_rule_filters,
    //     luminar_common_rules,
    //     0.5,
    // );
    // println!("{:#?}", luminar_manager.user_dict);
    // luminar_manager.launch();
}
