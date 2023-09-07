use byteorder::{LittleEndian, ReadBytesExt};

#[macro_use]
mod host;
use host::*;

mod protocol;
mod wire;

#[derive(Debug)]
enum Error {
    ConnectionClosed,
    ReadFailure,
    WriteFailure,
    PacketTooSmall,
    PacketDeserializeFailure(wire::Error),
    PacketSerializeFailure(wire::Error),
}

fn read_packet() -> Result<(), Error> {
    /* Read the length of the packet. */
    let buf = vec![0; 2];
    let res = read(&buf, 2).map_err(|_| Error::ReadFailure)?;
    if res == 0 {
        return Err(Error::ConnectionClosed);
    }

    /* Ensure it is within sane bounds. */
    let mut buf = &buf[..];
    let len = buf
        .read_u16::<LittleEndian>()
        .map_err(|_| Error::PacketTooSmall)?;
    if len < 2 {
        return Err(Error::PacketTooSmall);
    }

    /* Read the rest of the packet. */
    let buf = vec![0; (len - 2) as usize];
    read(&buf, (len - 2) as i32).map_err(|_| Error::ReadFailure)?;
    if res == 0 {
        return Err(Error::ConnectionClosed);
    }

    /* Validate and convert the packet to a struct. */
    let packet = protocol::read_packet(&buf).map_err(|e| Error::PacketDeserializeFailure(e))?;

    match packet {
        protocol::ClientPacket::ClientInfoList {
            content_type,
            openttd_version: _,
            branches: _,
        } => {
            let packet = protocol::ServerInfo {
                content_type: content_type,
                content_id: 0,
                filesize: 1,
                name: "test".to_string(),
                version: "1.0".to_string(),
                url: "".to_string(),
                description: "description".to_string(),
                unique_id: 123,
                md5: [0; 16],
                dependencies: vec![].into(),
                tags: vec!["test".to_string(), "bla".to_string()].into(),
            };

            let buf = wire::to_bytes(&packet).map_err(|e| Error::PacketSerializeFailure(e))?;
            write(&buf, buf.len() as i32).map_err(|_| Error::WriteFailure)?;
        }
        _ => (),
    };

    Ok(())
}

#[no_mangle]
pub extern "C" fn connect() {
    loop {
        match read_packet() {
            Ok(()) => (),
            Err(e) => {
                match e {
                    Error::ConnectionClosed => (),
                    _ => console_log!("Connection error: {:?}", e),
                };
                break;
            }
        }
    }
}
