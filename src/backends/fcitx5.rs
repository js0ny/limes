use anyhow::{Context, Result};
use zbus::{proxy, Connection};

const SERVICE: &str = "org.fcitx.Fcitx5";
const PATH: &str = "/controller";
const INTERFACE: &str = "org.fcitx.Fcitx.Controller1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputMethod {
    pub unique_name: String,
    pub name: String,
    pub native_name: String,
    pub icon: String,
    pub label: String,
    pub language_code: String,
    pub enabled: bool,
}

type InputMethodTuple = (String, String, String, String, String, String, bool);

pub struct Fcitx5Client {
    conn: Connection,
}

impl Fcitx5Client {
    pub async fn new() -> Result<Self> {
        let conn = Connection::session()
            .await
            .context("could not connect to the D-Bus session bus")?;

        Ok(Self { conn })
    }

    pub async fn current_input_method(&self) -> Result<String> {
        self.proxy()
            .await?
            .current_input_method()
            .await
            .context("could not get current Fcitx5 input method")
    }

    pub async fn set_current_input_method(&self, name: &str) -> Result<()> {
        self.proxy()
            .await?
            .set_current_im(name)
            .await
            .with_context(|| format!("could not switch Fcitx5 input method to '{name}'"))
    }

    pub async fn available_input_methods(&self) -> Result<Vec<InputMethod>> {
        let methods = self
            .proxy()
            .await?
            .available_input_methods()
            .await
            .context("could not list Fcitx5 input methods")?;

        Ok(methods.into_iter().map(InputMethod::from).collect())
    }

    pub async fn toggle(&self) -> Result<()> {
        self.proxy()
            .await?
            .toggle()
            .await
            .context("could not toggle Fcitx5 input method")
    }

    async fn proxy(&self) -> Result<FcitxControllerProxy<'_>> {
        FcitxControllerProxy::builder(&self.conn)
            .destination(SERVICE)?
            .path(PATH)?
            .interface(INTERFACE)?
            .build()
            .await
            .context("could not connect to Fcitx5 on the session bus")
    }
}

impl From<InputMethodTuple> for InputMethod {
    fn from(value: InputMethodTuple) -> Self {
        let (unique_name, name, native_name, icon, label, language_code, enabled) = value;

        Self {
            unique_name,
            name,
            native_name,
            icon,
            label,
            language_code,
            enabled,
        }
    }
}

#[proxy]
trait FcitxController {
    #[zbus(name = "CurrentInputMethod")]
    async fn current_input_method(&self) -> zbus::Result<String>;

    #[zbus(name = "SetCurrentIM")]
    async fn set_current_im(&self, name: &str) -> zbus::Result<()>;

    #[zbus(name = "AvailableInputMethods")]
    async fn available_input_methods(&self) -> zbus::Result<Vec<InputMethodTuple>>;

    #[zbus(name = "Toggle")]
    async fn toggle(&self) -> zbus::Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_available_input_method_tuple() {
        assert_eq!(
            InputMethod::from((
                "rime".to_string(),
                "Rime".to_string(),
                "中州韻".to_string(),
                "rime-icon".to_string(),
                "ㄓ".to_string(),
                "zh".to_string(),
                true,
            )),
            InputMethod {
                unique_name: "rime".to_string(),
                name: "Rime".to_string(),
                native_name: "中州韻".to_string(),
                icon: "rime-icon".to_string(),
                label: "ㄓ".to_string(),
                language_code: "zh".to_string(),
                enabled: true,
            },
        );
    }
}
