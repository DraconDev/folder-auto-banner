//! Docker integration — detects containers and docker-compose for a project
//!
//! Checks for docker-compose.yml, Dockerfile, and running containers.
//! Timeout: 500ms, Cache: 10 seconds

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;
use std::time::Duration;

const DOCKER_TIMEOUT: Duration = Duration::from_millis(500);

/// Docker info for a project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerInfo {
    pub has_compose: bool,
    pub has_dockerfile: bool,
    pub containers: Vec<ContainerInfo>,
}

/// Running container info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerInfo {
    pub name: String,
    pub status: String,
}

/// Detect Docker info for a project
pub fn detect_docker(path: &Path) -> Result<DockerInfo> {
    let has_compose = path.join("docker-compose.yml").exists()
        || path.join("docker-compose.yaml").exists()
        || path.join("compose.yml").exists()
        || path.join("compose.yaml").exists();

    let has_dockerfile = path.join("Dockerfile").exists()
        || path.join("Dockerfile.dev").exists()
        || path.join("Dockerfile.prod").exists();

    let containers = if has_compose || has_dockerfile {
        detect_containers(path)?
    } else {
        Vec::new()
    };

    Ok(DockerInfo {
        has_compose,
        has_dockerfile,
        containers,
    })
}

fn detect_containers(path: &Path) -> Result<Vec<ContainerInfo>> {
    let dir_name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    // Try docker-compose project filter
    let output = run_with_timeout(
        "docker",
        &[
            "ps",
            "--filter",
            &format!("label=com.docker.compose.project={}", dir_name),
            "--format",
            "{{.Names}}:{{.Status}}",
        ],
        DOCKER_TIMEOUT,
    )?;

    let mut containers = Vec::new();
    for line in output.lines() {
        if let Some((name, status)) = line.split_once(':') {
            containers.push(ContainerInfo {
                name: name.to_string(),
                status: status.to_string(),
            });
        }
    }

    // If no compose containers found, try volume filter
    if containers.is_empty() {
        let abs_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        let output = run_with_timeout(
            "docker",
            &[
                "ps",
                "--filter",
                &format!("volume={}", abs_path.display()),
                "--format",
                "{{.Names}}:{{.Status}}",
            ],
            DOCKER_TIMEOUT,
        )?;

        for line in output.lines() {
            if let Some((name, status)) = line.split_once(':') {
                containers.push(ContainerInfo {
                    name: name.to_string(),
                    status: status.to_string(),
                });
            }
        }
    }

    Ok(containers)
}

fn run_with_timeout(cmd: &str, args: &[&str], timeout: Duration) -> Result<String> {
    let mut command = Command::new(cmd);
    command.args(args);
    command.stdout(std::process::Stdio::piped());
    command.stderr(std::process::Stdio::null());

    let start = std::time::Instant::now();
    let mut child = command.spawn()?;

    loop {
        if let Some(_output) = child.try_wait()? {
            let result = child.wait_with_output()?;
            return Ok(String::from_utf8_lossy(&result.stdout).to_string());
        }

        if start.elapsed() > timeout {
            let _ = child.kill();
            let _ = child.wait();
            return Ok(String::new());
        }

        std::thread::sleep(Duration::from_millis(10));
    }
}
