use serde::Serialize;

pub fn print_each<T: Serialize, I: IntoIterator<Item = T>>(items: I) -> crate::error::Result<()> {
    let mut out = std::io::stdout().lock();
    for item in items {
        serde_json::to_writer(&mut out, &item)?;
        std::io::Write::write_all(&mut out, b"\n")?;
    }
    Ok(())
}
