//! OSC 发送（纯标准库 UDP）
//! VRChat OSC：/chatbox/input [text, send, notify]

use std::net::UdpSocket;

fn pad4(b: &[u8]) -> Vec<u8> {
    let mut out = b.to_vec();
    out.push(0);
    while out.len() % 4 != 0 {
        out.push(0);
    }
    out
}

fn encode(address: &str, args: &[OscArg]) -> Vec<u8> {
    let mut tags = String::from(",");
    let mut payload: Vec<u8> = Vec::new();
    for a in args {
        match a {
            OscArg::Bool(b) => {
                tags.push(if *b { 'T' } else { 'F' });
            }
            OscArg::Int(i) => {
                tags.push('i');
                payload.extend_from_slice(&i.to_be_bytes());
            }
            OscArg::Float(f) => {
                tags.push('f');
                payload.extend_from_slice(&f.to_be_bytes());
            }
            OscArg::Str(s) => {
                tags.push('s');
                payload.extend_from_slice(&pad4(s.as_bytes()));
            }
        }
    }
    let mut out = Vec::new();
    out.extend_from_slice(&pad4(address.as_bytes()));
    out.extend_from_slice(&pad4(tags.as_bytes()));
    out.extend_from_slice(&payload);
    out
}

enum OscArg {
    Bool(bool),
    Int(i32),
    Float(f32),
    Str(String),
}

/// 往 VRChat 聊天框发消息并立即发送
pub fn send_chatbox(text: &str, port: u16, notify: bool) -> Result<(), String> {
    let addr = format!("127.0.0.1:{port}");
    let sock = UdpSocket::bind("0.0.0.0:0").map_err(|e| format!("创建 UDP socket 失败: {e}"))?;
    let bytes = encode(
        "/chatbox/input",
        &[
            OscArg::Str(text.into()),
            OscArg::Bool(true),
            OscArg::Bool(notify),
        ],
    );
    sock.send_to(&bytes, &addr)
        .map_err(|e| format!("OSC 发送失败（VRChat OSC 端口 {port} 未监听？）: {e}"))?;
    Ok(())
}
