use dwntp::constants::UDP_DEFAULT_LISTEN_PORT;
use std::net::UdpSocket;

fn main() -> std::io::Result<()> {
    {
        let addr = "0.0.0.0";
        let port = UDP_DEFAULT_LISTEN_PORT;
        let socket = format!("{}:{}", addr, port);

        let socket = UdpSocket::bind(socket)?;
        let mut buf = [0; 10];
        let (amt, src) = socket.recv_from(&mut buf)?;

        let buf = &mut buf[..amt];
        buf.reverse();
        socket.send_to(buf, &src)?;
    }

    Ok(())
}
