use std::process;

use anyhow::Result;
use structopt::StructOpt;

trait RunnableCommand {
    fn command(&self) -> Vec<process::Command>;
}

#[derive(StructOpt)]
struct TestCommand {
    nargs: Vec<String>,
}

impl RunnableCommand for TestCommand {
    fn command(&self) -> Vec<process::Command> {
        let mut p = process::Command::new("cargo");
        p.args(&["test", "--all"]);
        p.args(&self.nargs);
        vec![p]
    }
}

#[derive(StructOpt)]
struct BuildCommand {
    #[structopt(short, long)]
    release: bool,

    nargs: Vec<String>,
}

impl RunnableCommand for BuildCommand {
    fn command(&self) -> Vec<process::Command> {
        let mut p = process::Command::new("uvx");
        p.args(&["maturin", "develop"]);
        if self.release {
            p.arg("--release");
        }
        p.args(&self.nargs);

        vec![p]
    }
}

#[derive(StructOpt)]
struct PackageCommand {
    // #[structopt(subcommand)]
    #[structopt(flatten)]
    build: BuildCommand,

    #[structopt(long)]
    entry: String,
}

impl RunnableCommand for PackageCommand {
    fn command(&self) -> Vec<process::Command> {
        let mut cmds = self.build.command();

        let mut args: Vec<_> =
            ["pyinstaller", "--additional-hooks-dir=hooks"]
                .into_iter()
                .map(String::from)
                .collect();
        args.push(self.entry.clone());

        let mut p = process::Command::new("uvx");
        p.args(&["pyinstaller", "--additional-hooks-dir", "hooks"]);
        p.arg(&self.entry);

        cmds.push(p);

        cmds
    }
}

#[derive(StructOpt)]
enum CommandInner {
    Test(TestCommand),

    Build(BuildCommand),

    Package(PackageCommand),
}

#[derive(StructOpt)]
struct Command {
    #[structopt(flatten)]
    inner: CommandInner,

    #[structopt(long)]
    show_output: bool,
}

impl Command {
    fn get_commands(&self) -> Vec<process::Command> {
        match &self.inner {
            CommandInner::Test(test) => test.command(),
            CommandInner::Build(build) => build.command(),
            CommandInner::Package(package) => package.command(),
        }
    }

    fn run(self) -> Result<()> {
        let cmds = self.get_commands();
        for mut cmd in cmds.into_iter() {
            let output = cmd.output()?;
            if self.show_output {
                let s = |v: Vec<u8>| {
                    let len = v.len();
                    let cap = v.capacity();
                    let ptr = v.leak();

                    unsafe {
                        String::from_raw_parts(ptr.as_mut_ptr(), len, cap)
                    }
                };

                println!("{}", s(output.stdout));
                eprintln!("{}", s(output.stderr));
            }
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    let args = Command::from_args();
    args.run()?;

    Ok(())
}
