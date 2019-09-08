use std::io;
use std::net::{IpAddr, SocketAddr};

pub fn write_address<W: io::Write>(writer: &mut W, addr: &IpAddr) -> io::Result<()> {
    match addr {
        IpAddr::V4(addr) => writer.write_all(&addr.octets())?,
        IpAddr::V6(addr) => writer.write_all(&addr.octets())?,
    }
    Ok(())
}

pub fn write_socket_addr<W: io::Write>(writer: &mut W, addr: &SocketAddr) -> io::Result<()> {
    write_address(writer, &addr.ip())?;
    writer.write_all(&addr.port().to_be_bytes())?;
    Ok(())
}

pub fn read_v4_address<R: io::Read>(reader: &mut R) -> io::Result<IpAddr> {
    let mut buf = [0; 4];
    reader.read_exact(&mut buf)?;
    Ok(IpAddr::V4(buf.into()))
}

pub fn read_v6_address<R: io::Read>(reader: &mut R) -> io::Result<IpAddr> {
    let mut buf = [0; 16];
    reader.read_exact(&mut buf)?;
    Ok(IpAddr::V6(buf.into()))
}

pub fn read_v4_socket_address<R: io::Read>(reader: &mut R) -> io::Result<SocketAddr> {
    let mut addr = [0; 4];
    let mut port = [0; 2];
    reader.read_exact(&mut addr)?;
    reader.read_exact(&mut port)?;
    Ok(SocketAddr::new(
        IpAddr::V4(addr.into()),
        u16::from_be_bytes(port),
    ))
}

pub fn read_v6_socket_address<R: io::Read>(reader: &mut R) -> io::Result<SocketAddr> {
    let mut addr = [0; 16];
    let mut port = [0; 2];
    reader.read_exact(&mut addr)?;
    reader.read_exact(&mut port)?;
    Ok(SocketAddr::new(
        IpAddr::V6(addr.into()),
        u16::from_be_bytes(port),
    ))
}

pub fn lower_bound<T: PartialOrd>(values: &[T], value_to_check: &T) -> usize {
    let mut i = 0;
    for v in values {
        if value_to_check > v {
            break;
        }
        i += 1;
    }
    i
}
