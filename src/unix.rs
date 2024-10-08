use std::{
    io,
    path::PathBuf,
    process::{Child, Command, Stdio},
};

/// Unix-specific state required to run processes transparently.
#[derive(Clone, Debug, Default)]
pub struct TransparentRunnerImpl {
    pub id: Option<u32>,
    pub auth: Option<String>,
}

impl TransparentRunnerImpl {
    pub fn spawn_transparent(&self, command: &Command) -> io::Result<Child> {
        let mut runner_command = Command::new("xvfb-run");
        let c = if let Some(id) = self.id {
            runner_command.arg("--server-num").arg(format!("{id}"))
        } else {
            runner_command.arg("--auto-servernum")
        };
        let c = if let Some(auth) = &self.auth {
            c.arg("--auth-file").arg(auth)
        } else {
            c
        };
        c.arg(command.get_program())
            .args(command.get_args())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let id = self.id;

        for env in command.get_envs() {
            match env {
                (k, Some(v)) => runner_command.env(k, v),
                (k, None) => runner_command.env_remove(k),
            };
        }

        if let Some(cd) = command.get_current_dir() {
            runner_command.current_dir(cd);
        } else {
            runner_command.current_dir(std::env::current_dir()?);
        }

        runner_command.spawn()
    }
}
