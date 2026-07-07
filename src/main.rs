mod cli;
mod fcitx5;

use anyhow::Result;
use cli::Command;
use fcitx5::{Fcitx5Client, InputMethod};

#[tokio::main]
async fn main() {
    let command = match cli::parse_args(std::env::args().skip(1)) {
        Ok(command) => command,
        Err(error) => error.exit(),
    };

    if let Err(error) = run(command).await {
        eprintln!("error: {error:#}");
        std::process::exit(1);
    }
}

async fn run(command: Command) -> Result<()> {
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
