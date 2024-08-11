mod uds_impl {
    #[cfg(unix)]
    pub use std::os::unix::net::{UnixStream, UnixListener, SocketAddr};

    #[cfg(windows)]
    pub use uds_windows::{UnixStream, UnixListener, SocketAddr};
}
mod unix_stream {
    use std::io;
    use std::net::Shutdown;
    use std::path::Path;
    use std::time::Duration;
    use crate::{SocketAddr, uds_impl};

    #[derive(Debug)]
    pub struct UnixStream(pub(crate) uds_impl::UnixStream);

    impl UnixStream {
        pub fn connect(path: impl AsRef<Path>) -> io::Result<Self> {
            Ok(Self(uds_impl::UnixStream::connect(path)?))
        }

        pub fn pair() -> io::Result<(Self, Self)> {
            let (sock1, sock2) = uds_impl::UnixStream::pair()?;
            Ok((Self(sock1), Self(sock2)))
        }

        pub fn try_clone(&self) -> io::Result<Self> {
            Ok(Self(self.0.try_clone()?))
        }

        pub fn local_addr(&self) -> io::Result<SocketAddr> {
            Ok(SocketAddr(self.0.local_addr()?))
        }

        pub fn peer_addr(&self) -> io::Result<SocketAddr> {
            Ok(SocketAddr(self.0.peer_addr()?))
        }

        pub fn set_read_timeout(&self, dur: Option<Duration>) -> io::Result<()> {
            self.0.set_read_timeout(dur)
        }

        pub fn set_write_timeout(&self, dur: Option<Duration>) -> io::Result<()> {
            self.0.set_write_timeout(dur)
        }

        pub fn read_timeout(&self) -> io::Result<Option<Duration>> {
            self.0.read_timeout()
        }

        pub fn write_timeout(&self) -> io::Result<Option<Duration>> {
            self.0.write_timeout()
        }

        pub fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
            self.0.set_nonblocking(nonblocking)
        }

        pub fn take_error(&self) -> io::Result<Option<io::Error>> {
            self.0.take_error()
        }

        pub fn shutdown(&self, how: Shutdown) -> io::Result<()> {
            self.0.shutdown(how)
        }
    }
}
mod unix_listener {
    use std::io;
    use std::path::Path;
    use crate::{Incoming, SocketAddr, uds_impl, UnixStream};

    #[derive(Debug)]
    pub struct UnixListener(uds_impl::UnixListener);

    impl UnixListener {
        pub fn bind(path: impl AsRef<Path>) -> io::Result<Self> {
            Ok(Self(uds_impl::UnixListener::bind(path)?))
        }

        pub fn accept(&self) -> io::Result<(UnixStream, SocketAddr)> {
            let (stream, addr) = self.0.accept()?;
            Ok((UnixStream(stream), SocketAddr(addr)))
        }

        pub fn try_clone(&self) -> io::Result<Self> {
            Ok(Self(self.0.try_clone()?))
        }

        pub fn local_addr(&self) -> io::Result<SocketAddr> {
            Ok(SocketAddr(self.0.local_addr()?))
        }

        pub fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
            self.0.set_nonblocking(nonblocking)
        }

        pub fn take_error(&self) -> io::Result<Option<io::Error>> {
            self.0.take_error()
        }

        pub fn incoming(&self) -> Incoming {
            Incoming { listener: self }
        }
    }

    impl<'a> IntoIterator for &'a UnixListener {
        type Item = io::Result<UnixStream>;
        type IntoIter = Incoming<'a>;

        fn into_iter(self) -> Self::IntoIter {
            self.incoming()
        }
    }
}
mod socket_addr {
    use std::path::Path;
    use crate::uds_impl;

    #[derive(Debug)]
    pub struct SocketAddr(pub(crate) uds_impl::SocketAddr);

    impl SocketAddr {
        pub fn as_pathname(&self) -> Option<&Path> {
            self.0.as_pathname()
        }

        pub fn is_unnamed(&self) -> bool {
            self.0.is_unnamed()
        }
    }
}
mod incoming {
    use std::io;
    use crate::{UnixListener, UnixStream};

    #[derive(Debug)]
    pub struct Incoming<'a> {
        pub(crate) listener: &'a UnixListener,
    }

    impl<'a> Iterator for Incoming<'a> {
        type Item = io::Result<UnixStream>;

        fn next(&mut self) -> Option<Self::Item> {
            Some(self.listener.accept().map(|s| s.0))
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            (usize::MAX, None)
        }
    }
}

pub use unix_stream::UnixStream;
pub use unix_listener::UnixListener;
pub use socket_addr::SocketAddr;
pub use incoming::Incoming;
