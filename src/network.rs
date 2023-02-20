use std::net::TcpListener;
use threadpool::ThreadPool;

#[derive(Debug)]
pub struct LuminarNetServer {
    pub listerner: TcpListener,
    pub pool: ThreadPool,
}
impl LuminarNetServer {
    pub fn new(addr: &str, num_worker: usize) -> LuminarNetServer {
        return LuminarNetServer {
            listerner: TcpListener::bind(addr).expect(
                format!("failed to init luminar network service at ip {:?}", addr).as_str(),
            ),
            pool: ThreadPool::new(num_worker),
        };
    }
}
