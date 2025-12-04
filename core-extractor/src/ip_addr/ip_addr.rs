use std::{ops::Deref, str::FromStr};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IpAddr(std::net::IpAddr);

impl IpAddr {
    pub fn into_inner(self) -> std::net::IpAddr {
        self.0
    }
}

impl FromStr for IpAddr {
    type Err = std::net::AddrParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(IpAddr(s.parse()?))
    }
}

impl Deref for IpAddr {
    type Target = std::net::IpAddr;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
