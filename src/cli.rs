use clap::{Parser, Subcommand};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Get,
    Set { name: String },
    List,
    Toggle,
}

#[derive(Debug, Parser)]
#[command(
    name = "limes",
    about = "Linux input method switcher",
    disable_help_subcommand = true
)]
struct Cli {
    #[arg(long, default_value = "fcitx5", global = true)]
    backend: String,

    input_method: Option<String>,

    #[command(subcommand)]
    command: Option<NativeCommand>,
}

#[derive(Debug, Subcommand)]
enum NativeCommand {
    Get,
    Set { name: String },
    List,
    Toggle,
}

pub fn parse_args(args: impl IntoIterator<Item = String>) -> Result<Command, clap::Error> {
    let cli = Cli::try_parse_from(std::iter::once("limes".to_string()).chain(args))?;

    let Cli {
        command,
        input_method,
        backend: _,
    } = cli;

    Ok(match (command, input_method) {
        (Some(NativeCommand::Get), None) | (None, None) => Command::Get,
        (Some(NativeCommand::Set { name }), None) | (None, Some(name)) => Command::Set { name },
        (Some(NativeCommand::List), None) => Command::List,
        (Some(NativeCommand::Toggle), None) => Command::Toggle,
        _ => unreachable!("clap rejects extra positional arguments"),
    })
}

#[cfg(test)]
mod tests {
    use clap::{error::ErrorKind, CommandFactory};

    use super::*;

    fn parse(args: &[&str]) -> Result<Command, clap::Error> {
        parse_args(args.iter().map(|arg| arg.to_string()))
    }

    fn assert_parses(args: &[&str], expected: Command) {
        assert_eq!(parse(args).unwrap(), expected, "args: {args:?}");
    }

    fn assert_error_kind(args: &[&str], expected: ErrorKind) {
        let error = match parse(args) {
            Ok(command) => panic!("args should fail: {args:?}, parsed as {command:?}"),
            Err(error) => error,
        };

        assert_eq!(error.kind(), expected, "args: {args:?}");
    }

    #[test]
    fn parses_im_select_get_forms() {
        for args in [&[][..], &["get"][..]] {
            assert_parses(args, Command::Get);
        }
    }

    #[test]
    fn parses_native_subcommands() {
        let cases = [
            (&["list"][..], Command::List),
            (&["toggle"][..], Command::Toggle),
            (
                &["set", "rime"][..],
                Command::Set {
                    name: "rime".to_string(),
                },
            ),
        ];

        for (args, expected) in cases {
            assert_parses(args, expected);
        }
    }

    #[test]
    fn parses_single_unknown_argument_as_im_select_set() {
        for name in ["keyboard-us", "rime"] {
            assert_parses(
                &[name],
                Command::Set {
                    name: name.to_string(),
                },
            );
        }
    }

    #[test]
    fn backend_flag_is_accepted_without_changing_command_parse() {
        let cases = [
            (
                &["list"][..],
                &["list", "--backend", "fcitx5"][..],
                Command::List,
            ),
            (
                &["keyboard-us"][..],
                &["--backend", "fcitx5", "keyboard-us"][..],
                Command::Set {
                    name: "keyboard-us".to_string(),
                },
            ),
        ];

        for (without_backend, with_backend, expected) in cases {
            assert_parses(without_backend, expected.clone());
            assert_parses(with_backend, expected.clone());
            assert_eq!(
                parse(with_backend).unwrap(),
                parse(without_backend).unwrap(),
                "backend flag changed parse result for args: {with_backend:?}",
            );
        }
    }

    #[test]
    fn help_text_exposes_backend_flag_and_default() {
        let help = Cli::command().render_long_help().to_string();

        assert!(
            help.contains("--backend <BACKEND>"),
            "help did not expose --backend option:\n{help}",
        );
        assert!(
            help.contains("[default: fcitx5]"),
            "help did not show fcitx5 backend default:\n{help}",
        );
    }

    #[test]
    fn returns_display_help_for_help_flags() {
        for args in [&["--help"][..], &["-h"][..]] {
            assert_error_kind(args, ErrorKind::DisplayHelp);
        }
    }

    #[test]
    fn rejects_invalid_arity_with_clap_errors() {
        let cases = [
            (&["set"][..], ErrorKind::MissingRequiredArgument),
            (&["set", "rime", "extra"][..], ErrorKind::UnknownArgument),
            (&["get", "rime"][..], ErrorKind::UnknownArgument),
            (&["list", "rime"][..], ErrorKind::UnknownArgument),
            (&["toggle", "rime"][..], ErrorKind::UnknownArgument),
            (&["rime", "keyboard-us"][..], ErrorKind::UnknownArgument),
        ];

        for (args, expected) in cases {
            assert_error_kind(args, expected);
        }
    }
}
