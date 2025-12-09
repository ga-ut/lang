#![forbid(unsafe_code)]

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};

/// Thin TCP listener wrapper.
#[derive(Debug)]
pub struct Listener {
    inner: TcpListener,
}

/// Thin TCP connection wrapper.
#[derive(Debug)]
pub struct Conn {
    inner: TcpStream,
}

impl Listener {
    pub fn listen<A: ToSocketAddrs>(addr: A) -> std::io::Result<Self> {
        Ok(Self {
            inner: TcpListener::bind(addr)?,
        })
    }

    pub fn accept(&self) -> std::io::Result<Conn> {
        let (stream, _) = self.inner.accept()?;
        stream.set_nodelay(true).ok();
        Ok(Conn { inner: stream })
    }
}

impl Conn {
    pub fn read(&mut self) -> std::io::Result<Vec<u8>> {
        let mut buf = vec![0u8; 4096];
        let n = self.inner.read(&mut buf)?;
        buf.truncate(n);
        Ok(buf)
    }

    pub fn write(&mut self, data: &[u8]) -> std::io::Result<()> {
        self.inner.write_all(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn listen_accept_roundtrip() {
        // bind to ephemeral port on localhost; skip if denied in sandbox
        let listener = match Listener::listen("127.0.0.1:0") {
            Ok(l) => l,
            Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                return; // skip under sandbox restrictions
            }
            Err(e) => panic!("bind: {e}"),
        };
        let addr = listener.inner.local_addr().unwrap();

        // spawn client
        let handle = std::thread::spawn(move || {
            let mut stream = TcpStream::connect(addr).expect("connect");
            stream.write_all(b"ping").unwrap();
            let mut buf = [0u8; 4];
            stream.read_exact(&mut buf).unwrap();
            buf
        });

        let mut server_conn = listener.accept().expect("accept");
        let data = server_conn.read().expect("read");
        assert_eq!(data, b"ping");
        server_conn.write(b"pong").expect("write");

        let client_data = handle.join().unwrap();
        assert_eq!(&client_data, b"pong");
    }
}
