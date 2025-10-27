
#[cfg(test)]
mod tests {
    use opencode_rust::cli::{Command, Opts};
    use clap::Parser;

    #[test]
    fn test_run_command() {
        let opts = Opts::parse_from(&["opencode-rust", "run", "hello", "world", "--model", "test-model"]);
        match opts.command {
            Command::Run(run_cmd) => {
                assert_eq!(run_cmd.message, vec!["hello", "world"]);
                assert_eq!(run_cmd.model.unwrap(), "test-model");
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
}
