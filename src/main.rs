mod backends;
mod cli;

use crate::backends::{Backend, Fcitx5Client, Fcitx5RimeClient, IBusClient, InputMethod, RimeMode};
use anyhow::Result;
use cli::Command;

#[tokio::main]
async fn main() {
    let parsed = match cli::parse_args(std::env::args().skip(1)) {
        Ok(parsed) => parsed,
        Err(error) => error.exit(),
    };

    match parsed {
        cli::Parsed::Completion(shell) => cli::generate_completion(shell),
        cli::Parsed::Run {
            command,
            backend,
            mode,
        } => {
            let backend = backend.unwrap_or(Backend::Fcitx5);
            if let Err(error) = run(command, backend, mode).await {
                eprintln!("error: {error:#}");
                std::process::exit(1);
            }
        }
    }
}

async fn run(command: Command, backend: Backend, mode: Option<RimeMode>) -> Result<()> {
    if mode.is_some() && backend != Backend::Fcitx5Rime {
        anyhow::bail!("--mode is only supported with --backend fcitx5-rime");
    }

    match backend {
        Backend::Fcitx5 => {
            let client = Fcitx5Client::new().await?;
            match command {
                Command::Get => {
                    println!("{}", client.current_input_method().await?);
                }
                Command::Set { name } => {
                    client.set_current_input_method(&name).await?;
                }
                Command::List => {
                    for method in client.available_input_methods().await? {
                        println!("{}", format_input_method(&method));
                    }
                }
                Command::Toggle => {
                    client.toggle().await?;
                }
            }
        }
        Backend::Fcitx5Rime => {
            let client = Fcitx5RimeClient::new(mode.unwrap_or(RimeMode::Ascii)).await?;
            match command {
                Command::Get => {
                    println!("{}", client.get().await?);
                }
                Command::Set { name } => {
                    client.set(&name).await?;
                }
                Command::List => {
                    for name in client.list().await? {
                        println!("{name}");
                    }
                }
                Command::Toggle => {
                    client.toggle().await?;
                }
            }
        }
        Backend::Ibus => {
            match command {
                Command::Get => {
                    let client = IBusClient::new().await?;
                    println!("{}", client.get().await?);
                }
                Command::Set { name } => {
                    let client = IBusClient::new().await?;
                    client.set(&name).await?;
                }
                Command::List => {
                    let client = IBusClient::new().await?;
                    for name in client.list().await? {
                        println!("{name}");
                    }
                }
                Command::Toggle => {
                    IBusClient::toggle()?;
                }
            }
        }
    }

    Ok(())
}

fn format_input_method(method: &InputMethod) -> String {
    format!("{}\t{}", method.unique_name, method.name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_input_method_for_list_output() {
        let method = InputMethod {
            unique_name: "keyboard-us".to_string(),
            name: "Keyboard - English (US)".to_string(),
            native_name: "Keyboard US native name should not be printed".to_string(),
            icon: "keyboard-icon".to_string(),
            label: "EN".to_string(),
            language_code: "en".to_string(),
            enabled: true,
        };

        assert_eq!(
            format_input_method(&method),
            "keyboard-us\tKeyboard - English (US)",
        );
    }
}
