use std::sync::{Arc, Mutex};
use std::thread::{self, sleep};

use crate::backend::LuminarResManager;
use crate::network::LuminarNetServer;
pub struct LuminarServer {
    pub res_manager: Arc<Mutex<LuminarResManager>>,
    pub net_server: LuminarNetServer,
}

impl LuminarServer {
    pub fn new(res_manager: LuminarResManager, net_server: LuminarNetServer) -> LuminarServer {
        LuminarServer {
            res_manager: Arc::new(Mutex::new(res_manager)),
            net_server: net_server,
        }
    }
    // main method

    pub fn launch(self) {
        println!("lumianr starts running ...");
        let shared_res_manager = Arc::clone(&self.res_manager);
        let monitor_thread = thread::spawn(move || loop {
            let mut res_manager_guard = shared_res_manager.lock().expect("msaaag");
            let refresh_interval = res_manager_guard.refresh_interval;
            res_manager_guard.monitor_update();
            res_manager_guard.log_update();
            drop(res_manager_guard);
            sleep(std::time::Duration::from_secs_f32(refresh_interval));
        });
        monitor_thread
            .join()
            .expect("failed to join mointor thread");
    }
    // #[allow(dead_code)]
    // fn handle_request(mut stream: TcpStream, res_manager: Arc<Mutex<LuminarResManager>>) {
    //     let buf_reader = std::io::BufReader::new(&stream);
    //     let mut buf = [0; 512];
    //     let _ = stream.read(&mut buf).unwrap();
    //     let cmd = String::from_utf8(buf.to_vec()).unwrap();
    //     let cmd = cmd.trim_matches(char::from(0));
    //     stream.write("received your request".as_bytes()).unwrap();
    //     println!("msg sent.");
    // }
}
