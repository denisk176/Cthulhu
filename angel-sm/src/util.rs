use std::time::Duration;
use serde::{Deserialize, Deserializer};

#[derive(Deserialize)]
#[serde(untagged)]
enum VoS<T> {
    Single(T),
    Multiple(Vec<T>),
}

pub fn vec_or_single<'de, T, D>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    VoS::<T>::deserialize(deserializer).map(|v| match v {
        VoS::Single(v) => vec![v],
        VoS::Multiple(v) => v,
    })
}


pub fn deser_duration<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let d = f64::deserialize(deserializer)?;
    let c = Duration::from_secs_f64(d);
    Ok(c)
}