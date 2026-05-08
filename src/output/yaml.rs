use serde::Serialize;

pub fn print<T: Serialize>(value: &T) -> crate::error::Result<()> {
    let s = serde_yaml_ng::to_string(value).map_err(|e| crate::Error::Serde(e.to_string()))?;
    print!("{s}");
    Ok(())
}
