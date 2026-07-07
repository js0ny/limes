use anyhow::{bail, Context, Result};
use clap::ValueEnum;
use zbus::{proxy, Connection};

const SERVICE: &str = "org.fcitx.Fcitx5";
const PATH: &str = "/rime";
const INTERFACE: &str = "org.fcitx.Fcitx.Rime1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum RimeMode {
    #[value(name = "ascii")]
    Ascii,
    #[value(name = "schema")]
    Schema,
}

pub struct Fcitx5RimeClient {
    conn: Connection,
    mode: RimeMode,
}

impl Fcitx5RimeClient {
    pub async fn new(mode: RimeMode) -> Result<Self> {
        let conn = Connection::session()
            .await
            .context("could not connect to the D-Bus session bus")?;

        Ok(Self { conn, mode })
    }

    pub async fn get(&self) -> Result<String> {
        let proxy = self.proxy().await?;
        match self.mode {
            RimeMode::Ascii => {
                let on = proxy
                    .is_ascii_mode()
                    .await
                    .context("could not read rime ascii mode")?;
                Ok(if on { "true" } else { "false" }.to_string())
            }
            RimeMode::Schema => proxy
                .current_schema()
                .await
                .context("could not get current rime schema"),
        }
    }

    pub async fn set(&self, name: &str) -> Result<()> {
        let proxy = self.proxy().await?;
        match self.mode {
            RimeMode::Ascii => {
                let on = match name {
                    "true" => true,
                    "false" => false,
                    other => bail!(
                        "invalid ascii state '{other}', expected 'true' or 'false'"
                    ),
                };
                proxy
                    .set_ascii_mode(on)
                    .await
                    .context("could not set rime ascii mode")
            }
            RimeMode::Schema => proxy
                .set_schema(name)
                .await
                .with_context(|| format!("could not set rime schema to '{name}'")),
        }
    }

    pub async fn list(&self) -> Result<Vec<String>> {
        match self.mode {
            RimeMode::Ascii => bail!("list is not supported in ascii mode; use `--mode schema`"),
            RimeMode::Schema => self
                .proxy()
                .await?
                .list_all_schemas()
                .await
                .context("could not list rime schemas"),
        }
    }

    pub async fn toggle(&self) -> Result<()> {
        let proxy = self.proxy().await?;
        match self.mode {
            RimeMode::Ascii => {
                let current = proxy
                    .is_ascii_mode()
                    .await
                    .context("could not read rime ascii mode")?;
                proxy
                    .set_ascii_mode(!current)
                    .await
                    .context("could not toggle rime ascii mode")
            }
            RimeMode::Schema => bail!("toggle is not supported for rime schemas; use `set <schema>`"),
        }
    }

    async fn proxy(&self) -> Result<RimeProxy<'_>> {
        RimeProxy::builder(&self.conn)
            .destination(SERVICE)?
            .path(PATH)?
            .interface(INTERFACE)?
            .build()
            .await
            .context("could not connect to Fcitx5 Rime on the session bus")
    }
}

#[proxy]
trait Rime {
    #[zbus(name = "GetCurrentSchema")]
    async fn current_schema(&self) -> zbus::Result<String>;

    #[zbus(name = "IsAsciiMode")]
    async fn is_ascii_mode(&self) -> zbus::Result<bool>;

    #[zbus(name = "ListAllSchemas")]
    async fn list_all_schemas(&self) -> zbus::Result<Vec<String>>;

    #[zbus(name = "SetAsciiMode")]
    async fn set_ascii_mode(&self, mode: bool) -> zbus::Result<()>;

    #[zbus(name = "SetSchema")]
    async fn set_schema(&self, name: &str) -> zbus::Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rime_mode_value_names_are_ascii_and_schema() {
        assert_eq!(RimeMode::Ascii, RimeMode::Ascii);
        assert_eq!(RimeMode::Schema, RimeMode::Schema);
        assert_ne!(RimeMode::Ascii, RimeMode::Schema);
    }
}
