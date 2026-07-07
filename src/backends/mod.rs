mod fcitx5;
mod fcitx5_rime;

pub use fcitx5::{Fcitx5Client, InputMethod};
pub use fcitx5_rime::{Fcitx5RimeClient, RimeMode};

use clap::ValueEnum;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Backend {
    #[value(name = "fcitx5")]
    Fcitx5,
    #[value(name = "fcitx5-rime")]
    Fcitx5Rime,
}
