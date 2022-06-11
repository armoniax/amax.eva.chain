use serde::{de::Error, Deserialize, Deserializer};

/// deserialize u32 with 0x
pub fn deserialize_u32_0x<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let buf = String::deserialize(deserializer)?;

    let parsed = match buf.strip_prefix("0x") {
        Some(buf) => u32::from_str_radix(buf, 16),
        None => buf.parse::<u32>(),
    };

    parsed.map_err(|e| Error::custom(format!("parsing error: {:?} from '{}'", e, buf)))
}

/// deserialize u64 with 0x
pub fn deserialize_u64_0x<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    let buf = String::deserialize(deserializer)?;

    let parsed = match buf.strip_prefix("0x") {
        Some(buf) => u64::from_str_radix(buf, 16),
        None => buf.parse::<u64>(),
    };

    parsed.map_err(|e| Error::custom(format!("parsing error: {:?} from '{}'", e, buf)))
}
