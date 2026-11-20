use std::{
    io,
    net::{SocketAddr, UdpSocket},
};

pub fn command(bind: SocketAddr, token: &str) -> io::Result<String> {
    let ip = if bind.ip().is_unspecified() {
        // UDP connect chooses the outbound interface without sending traffic.
        let (local, route) = if bind.is_ipv4() {
            ("0.0.0.0:0", "192.0.2.1:80")
        } else {
            ("[::]:0", "[2001:db8::1]:80")
        };
        let socket = UdpSocket::bind(local)?;
        socket.connect(route)?;
        socket.local_addr()?.ip()
    } else {
        bind.ip()
    };
    Ok(format!(
        "tictactoe join {} --invite {token}",
        SocketAddr::new(ip, bind.port())
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preserves_explicit_address_and_port() {
        assert_eq!(
            command("192.168.1.8:9000".parse().unwrap(), "abc").unwrap(),
            "tictactoe join 192.168.1.8:9000 --invite abc"
        );
        assert_eq!(
            command("[::1]:9001".parse().unwrap(), "abc").unwrap(),
            "tictactoe join [::1]:9001 --invite abc"
        );
    }
}
