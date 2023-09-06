use byteorder::{LittleEndian, ReadBytesExt};

#[macro_use]
mod host;
use host::*;

mod protocol;
mod wire;

fn read_packet() -> Result<(), i32> {
    /* Read the length of the packet. */
    let buf = vec![0; 2];
    read(&buf, 2)?;

    /* Ensure it is within sane bounds. */
    let mut buf = &buf[..];
    let len = buf.read_u16::<LittleEndian>().unwrap();
    if len < 2 {
        console_log!(
            "Dropping invalid packet; impossible length field of {}",
            len
        );
        return Err(-1);
    }

    /* Read the rest of the packet. */
    let buf = vec![0; (len - 2) as usize];
    read(&buf, (len - 2) as i32)?;

    console_log!("Buffer: {:?}", buf);

    /* Validate and convert the packet to a struct. */
    let packet = match wire::de::from_bytes::<protocol::content::Packet>(&buf) {
        Ok(packet) => packet,
        Err(err) => {
            console_log!("Dropping invalid packet; {}", err);
            return Err(-1);
        }
    };

    console_log!("{:?}", packet);

    Ok(())
}

#[no_mangle]
pub extern "C" fn connect() {
    loop {
        match read_packet() {
            Ok(()) => (),
            Err(_) => {
                console_log!("Closing connection due to previous error");
                break;
            }
        }
    }
}
