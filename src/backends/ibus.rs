use anyhow::{bail, Context, Result};
use zbus::{
    proxy,
    zvariant::{OwnedValue, Value},
    Connection,
};

const IBUS_DEST: &str = "org.freedesktop.IBus";
const IBUS_PATH: &str = "/org/freedesktop/IBus";
const IBUS_IFACE: &str = "org.freedesktop.IBus";
pub struct IBusClient {
    conn: Connection,
}

impl IBusClient {
    pub async fn new() -> Result<Self> {
        let conn = zbus::connection::Builder::ibus()
            .context("could not resolve IBus private D-Bus address")?
            .build()
            .await
            .context("could not connect to IBus; is ibus-daemon running?")?;

        Ok(Self { conn })
    }

    pub async fn get(&self) -> Result<String> {
        let value = self
            .proxy()
            .await?
            .get_global_engine()
            .await
            .context("could not get current IBus engine")?;

        parse_engine_desc_name(&value)
    }

    pub async fn set(&self, name: &str) -> Result<()> {
        self.proxy()
            .await?
            .set_global_engine(name)
            .await
            .with_context(|| format!("could not switch IBus global engine to '{name}'"))
    }

    pub async fn list(&self) -> Result<Vec<String>> {
        let values = self
            .proxy()
            .await?
            .list_engines()
            .await
            .context("could not list IBus engines")?;

        values.iter().map(parse_engine_desc_name).collect()
    }

    pub fn toggle() -> Result<()> {
        bail!("IBus backend does not support toggle")
    }

    async fn proxy(&self) -> Result<IBusProxy<'_>> {
        IBusProxy::builder(&self.conn)
            .destination(IBUS_DEST)?
            .path(IBUS_PATH)?
            .interface(IBUS_IFACE)?
            .build()
            .await
            .context("could not connect to IBus on its private D-Bus address")
    }
}

pub fn parse_engine_desc_name(value: &OwnedValue) -> Result<String> {
    parse_engine_desc_name_from_value(value)
}

fn parse_engine_desc_name_from_value(mut value: &Value<'_>) -> Result<String> {
    while let Value::Value(inner) = value {
        value = inner;
    }

    let Value::Structure(desc) = value else {
        bail!("IBus engine description is not a struct")
    };

    let name = desc
        .fields()
        .get(2)
        .context("IBus engine description is missing engine id field")?;

    String::try_from(name).context("IBus engine id field is not a string")
}

#[proxy]
trait IBus {
    #[zbus(name = "GetGlobalEngine")]
    async fn get_global_engine(&self) -> zbus::Result<OwnedValue>;

    #[zbus(name = "ListEngines")]
    async fn list_engines(&self) -> zbus::Result<Vec<OwnedValue>>;

    #[zbus(name = "SetGlobalEngine")]
    async fn set_global_engine(&self, name: &str) -> zbus::Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use zbus::zvariant::{Structure, Value};

    #[test]
    fn parses_engine_name_from_engine_desc_variant() {
        let desc = Structure::from(("IBusEngineDesc", "metadata", "libpinyin"));
        let value = OwnedValue::try_from(Value::Value(Box::new(Value::Structure(desc)))).unwrap();

        assert_eq!(parse_engine_desc_name(&value).unwrap(), "libpinyin");
    }

    #[test]
    fn rejects_engine_desc_without_name_field() {
        let desc = Structure::from(("IBusEngineDesc", "metadata"));
        let value = OwnedValue::try_from(Value::Structure(desc)).unwrap();

        assert!(parse_engine_desc_name(&value).is_err());
    }
}
