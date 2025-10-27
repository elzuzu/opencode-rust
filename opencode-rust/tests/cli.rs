#[cfg(test)]
mod tests {
    use clap::Parser;
    use opencode_rust::cli::{Command, Opts, cmd};

    #[test]
    fn test_run_command() {
        let opts = Opts::parse_from(&[
            "opencode-rust",
            "run",
            "hello",
            "world",
            "--model",
            "test-model",
        ]);
        match opts.command {
            Command::Run(run_cmd) => {
                assert_eq!(run_cmd.message, vec!["hello", "world"]);
                assert_eq!(run_cmd.model.unwrap(), "test-model");
                assert_eq!(run_cmd.format, cmd::run::OutputFormat::Default);
            }
            _ => panic!("Expected Run command"),
        }
    }

    #[test]
    fn test_generate_command() {
        let opts = Opts::parse_from(&["opencode-rust", "generate"]);
        match opts.command {
            Command::Generate => (),
            _ => panic!("Expected Generate command"),
        }
    }

    #[test]
    fn test_auth_login_command() {
        let opts = Opts::parse_from(&["opencode-rust", "auth", "login", "https://example.com"]);
        match opts.command {
            Command::Auth(auth_cmd) => match auth_cmd.action {
                cmd::auth::AuthAction::Login(login) => {
                    assert_eq!(login.url.as_deref(), Some("https://example.com"));
                }
                _ => panic!("Expected auth login"),
            },
            _ => panic!("Expected Auth command"),
        }
    }

    #[test]
    fn test_upgrade_method() {
        let opts = Opts::parse_from(&["opencode-rust", "upgrade", "1.2.3", "--method", "bun"]);
        match opts.command {
            Command::Upgrade(upgrade_cmd) => {
                assert_eq!(upgrade_cmd.target.as_deref(), Some("1.2.3"));
                assert!(matches!(
                    upgrade_cmd.method,
                    Some(cmd::upgrade::UpgradeMethod::Bun)
                ));
            }
            _ => panic!("Expected Upgrade command"),
        }
    }
}
