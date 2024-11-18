use crate::cli::commands::VcpkgArgs;

pub fn run(
    command: &str,
    args: &Vec<&str>,
    work_dir: &str,
    capture_stdout: bool,
    capture_stderr: bool,
    silent: bool,
) -> std::io::Result<std::process::Output> {
    let vcpkg_conf = VcpkgArgs::load_or_default();
    let envs = vcpkg_conf.get_envs();

    if !silent {
        tracing::info!(
            "command: {}, args: {}, cwd: {}, envs: {}",
            command,
            args.join(" "),
            work_dir,
            serde_json::to_string(&envs).unwrap()
        );
    }

    return std::process::Command::new(command)
        .args(args)
        .current_dir(work_dir)
        .envs(envs)
        .stdout(if capture_stdout {
            std::process::Stdio::piped()
        } else {
            std::process::Stdio::inherit()
        })
        .stderr(if capture_stderr {
            std::process::Stdio::piped()
        } else {
            std::process::Stdio::inherit()
        })
        .output();
}
