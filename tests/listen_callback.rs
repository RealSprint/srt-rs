use std::os::raw::{c_char, c_int, c_void};

unsafe extern "C" fn accept_all(
    _opaq: *mut c_void,
    _ns: libsrt_sys::SRTSOCKET,
    _hsversion: c_int,
    _peeraddr: *const libsrt_sys::sockaddr,
    _streamid: *const c_char,
) -> c_int {
    0
}

/// libsrt >= 1.5.4 rejects `srt_listen_callback` on a socket that is already
/// listening, so the builder has to install the hook before `srt_listen`.
#[test]
fn listen_with_callback_succeeds() {
    srt_rs::startup().unwrap();

    let listener = srt_rs::async_builder()
        .set_live_transmission_type()
        .set_linger(0)
        .set_receive_latency(120)
        .listen("127.0.0.1:0", 2, Some(accept_all), None)
        .expect("listen with a callback installed");

    assert!(listener.local_addr().unwrap().port() != 0);
    listener.close().unwrap();

    let listener = srt_rs::builder()
        .set_live_transmission_type()
        .listen("127.0.0.1:0", 2, Some(accept_all), None)
        .expect("sync listen with a callback installed");
    listener.close().unwrap();
}
