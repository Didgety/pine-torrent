use std::fmt;

use serde::{de::{self, Deserialize, Deserializer, Visitor},
            ser::{Serialize, Serializer}};

#[derive(Debug, Clone)]
pub struct Hashes(pub Vec<[u8; 20]>);

struct HashVisitor;

impl<'de> Visitor<'de> for HashVisitor {
    type Value = Hashes;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("A byte string with length that is a multiple of 20")
    }

    fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if value.len() % 20 != 0 {
            return Err(E::custom(format!("length is not 20: {}", value.len())));
        }
        // TODO use array_chunks once stable
        // https://doc.rust-lang.org/std/slice/struct.ArrayChunks.html
        Ok(Hashes(
            value.chunks_exact(20)
                 .map(|slice_20| slice_20.try_into().expect("guaranteed 20 length"))
                 .collect(),
        ))
    }
}

impl<'de> Deserialize<'de> for Hashes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de>,
    {
        deserializer.deserialize_bytes(HashVisitor)
    }
}

impl Serialize for Hashes {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        let slice = self.0.concat();
        serializer.serialize_bytes(&slice)
    }
}