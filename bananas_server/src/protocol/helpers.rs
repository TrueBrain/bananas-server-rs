use serde::{Deserialize, Deserializer, Serialize};

use super::super::wire;

#[derive(Serialize, Debug)]
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
        wire::de::VecLen::<L, T>::deserialize(deserializer)
            .map(|x| VecLen::<L, T>(std::marker::PhantomData, x.items))
    }
}
