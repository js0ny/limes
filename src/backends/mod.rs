mod fcitx5;
mod fcitx5_rime;
mod ibus;

pub use fcitx5::{Fcitx5Client, InputMethod};
pub use fcitx5_rime::{Fcitx5RimeClient, RimeMode};
pub use ibus::IBusClient;

use clap::ValueEnum;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Backend {
    #[value(name = "fcitx5")]
    Fcitx5,
    #[value(name = "fcitx5-rime")]
    Fcitx5Rime,
    #[value(name = "ibus")]
    Ibus,
}
