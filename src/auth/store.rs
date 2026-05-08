use keyring::Entry;

use crate::error::{Error, Result};

const SERVICE: &str = "sh.keygen.cli";

pub fn save_token(profile: &str, token: &str) -> Result<()> {
    Entry::new(SERVICE, profile)
        .map_err(|e| Error::auth(format!("keyring init: {e}")))?
        .set_password(token)
        .map_err(|e| Error::auth(format!("keyring write: {e}")))?;
    Ok(())
}

pub fn load_token(profile: &str) -> Result<Option<String>> {
    let entry =
        Entry::new(SERVICE, profile).map_err(|e| Error::auth(format!("keyring init: {e}")))?;
    match entry.get_password() {
        Ok(t) => Ok(Some(t)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(Error::auth(format!("keyring read: {e}"))),
    }
}

pub fn delete_token(profile: &str) -> Result<()> {
    let entry =
        Entry::new(SERVICE, profile).map_err(|e| Error::auth(format!("keyring init: {e}")))?;
    match entry.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(Error::auth(format!("keyring delete: {e}"))),
    }
}
