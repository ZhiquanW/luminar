use dist_cmd::distribute_commonds;
use structopt::StructOpt;
mod argparse;
#[macro_use]
extern crate prettytable;
mod backend;
mod cfg;
mod core;
mod dist_cmd;
mod network;
mod utils;
fn main() {
    let args = argparse::LuminarArgs::from_args();
    distribute_commonds(args.cmd);
}

// fn handle_request(mut stream: TcpStream) {
//     use std::io::{BufRead, Read};
//     let buf_reader = std::io::BufReader::new(&mut stream);
//     let http_request: Vec<_> = buf_reader
//         .lines()
//         .map(|result| result.unwrap())
//         .take_while(|line| !line.is_empty())
//         .collect();
//     std::thread::sleep(Duration::from_secs(2));
// }
// fn run() {
//     use ::std::net::TcpListener;
//     let listener = TcpListener::bind("127.0.0.1:3114").expect("failed to bind listener");
//     println!("[luminar service] start listen requests");
//     let pool = ThreadPool::new(4);
//     for stream in listener.incoming() {
//         let stream = stream.expect("failed to establish connectionfrom incoming tcp stream");
//         pool.execute(|| handle_request(stream));
//     }
// }
// fn status() {
//     use std::io::Write;
//     use std::net::TcpStream;
//     println!("status");
//     let mut stream = TcpStream::connect("127.0.0.1:3114").unwrap();
//     stream.write_all("status".as_bytes()).unwrap();
// }

// fn add(data: Arc<Mutex<LuminarResManager>>) {
//     println!("Add in");
//     for _ in 0..16 {
//         {
//             let mut data = data.lock().expect("failed to lock data in add.");
//             data.refresh_interval += 1.0;
//             println!("add: {:?}", data.refresh_interval);
//         }
//         let mut rng = rand::thread_rng();
//         let n1: f32 = rng.gen();
//         std::thread::sleep(Duration::from_secs_f32(n1));
//     }
// }

// fn double_minus(data: Arc<Mutex<LuminarResManager>>) {
//     println!("Minus in");
//     for _ in 0..16 {
//         {
//             let mut data = data.lock().expect("failed to lock data in double_minus.");
//             data.refresh_interval -= 1.0;
//             println!("double_minus: {:?}", data.refresh_interval);
//         }
//         let mut rng = rand::thread_rng();
//         let n1: f32 = rng.gen();
//         std::thread::sleep(Duration::from_secs_f32(n1));
//     }
// }
// fn mutex() {
//     let (luminar_users_info, luminar_rule_filters, luminar_common_rules) =
//         load_luminar_configuration("luminar_users_conf.json");
//     let mut luminar_manager = LuminarResManager::new(
//         luminar_users_info,
//         luminar_rule_filters,
//         luminar_common_rules,
//         0.5,
//     );

//     let data = Arc::new(Mutex::new(luminar_manager));
//     let thread_data = Arc::clone(&data);
//     let add_thread = thread::spawn(move || add(thread_data));
//     let thread_data = Arc::clone(&data);
//     let minus_thread = thread::spawn(move || double_minus(thread_data));
//     std::thread::sleep(Duration::from_secs(2));
//     println!("AA");
//     add_thread.join();
//     println!("add_thread stoped");
//     minus_thread.join();
//     println!("miuns thread stopped");
// }
