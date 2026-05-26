use tokio::io::{self, AsyncReadExt};

pub async fn read_varint<R: io::AsyncRead + Unpin>(reader: &mut R) -> io::Result<i32> {
    let mut result = 0;
    let mut shift = 0;
    loop {
        let byte = reader.read_u8().await?;
        result |= ((byte & 0x7F) as i32) << shift;
        if (byte & 0x80) == 0 {
            break;
        }
        shift += 7;
        if shift >= 32 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "VarInt too big"));
        }
    }
    Ok(result)
}

pub fn read_varint_sync(buf: &mut &[u8]) -> io::Result<i32> {
    let mut result = 0;
    let mut shift = 0;
    loop {
        if buf.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "VarInt truncated",
            ));
        }
        let byte = buf[0];
        *buf = &buf[1..];
        result |= ((byte & 0x7F) as i32) << shift;
        if (byte & 0x80) == 0 {
            break;
        }
        shift += 7;
        if shift >= 32 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "VarInt too big"));
        }
    }
    Ok(result)
}

pub fn encode_varint(mut value: i32) -> Vec<u8> {
    let mut buf = Vec::new();
    loop {
        if (value & !0x7F) == 0 {
            buf.push(value as u8);
            break;
        }
        buf.push(((value & 0x7F) | 0x80) as u8);
        value >>= 7;
    }
    buf
}
