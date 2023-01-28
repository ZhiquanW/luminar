use std::{
    default, fs,
    io::{Read, Write},
    net::TcpStream,
    path::Path,
};

use serde::__private::de;

use crate::{
    argparse,
    backend::LuminarResManager,
    cfg::{self, LuminarGlobalConfig, LuminarRule, LuminarRuleFilter, LuminarUserInfo},
    core::LuminarService,
    network::LuminarNetServer,
};
// this initial action will be called when luminar: run is first called on the machine
// the status is initialized or not is detect by checking the existance of configuration file: '/etc/luminar/luminarc',
// if the cfg fie is not created, a default config file will be created, and be returned
// else the existing cfg file will be created.
pub fn init_luminar() -> (
    Vec<LuminarUserInfo>,
    Vec<LuminarRuleFilter>,
    Vec<LuminarRule>,
    u32,
    f32,
) {
    let lumianr_cfg_folder = Path::new("/etc/luminar/");
    let lumianr_cfg_path = Path::new("/etc/luminar/luminarc");
    // Recursively create a directory and all of its parent components if they are missing.
    std::fs::create_dir_all(lumianr_cfg_folder)
        .expect(format!("failed to create folder: {:?}", lumianr_cfg_folder).as_str());
    // create cfg file if not exist
    if lumianr_cfg_path.exists() {
        let (users_info, rule_filters, common_rules, port, refresh_interval) =
            cfg::load_luminar_configuration(lumianr_cfg_path);
        return (
            users_info,
            rule_filters,
            common_rules,
            port,
            refresh_interval,
        );
    } else {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open("/etc/luminar/luminarc")
            .expect("failed to open file");
        let default_cfg = LuminarGlobalConfig::default();
        let res = file.write_all(
            serde_json::to_string_pretty(&default_cfg)
                .unwrap()
                .as_str()
                .as_bytes(),
        );
        return (
            Vec::new(),
            default_cfg.rule_filters,
            default_cfg.common_rules,
            default_cfg.port,
            default_cfg.refresh_interval,
        );
    }
}
pub fn distribute_commonds(cmd: argparse::Command) {
    let (users_info, rule_filters, common_rules, port, refresh_interval) = init_luminar();
    match cmd {
        argparse::Command::Run { cfg_path } => run(
            users_info,
            rule_filters,
            common_rules,
            port,
            refresh_interval,
        ),
        argparse::Command::Status => {
            let mut stream = TcpStream::connect("127.0.0.1:3114").unwrap();
            stream.write_all("status".as_bytes()).unwrap();
            let mut buffer = [0; 512];
            let mut reader = std::io::BufReader::new(&stream);
            println!("buf reader created");
            reader.read(&mut buffer).unwrap();
            println!("read from server:{}", String::from_utf8_lossy(&buffer));
        }
        argparse::Command::Mutex => todo!(),
    }
}

/*
run command runs luminar service which is the core module of luminar
the luminar service will be launched and run persistently which manage the computing resource based on the configuration file
the lumianr service consists of two modules:
1. compute resource manager: monitor the resource usage, kill process if the corresponding rule is consumed
2. communication service: luminar service run as a local server, all the commands should be sent from client as a network request.
the communication service listens to the request from the client once the luminar service is running
*/
fn run(
    users_info: Vec<LuminarUserInfo>,
    rule_filters: Vec<LuminarRuleFilter>,
    common_rules: Vec<LuminarRule>,
    port: u32,
    refresh_interval: f32,
) {
    let binding = format!("127.0.0.1:{}", port);
    // let ip_addr = binding.as_str();
    let service = LuminarService::new(
        LuminarResManager::new(users_info, rule_filters, common_rules, refresh_interval),
        LuminarNetServer::new(&binding, 4),
    );
    println!("[luminar] backend service initialized.");
    service.launch();
}

fn status() {}
