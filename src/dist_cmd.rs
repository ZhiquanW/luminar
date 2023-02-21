use std::{
    io::{Read, Write},
    net::TcpStream,
    path::Path,
};

use crate::{
    argparse,
    backend::LuminarResManager,
    core::LuminarServer,
    network::LuminarNetServer,
    utils::{ini_luminar_cfg_file, load_luminar_configuration},
};
// this initial action will be called when luminar: run is first called on the machine
// the status is initialized or not is detect by checking the existance of configuration file: '/etc/luminar/luminarc',
// if the cfg fie is not created, a default config file will be created, and be returned
// else the existing cfg file will be created.
pub fn distribute_commonds(cmd: argparse::Command) {
    match cmd {
        argparse::Command::Server { cfg_path } => server(cfg_path),
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
the luminar service consists of two modules:
1. compute resource manager: monitor the resource usage, kill process if the corresponding rule is consumed
2. communication service: luminar service run as a local server, all the commands should be sent from client as a network request.
the communication service listens to the request from the client once the luminar service is running
*/
fn server(working_dir: Option<String>) {
    let binding = match &working_dir {
        None => "/etc/luminar",
        Some(s) => s.as_str(),
    };
    let working_dir = Path::new(&binding);
    if !working_dir.exists() {
        // Recursively create a directory and all of its parent components if they are missing.
        std::fs::create_dir_all(working_dir)
            .expect(format!("failed to create folder: {:?}", working_dir).as_str());
    }
    // create default configuration file if no configuration file exist
    let luminar_cfg_path = working_dir.join("luminarc.json");
    if !luminar_cfg_path.exists() {
        ini_luminar_cfg_file(&luminar_cfg_path).expect("failed to init lumianr config file");
    }
    // load lumianr config file
    let (users_info, rule_filters, common_rules, port, refresh_interval) =
        load_luminar_configuration(&luminar_cfg_path);
    let ip_addr = format!("127.0.0.1:{}", port);
    let service = LuminarServer::new(
        LuminarResManager::new(
            working_dir,
            users_info,
            rule_filters,
            common_rules,
            refresh_interval,
        ),
        LuminarNetServer::new(&ip_addr, 4),
    );
    service.launch();
}
