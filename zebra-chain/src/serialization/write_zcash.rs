use std::{
    io,
    net::{IpAddr, SocketAddr},
};

use byteorder::{BigEndian, LittleEndian, WriteBytesExt};

/// Extends [`Write`] with methods for writing Zcash/Bitcoin types.
///
/// [`Write`]: https://doc.rust-lang.org/std/io/trait.Write.html
pub trait WriteZcashExt: io::Write {
    /// Writes a `u64` using the Bitcoin `CompactSize` encoding.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use zebra_chain::serialization::WriteZcashExt;
    ///
    /// let mut buf = Vec::new();
    /// buf.write_compactsize(0x12).unwrap();
    /// assert_eq!(buf, b"\x12");
    ///
    /// let mut buf = Vec::new();
    /// buf.write_compactsize(0xfd).unwrap();
    /// assert_eq!(buf, b"\xfd\xfd\x00");
    ///
    /// let mut buf = Vec::new();
    /// buf.write_compactsize(0xaafd).unwrap();
    /// assert_eq!(buf, b"\xfd\xfd\xaa");
    ///
    /// let mut buf = Vec::new();
    /// buf.write_compactsize(0xbbaafd).unwrap();
    /// assert_eq!(buf, b"\xfe\xfd\xaa\xbb\x00");
    ///
    /// let mut buf = Vec::new();
    /// buf.write_compactsize(0x22ccbbaafd).unwrap();
    /// assert_eq!(buf, b"\xff\xfd\xaa\xbb\xcc\x22\x00\x00\x00");
    /// ```
    #[inline]
    fn write_compactsize(&mut self, n: u64) -> io::Result<()> {
        match n {
            0x0000_0000..=0x0000_00fc => self.write_u8(n as u8),
            0x0000_00fd..=0x0000_ffff => {
                self.write_u8(0xfd)?;
                self.write_u16::<LittleEndian>(n as u16)
            }
            0x0001_0000..=0xffff_ffff => {
                self.write_u8(0xfe)?;
                self.write_u32::<LittleEndian>(n as u32)
            }
            _ => {
                self.write_u8(0xff)?;
                self.write_u64::<LittleEndian>(n)
            }
        }
    }

    /// Write an `IpAddr` in Bitcoin format.
    #[inline]
    fn write_ip_addr(&mut self, addr: IpAddr) -> io::Result<()> {
        use std::net::IpAddr::*;
        let v6_addr = match addr {
            V4(ref v4) => v4.to_ipv6_mapped(),
            V6(v6) => v6,
        };
        self.write_all(&v6_addr.octets())
    }

    /// Write a `SocketAddr` in Bitcoin format.
    #[inline]
    fn write_socket_addr(&mut self, addr: SocketAddr) -> io::Result<()> {
        self.write_ip_addr(addr.ip())?;
        self.write_u16::<BigEndian>(addr.port())
    }

    /// Write a string in Bitcoin format.
    #[inline]
    fn write_string(&mut self, string: &str) -> io::Result<()> {
        self.write_compactsize(string.len() as u64)?;
        self.write_all(string.as_bytes())
    }

    /// Convenience method to write exactly 32 u8's.
    #[inline]
    fn write_32_bytes(&mut self, bytes: &[u8; 32]) -> io::Result<()> {
        self.write_all(bytes)
    }

    /// Convenience method to write exactly 64 u8's.
    #[inline]
    fn write_64_bytes(&mut self, bytes: &[u8; 64]) -> io::Result<()> {
        self.write_all(bytes)
    }
}

/// Mark all types implementing `Write` as implementing the extension.
impl<W: io::Write + ?Sized> WriteZcashExt for W {}
