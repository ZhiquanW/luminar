use std::io::{BufRead, Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

use nvml_wrapper::enums::device::BusType;

use crate::backend::LuminarResManager;
use crate::dist_cmd;
use crate::network::LuminarNetServer;
pub struct LuminarService {
    pub res_manager: Arc<Mutex<LuminarResManager>>,
    pub net_server: LuminarNetServer,
}

impl LuminarService {
    pub fn new(res_manager: LuminarResManager, net_server: LuminarNetServer) -> LuminarService {
        LuminarService {
            res_manager: Arc::new(Mutex::new(res_manager)),
            net_server: net_server,
        }
    }
    pub fn launch(self) {
        println!("[luminar] start launching service.");
        for stream in self.net_server.listerner.incoming() {
            if let Ok(stream) = stream {
                let shared_res_manager = Arc::clone(&self.res_manager);
                self.net_server.pool.execute(|| {
                    LuminarService::handle_request(stream, shared_res_manager);
                });
            } else {
                println!("parse Result<TcpStream> results Error")
            }
        }
    }
    fn handle_request(mut stream: TcpStream, res_manager: Arc<Mutex<LuminarResManager>>) {
        let buf_reader = std::io::BufReader::new(&stream);
        let mut buf = [0; 512];
        let _ = stream.read(&mut buf).unwrap();
        let cmd = String::from_utf8(buf.to_vec()).unwrap();
        let cmd = cmd.trim_matches(char::from(0));
        stream.write("received your request".as_bytes()).unwrap();
        println!("msg sent.");
    }
}
