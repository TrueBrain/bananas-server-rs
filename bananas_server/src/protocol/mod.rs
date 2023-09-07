use serde::ser::Error;
use serde::{Deserialize, Deserializer, Serialize};

mod content;
pub use content::*;

use super::wire;

#[derive(Debug)]
pub struct VecLen<L, T>(std::marker::PhantomData<L>, Vec<T>);

impl<'de, L, T> Deserialize<'de> for VecLen<L, T>
where
    T: Deserialize<'de>,
    L: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        wire::DeVecLen::<L, T>::deserialize(deserializer)
            .map(|x| VecLen::<L, T>(std::marker::PhantomData, x.items))
    }
}

impl<L, T> Serialize for VecLen<L, T>
where
    T: Serialize,
    L: Serialize + std::convert::TryFrom<usize>,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ::serde::ser::Serializer,
    {
        let len = L::try_from(self.1.len())
            .map_err(|_| S::Error::custom("VecLen too long for length size"))?;

        wire::SerVecLen::<L, T> {
            len,
            items: &self.1,
        }
        .serialize(serializer)
    }
}

impl<L, T> From<Vec<T>> for VecLen<L, T> {
    fn from(v: Vec<T>) -> Self {
        VecLen::<L, T>(std::marker::PhantomData, v)
    }
}

#[derive(Debug, Deserialize)]
#[serde(try_from = "content::ClientPacket")]
struct ClientPacketInternal(content::ClientPacket);

impl<'a> TryFrom<content::ClientPacket> for ClientPacketInternal {
    type Error = wire::Error;

    fn try_from(packet: content::ClientPacket) -> Result<Self, Self::Error> {
        let packet = match &packet {
            /* Run validation on packets that require this. */
            content::ClientPacket::ClientInfoList {
                content_type: _,
                openttd_version,
                branches,
            } => {
                if branches.is_some() && *openttd_version != 0xffffffff {
                    return Err(wire::Error::ValidationError(
                        "branches given, but openttd-version doesn't allow it".to_string(),
                    ));
                }
                if branches.is_none() && *openttd_version == 0xffffffff {
                    return Err(wire::Error::ValidationError(
                        "no branches given, but openttd-version demands it".to_string(),
                    ));
                }
                packet
            }
            _ => packet,
        };
        Ok(ClientPacketInternal(packet))
    }
}

pub fn read_packet(buf: &[u8]) -> Result<content::ClientPacket, wire::Error> {
    Ok(wire::from_bytes::<ClientPacketInternal>(buf)?.0)
}
