use std::process::Stdio;
use std::env::current_dir;
use std::io::{Result, Error, ErrorKind};

use tokio::process::Command;

pub async fn run_cmd(cmd_name: &str, args: &[&str], inherit_output: bool, cwd: Option<String>) -> Result<()> {
    log::info!("running: {} {}", cmd_name, args.join(" "));

    let status = Command::new(cmd_name)
        .args(args)
        .stdin(Stdio::null())
        .stdout(match inherit_output {
            true => Stdio::inherit(),
            false => Stdio::null(),
        })
        .stderr(match inherit_output {
            true => Stdio::inherit(),
            false => Stdio::null(),
        })
        .current_dir(match cwd {
            Some(dirname) => dirname.clone().into(),
            None => current_dir().unwrap(),
        })
        .spawn()?
        .await?;

    if !status.success() {
        return Err(Error::new(
            ErrorKind::Other,
            format!("failed to run {}, status code: {}", cmd_name, status.code().unwrap()),
        ));
    }

    Ok(())
}
