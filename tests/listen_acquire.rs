#![cfg(target_os = "linux")]

use std::{
    net::{SocketAddr, UdpSocket},
    os::raw::{c_char, c_int, c_void},
    sync::Mutex,
    time::Duration,
};

use futures::future::select_all;
use socket2::{Domain, Protocol, Socket, Type};

const CLIENTS: usize = 8;

unsafe extern "C" fn record_socket(
    opaque: *mut c_void,
    ns: libsrt_sys::SRTSOCKET,
    _hsversion: c_int,
    _peeraddr: *const libsrt_sys::sockaddr,
    _streamid: *const c_char,
) -> c_int {
    let seen = &*(opaque as *const Mutex<Vec<i32>>);
    seen.lock().unwrap().push(ns);
    0
}

fn reuse_port_socket(addr: SocketAddr) -> UdpSocket {
    let socket = Socket::new(Domain::for_address(addr), Type::DGRAM, Some(Protocol::UDP)).unwrap();
    socket.set_reuse_port(true).unwrap();
    socket.bind(&addr.into()).unwrap();
    socket.into()
}

fn receive_queue_threads() -> usize {
    std::fs::read_dir("/proc/self/task")
        .unwrap()
        .filter_map(|task| std::fs::read_to_string(task.ok()?.path().join("comm")).ok())
        .filter(|comm| comm.starts_with("SRT:RcvQ"))
        .count()
}

/// Listeners sharing a port through SO_REUSEPORT each get their own multiplexer, and every
/// connection is accepted under the socket id its listen callback saw.
#[tokio::test]
async fn listeners_sharing_a_port_get_their_own_multiplexer() {
    srt_rs::startup().unwrap();
    let threads_before = receive_queue_threads();
    let seen = Box::new(Mutex::new(Vec::new()));
    let opaque = &*seen as *const Mutex<Vec<i32>> as *mut c_void;

    let first = reuse_port_socket("127.0.0.1:0".parse().unwrap());
    let addr = first.local_addr().unwrap();
    let listeners = [first, reuse_port_socket(addr)].map(|udp_socket| {
        srt_rs::async_builder()
            .set_live_transmission_type()
            .listen_acquire(
                udp_socket,
                CLIENTS as i32,
                Some(record_socket),
                Some(opaque),
            )
            .unwrap()
    });

    assert_eq!(receive_queue_threads(), threads_before + 2);
    for listener in &listeners {
        assert_eq!(listener.local_addr().unwrap(), addr);
    }

    let mut clients = Vec::new();
    for _ in 0..CLIENTS {
        let client = srt_rs::async_builder()
            .set_live_transmission_type()
            .connect(addr)
            .unwrap();
        clients.push(
            tokio::time::timeout(Duration::from_secs(5), client)
                .await
                .unwrap()
                .unwrap(),
        );
    }

    let mut accepted = Vec::new();
    while accepted.len() < CLIENTS {
        let accepts = listeners.iter().map(|listener| listener.accept());
        let (result, _, _) = tokio::time::timeout(Duration::from_secs(5), select_all(accepts))
            .await
            .unwrap();
        let (stream, _) = result.unwrap();
        accepted.push(stream.id());
    }

    let mut seen_ids = seen.lock().unwrap().clone();
    seen_ids.sort();
    accepted.sort();
    assert_eq!(accepted, seen_ids);

    drop(clients);
    drop(listeners);
    srt_rs::cleanup().unwrap();
}
