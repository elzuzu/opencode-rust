# Ticket 1: Migrate CLI Parsing to Clap

**Task:** Migrate the command-line interface from `yargs` to `clap`.

**Description:** The current CLI is built using `yargs`. This ticket involves replacing the `yargs` implementation with `clap`, a popular and powerful command-line argument parser for Rust. The goal is to replicate the existing command structure and argument parsing logic.

**Acceptance Criteria:**
- All existing commands (`run`, `generate`, `auth`, etc.) are implemented in `clap`.
- All command-line options and flags are correctly parsed.
- The help and version messages are displayed correctly.
- The application logic is correctly invoked based on the parsed arguments.

**Files to Migrate:**
- `packages/opencode/src/index.ts`
- `packages/opencode/src/cli/cmd/*.ts`

**Suggested Rust Crates:**
- `clap` (with the `derive` feature)
- `tokio` (for the async runtime)

**Unit Test Examples:**

```rust
// in tests/cli.rs

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[derive(Parser, Debug)]
    #[clap(name = "opencode-rust")]
    struct Opts {
        #[clap(subcommand)]
        command: Command,
    }

    #[derive(Parser, Debug)]
    enum Command {
        Run {
            #[clap(short, long)]
            name: String,
        },
        Generate,
    }

    #[test]
    fn test_run_command() {
        let opts = Opts::parse_from(&["opencode-rust", "run", "--name", "test"]);
        match opts.command {
            Command::Run { name } => assert_eq!(name, "test"),
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
```
