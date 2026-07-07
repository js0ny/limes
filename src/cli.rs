use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;

use crate::backends::{Backend, RimeMode};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Get,
    Set { name: String },
    List,
    Toggle,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Parsed {
    Run {
        command: Command,
        backend: Option<Backend>,
        mode: Option<RimeMode>,
    },
    Completion(Shell),
}

#[derive(Debug, Parser)]
#[command(
    name = "limes",
    about = "Linux input method switcher",
    disable_help_subcommand = true
)]
struct Cli {
    #[arg(long, global = true)]
    backend: Option<Backend>,

    #[arg(long, global = true)]
    mode: Option<RimeMode>,

    #[command(subcommand)]
    command: Option<TopCommand>,

    input_method: Option<String>,
}

#[derive(Debug, Subcommand)]
enum TopCommand {
    Get,
    Set { name: String },
    List,
    Toggle,
    Completion { shell: Shell },
}

pub fn parse_args(args: impl IntoIterator<Item = String>) -> Result<Parsed, clap::Error> {
    let cli = Cli::try_parse_from(std::iter::once("limes".to_string()).chain(args))?;

    let Cli {
        backend,
        mode,
        command,
        input_method,
    } = cli;

    let parsed = match (command, input_method) {
        (Some(TopCommand::Completion { shell }), _) => Parsed::Completion(shell),
        (Some(TopCommand::Get), None) | (None, None) => Parsed::Run {
            command: Command::Get,
            backend,
            mode,
        },
        (Some(TopCommand::Set { name }), None) | (None, Some(name)) => Parsed::Run {
            command: Command::Set { name },
            backend,
            mode,
        },
        (Some(TopCommand::List), None) => Parsed::Run {
            command: Command::List,
            backend,
            mode,
        },
        (Some(TopCommand::Toggle), None) => Parsed::Run {
            command: Command::Toggle,
            backend,
            mode,
        },
        _ => unreachable!("clap rejects extra positional arguments"),
    };

    Ok(parsed)
}

pub fn generate_completion(shell: Shell) {
    let mut cmd = Cli::command();
    let mut out = std::io::stdout();
    clap_complete::generate(shell, &mut cmd, "limes", &mut out);
}

#[cfg(test)]
mod tests {
    use clap::{error::ErrorKind, CommandFactory};

    use super::*;
    use crate::backends::{Backend, RimeMode};

    fn parse(args: &[&str]) -> Result<Parsed, clap::Error> {
        parse_args(args.iter().map(|arg| arg.to_string()))
    }

    fn assert_run(
        args: &[&str],
        expected_command: Command,
        expected_backend: Option<Backend>,
        expected_mode: Option<RimeMode>,
    ) {
        let parsed = parse(args).unwrap();
        match parsed {
            Parsed::Run {
                command,
                backend,
                mode,
            } => {
                assert_eq!(command, expected_command, "args: {args:?}");
                assert_eq!(backend, expected_backend, "args: {args:?}");
                assert_eq!(mode, expected_mode, "args: {args:?}");
            }
            other => panic!("args {args:?}: expected Run, got {other:?}"),
        }
    }

    fn assert_completion(args: &[&str], expected: Shell) {
        let parsed = parse(args).unwrap();
        match parsed {
            Parsed::Completion(shell) => assert_eq!(shell, expected, "args: {args:?}"),
            other => panic!("args {args:?}: expected Completion, got {other:?}"),
        }
    }

    fn assert_error_kind(args: &[&str], expected: ErrorKind) {
        let error = match parse(args) {
            Ok(parsed) => panic!("args should fail: {args:?}, parsed as {parsed:?}"),
            Err(error) => error,
        };

        assert_eq!(error.kind(), expected, "args: {args:?}");
    }

    #[test]
    fn parses_bare_commands_with_default_backend() {
        let cases = [
            (&[][..], Command::Get),
            (&["get"][..], Command::Get),
            (&["list"][..], Command::List),
            (&["toggle"][..], Command::Toggle),
            (
                &["set", "rime"][..],
                Command::Set {
                    name: "rime".to_string(),
                },
            ),
        ];

        for (args, command) in cases {
            assert_run(args, command, None, None);
        }
    }

    #[test]
    fn parses_im_select_shortcut_as_set() {
        for name in ["keyboard-us", "rime"] {
            assert_run(
                &[name],
                Command::Set {
                    name: name.to_string(),
                },
                None,
                None,
            );
        }
    }

    #[test]
    fn parses_backend_override() {
        assert_run(
            &["--backend", "fcitx5", "list"][..],
            Command::List,
            Some(Backend::Fcitx5),
            None,
        );
        assert_run(
            &["--backend", "fcitx5-rime", "list"][..],
            Command::List,
            Some(Backend::Fcitx5Rime),
            None,
        );
        assert_run(
            &["list", "--backend", "fcitx5-rime"][..],
            Command::List,
            Some(Backend::Fcitx5Rime),
            None,
        );
        assert_run(
            &["--backend", "ibus", "list"][..],
            Command::List,
            Some(Backend::Ibus),
            None,
        );
    }

    #[test]
    fn parses_mode_override() {
        assert_run(
            &["--backend", "fcitx5-rime", "--mode", "schema", "list"][..],
            Command::List,
            Some(Backend::Fcitx5Rime),
            Some(RimeMode::Schema),
        );
        assert_run(
            &["--backend", "fcitx5-rime", "--mode", "ascii", "toggle"][..],
            Command::Toggle,
            Some(Backend::Fcitx5Rime),
            Some(RimeMode::Ascii),
        );
    }

    #[test]
    fn mode_is_accepted_with_non_rime_backend_at_parse_time() {
        // --mode is global; cross-backend validation happens at run time, not parse time.
        assert_run(
            &["--backend", "fcitx5", "--mode", "schema", "list"][..],
            Command::List,
            Some(Backend::Fcitx5),
            Some(RimeMode::Schema),
        );
        assert_run(
            &["--backend", "ibus", "--mode", "ascii", "list"][..],
            Command::List,
            Some(Backend::Ibus),
            Some(RimeMode::Ascii),
        );
    }

    #[test]
    fn parses_completion_subcommand() {
        for (args, expected) in [
            (&["completion", "bash"][..], Shell::Bash),
            (&["completion", "zsh"][..], Shell::Zsh),
            (&["completion", "fish"][..], Shell::Fish),
            (&["completion", "elvish"][..], Shell::Elvish),
            (&["completion", "powershell"][..], Shell::PowerShell),
        ] {
            assert_completion(args, expected);
        }
    }

    #[test]
    fn completion_ignores_backend_and_mode_flags() {
        assert_completion(
            &["--backend", "fcitx5-rime", "completion", "bash"][..],
            Shell::Bash,
        );
    }

    #[test]
    fn rejects_unknown_backend_value() {
        assert_error_kind(&["--backend", "unknown", "list"][..], ErrorKind::InvalidValue);
    }

    #[test]
    fn rejects_unknown_mode_value() {
        assert_error_kind(&["--mode", "foo", "list"][..], ErrorKind::InvalidValue);
    }

    #[test]
    fn rejects_completion_without_shell() {
        assert_error_kind(&["completion"][..], ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn rejects_unknown_shell() {
        assert_error_kind(&["completion", "tcsh"][..], ErrorKind::InvalidValue);
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

    #[test]
    fn returns_display_help_for_help_flags() {
        for args in [&["--help"][..], &["-h"][..]] {
            assert_error_kind(args, ErrorKind::DisplayHelp);
        }
    }

    #[test]
    fn help_text_exposes_backend_and_mode_flags() {
        let help = Cli::command().render_long_help().to_string();

        assert!(
            help.contains("--backend <BACKEND>"),
            "help did not expose --backend:\n{help}",
        );
        assert!(
            help.contains("[possible values: fcitx5, fcitx5-rime, ibus]"),
            "help did not list backend possible values:\n{help}",
        );
        assert!(
            help.contains("--mode <MODE>"),
            "help did not expose --mode:\n{help}",
        );
        assert!(
            help.contains("[possible values: ascii, schema]"),
            "help did not list mode possible values:\n{help}",
        );
        assert!(
            help.contains("completion"),
            "help did not list completion subcommand:\n{help}",
        );
    }
}
