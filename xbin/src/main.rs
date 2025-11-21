use std::{
    fmt,
    process::{self, Stdio},
};

use anyhow::Result;
use clap::Parser;

trait RunnableCommand {
    fn run(&self) -> Result<()>;
}

#[derive(Parser)]
struct TestCommand {
    nargs: Vec<String>,

    #[arg(long)]
    hide_output: bool,

    #[arg(short, long)]
    all: bool,
}

impl RunnableCommand for TestCommand {
    fn run(&self) -> Result<()> {
        let mut p = process::Command::new("cargo");
        p.arg("test");
        if self.all {
            p.arg("--all");
        }
        p.args(&self.nargs);

        if self.hide_output {
            p.stdout(Stdio::null());
            p.stderr(Stdio::null());
        } else {
            p.stdout(Stdio::inherit());
            p.stderr(Stdio::inherit());
        }

        let status = p.status()?;

        if !status.success() {
            return Err(RunError.into());
        }
        Ok(())
    }
}

#[derive(Parser)]
struct BuildCommand {
    #[arg(short, long)]
    release: bool,

    nargs: Vec<String>,

    #[arg(long)]
    hide_output: bool,
}

impl RunnableCommand for BuildCommand {
    fn run(&self) -> Result<()> {
        let mut p = process::Command::new("uvx");
        p.args(&["maturin", "develop"]);
        if self.release {
            p.arg("--release");
        }
        p.args(&self.nargs);

        if self.hide_output {
            p.stdout(Stdio::null());
            p.stderr(Stdio::null());
        } else {
            p.stdout(Stdio::inherit());
            p.stderr(Stdio::inherit());
        }

        let status = p.status()?;

        if !status.success() {
            return Err(RunError.into());
        }
        Ok(())
    }
}

#[derive(Parser)]
struct RefreshEnvironmentCommand {
    #[arg(short, long)]
    release: bool,

    #[arg(long)]
    hide_output: bool,
}

impl RunnableCommand for RefreshEnvironmentCommand {
    fn run(&self) -> Result<()> {
        let cmd_runner = |program, args| -> Result<()> {
            let mut p = process::Command::new(program);
            p.args(args);

            if self.hide_output {
                p.stdout(Stdio::null());
                p.stderr(Stdio::null());
            } else {
                p.stdout(Stdio::inherit());
                p.stderr(Stdio::inherit());
            }

            let status = p.status()?;

            if !status.success() {
                return Err(RunError.into());
            }
            Ok(())
        };

        let args = {
            let mut v: Vec<String> =
                vec!["maturin".into(), "develop".into()];
            if self.release {
                v.push("--release".into());
            }
            v
        };
        cmd_runner("uvx", args.as_slice())?;

        cmd_runner("uv", &["cache".into(), "clean".into()])?;

        Ok(())
    }
}

#[derive(Parser)]
enum Command {
    Test(TestCommand),
    Build(BuildCommand),
    Refresh(RefreshEnvironmentCommand),
}

#[derive(Debug)]
struct RunError;

impl fmt::Display for RunError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Command failed to execute successfully.")
    }
}

impl std::error::Error for RunError {}

impl Command {
    fn run(self) -> Result<()> {
        match self {
            Command::Test(cmd) => cmd.run(),
            Command::Build(cmd) => cmd.run(),
            Command::Refresh(cmd) => cmd.run(),
        }
    }
}

fn main() -> Result<()> {
    let args = Command::parse();
    args.run()?;

    Ok(())
}
