use std::{ops::Deref, str::FromStr};

pub struct SocketAddr(std::net::SocketAddr);

impl SocketAddr {
    pub fn into_inner(self) -> std::net::SocketAddr {
        self.0
    }
}

impl FromStr for SocketAddr {
    type Err = std::net::AddrParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(SocketAddr(s.parse()?))
    }
}

impl Deref for SocketAddr {
    type Target = std::net::SocketAddr;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
