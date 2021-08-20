use portpicker::pick_unused_port;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::{thread, time::Duration};

use super::context;
use super::RuntimeFlavor;

fn local_rpc_addr(port: u16) -> String {
    format!("0.0.0.0:{}", port)
}

fn home_dir(port: u16) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!("sandbox-{}", port));
    path
}
pub struct SandboxServer {
    pub(self) port: u16,
    process: Option<Child>,
}

impl SandboxServer {
    pub fn new(port: u16) -> Self {
        Self {
            port,
            process: None,
        }
    }

    pub fn new_default() -> Self {
        let port = pick_unused_port().expect("no ports free");
        Self::new(port)
    }

    pub fn start(&mut self) -> io::Result<()> {
        println!("Starting up sandbox at localhost:{}", self.port);
        let home_dir = home_dir(self.port);

        // Remove dir if it already exists:
        let _ = fs::remove_dir_all(&home_dir);

        init_sandbox(&home_dir)?.wait()?;

        let child = start_sandbox(&home_dir, self.port)?;
        println!("Started sandbox: pid={:?}", child.id());
        self.process = Some(child);

        // TODO: Get rid of this sleep, and ping sandbox is alive instead:
        thread::sleep(Duration::from_secs(3));
        Ok(())
    }
}

impl Drop for SandboxServer {
    fn drop(&mut self) {
        if self.process.is_none() {
            return;
        }

        let child = self.process.as_mut().unwrap();

        println!(
            "Cleaning up sandbox: port={}, pid={}",
            self.port,
            child.id()
        );

        child
            .kill()
            .map_err(|e| format!("Could not cleanup sandbox due to: {:?}", e))
            .unwrap();
    }
}

#[cfg(target_os = "windows")]
fn start_sandbox(home_dir: &Path, port: u16) -> io::Result<Child> {
    Command::new("near-sandbox")
        .args(&[
            "--home",
            home_dir,
            "run",
            "--rpc-addr",
            &local_rpc_addr(port),
        ])
        .spawn()
}

#[cfg(target_os = "windows")]
fn init_sandbox(home_dir: &Path) -> io::Result<Child> {
    Command::new("near-sandbox")
        .args(&["--home", home_dir, "init"])
        .spawn()
}

#[cfg(not(target_os = "windows"))]
fn start_sandbox(home_dir: &Path, port: u16) -> io::Result<Child> {
    Command::new(
        "/usr/local/lib/node_modules/near-sandbox/node_modules/binary-install/bin/near-sandbox",
    )
    .arg("--home")
    .arg(home_dir)
    .arg("run")
    .arg("--rpc-addr")
    .arg(&local_rpc_addr(port))
    .spawn()
}

#[cfg(not(target_os = "windows"))]
fn init_sandbox(home_dir: &Path) -> io::Result<Child> {
    Command::new(
        "/usr/local/lib/node_modules/near-sandbox/node_modules/binary-install/bin/near-sandbox",
    )
    .arg("--home")
    .arg(home_dir)
    .arg("init")
    .spawn()
}

pub struct SandboxRuntime {
    server: SandboxServer,
    _guard: context::EnterGuard,
}

impl SandboxRuntime {
    pub fn new_default() -> Self {
        let server = SandboxServer::new_default();
        let port = server.port;

        Self {
            server,
            _guard: context::enter(RuntimeFlavor::Sandbox(port)),
        }
    }

    pub fn port(&self) -> u16 {
        self.server.port
    }

    pub fn rpc_addr(&self) -> String {
        format!("http://localhost:{}", self.port())
    }

    pub fn run(&mut self) -> io::Result<()> {
        self.server.start()
    }
}
