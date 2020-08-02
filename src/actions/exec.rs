use std::env;
use std::path::Path;
use std::process::Stdio;
use std::env::current_dir;
use std::io::{Result, Error, ErrorKind};

use tokio::process::Command;

use crate::config::scripts::Script;

pub struct CmdOpts<'a> {
    pub name: &'a str,
    pub privileged: bool,
    pub args: &'a [&'a str],
    pub inherit_output: bool,
    pub cwd: Option<&'a Path>,
}

impl<'a> Default for CmdOpts<'a> {
    fn default() -> Self {
        Self {
            name: "",
            privileged: true,
            args: &[],
            inherit_output: false,
            cwd: None,
        }
    }
}

pub async fn run_cmd(opts: CmdOpts<'_>) -> Result<()> {
    log::info!("running: {} {}", opts.name, opts.args.join(" "));

    let status = Command::new(opts.name)
        .uid(match opts.privileged {
            true => 0,
            false => env::var("SUDO_UID")
                .unwrap()
                .parse()
                .unwrap(),
        })
        .args(opts.args)
        .stdin(Stdio::null())
        .stdout(match opts.inherit_output {
            true => Stdio::inherit(),
            false => Stdio::null(),
        })
        .stderr(match opts.inherit_output {
            true => Stdio::inherit(),
            false => Stdio::null(),
        })
        .current_dir(match opts.cwd {
            Some(dirname) => dirname.clone().into(),
            None => current_dir().unwrap(),
        })
        .spawn()?
        .await?;

    if !status.success() {
        return Err(Error::new(
            ErrorKind::Other,
            format!("failed to run {}, status code: {}", opts.name, status.code().unwrap()),
        ));
    }

    Ok(())
}

pub async fn run_script(script: &Script) -> Result<()> {
    if let Ok(_) = run_cmd(CmdOpts {
        name: "bash",
        args: &["-c", &script.test],
        ..CmdOpts::default()
    }).await {
        println!("no changes found for script: {}", script.name);
        return Ok(());
    }

    println!("running script: {}", script.name);

    run_cmd(CmdOpts {
        name: "bash",
        args: &["-c", &script.cmd],
        ..CmdOpts::default()
    }).await
}
