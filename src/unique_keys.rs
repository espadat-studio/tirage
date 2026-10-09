use std::fmt;
use std::marker::PhantomData;

use serde::de::{Deserialize, Deserializer, Error, MapAccess, Visitor};

pub(crate) fn deserialize<'de, D, V>(
    deserializer: D,
    scope: &'static str,
) -> Result<Vec<(String, V)>, D::Error>
where
    D: Deserializer<'de>,
    V: Deserialize<'de>,
{
    deserializer.deserialize_map(UniqueKeys {
        scope,
        values: PhantomData,
    })
}

struct UniqueKeys<V> {
    scope: &'static str,
    values: PhantomData<V>,
}

impl<'de, V: Deserialize<'de>> Visitor<'de> for UniqueKeys<V> {
    type Value = Vec<(String, V)>;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a map")
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let mut entries: Vec<(String, V)> = Vec::new();
        while let Some((key, value)) = map.next_entry::<String, V>()? {
            if entries.iter().any(|(seen, _)| *seen == key) {
                return Err(A::Error::custom(format!(
                    "{}duplicate field `{key}`",
                    self.scope
                )));
            }
            entries.push((key, value));
        }
        Ok(entries)
    }
}
