use serde::Serialize;
use serde_json::Value;
use std::process::Command;
use thiserror::Error;

#[derive(Debug, Error)]
enum DockerError {
    #[error("{0}")]
    Message(String),
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ContainerSummary {
    id: String,
    name: String,
    image: String,
    command: String,
    created_at: String,
    running_for: String,
    status: String,
    state: String,
    ports: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct KeyValuePair {
    key: String,
    value: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ContainerDetails {
    id: String,
    name: String,
    image: String,
    command: String,
    created: String,
    status: String,
    state: String,
    running: bool,
    paused: bool,
    restarting: bool,
    dead: bool,
    pid: u64,
    exit_code: i64,
    started_at: Option<String>,
    finished_at: Option<String>,
    health: Option<String>,
    network_mode: Option<String>,
    ip_address: Option<String>,
    labels: Vec<KeyValuePair>,
    environment: Vec<String>,
    mounts: Vec<String>,
    ports: Vec<String>,
}

fn docker_error(message: impl Into<String>) -> DockerError {
    DockerError::Message(message.into())
}

fn run_docker(args: &[&str]) -> Result<String, DockerError> {
    let output = Command::new("docker")
        .args(args)
        .output()
        .map_err(|error| docker_error(format!("Failed to execute docker: {error}")))?;

    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).trim().to_string());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    Err(docker_error(if stderr.is_empty() {
        "docker returned a non-zero exit status".to_string()
    } else {
        stderr
    }))
}

fn state_from_status(status: &str) -> String {
    let lowered = status.trim().to_lowercase();

    if lowered.starts_with("up ") {
        "running".to_string()
    } else if lowered.starts_with("paused") {
        "paused".to_string()
    } else if lowered.starts_with("restarting") {
        "restarting".to_string()
    } else if lowered.starts_with("exited") {
        "stopped".to_string()
    } else if lowered.starts_with("created") {
        "created".to_string()
    } else if lowered.starts_with("dead") {
        "dead".to_string()
    } else {
        "unknown".to_string()
    }
}

fn first_name(names: &str) -> String {
    names
        .split(',')
        .find_map(|name| {
            let trimmed = name.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.trim_start_matches('/').to_string())
            }
        })
        .unwrap_or_else(|| names.trim().to_string())
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
struct DockerPsRow {
    #[serde(alias = "ID")]
    id: String,
    image: String,
    command: String,
    created_at: String,
    running_for: String,
    status: String,
    ports: String,
    names: String,
}

fn parse_summary_line(line: &str) -> Result<ContainerSummary, DockerError> {
    let row: DockerPsRow = serde_json::from_str(line)
        .map_err(|error| docker_error(format!("Failed to parse docker ps output: {error}")))?;

    Ok(ContainerSummary {
        id: row.id,
        name: first_name(&row.names),
        image: row.image,
        command: row.command,
        created_at: row.created_at,
        running_for: row.running_for,
        status: row.status.clone(),
        state: state_from_status(&row.status),
        ports: row.ports,
    })
}

fn json_string(value: &Value, key: &str) -> Option<String> {
    value.get(key)?.as_str().map(ToOwned::to_owned)
}

fn nested_string<'a>(value: &'a Value, path: &[&str]) -> Option<String> {
    let mut current = value;

    for key in path {
        current = current.get(*key)?;
    }

    current.as_str().map(ToOwned::to_owned)
}

fn json_u64(value: &Value, key: &str) -> u64 {
    value.get(key).and_then(Value::as_u64).unwrap_or_default()
}

fn json_i64(value: &Value, key: &str) -> i64 {
    value.get(key).and_then(Value::as_i64).unwrap_or_default()
}

fn json_bool(value: &Value, key: &str) -> bool {
    value.get(key).and_then(Value::as_bool).unwrap_or(false)
}

fn json_labels(value: &Value) -> Vec<KeyValuePair> {
    let labels = value
        .get("Config")
        .and_then(|config| config.get("Labels"))
        .and_then(Value::as_object);

    let Some(labels) = labels else {
        return Vec::new();
    };

    labels
        .iter()
        .map(|(key, value)| KeyValuePair {
            key: key.clone(),
            value: value.as_str().unwrap_or_default().to_string(),
        })
        .collect()
}

fn json_environment(value: &Value) -> Vec<String> {
    value
        .get("Config")
        .and_then(|config| config.get("Env"))
        .and_then(Value::as_array)
        .map(|entries| {
            entries
                .iter()
                .filter_map(Value::as_str)
                .map(ToOwned::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn json_mounts(value: &Value) -> Vec<String> {
    value
        .get("Mounts")
        .and_then(Value::as_array)
        .map(|mounts| {
            mounts
                .iter()
                .filter_map(|mount| {
                    let source = mount.get("Source").and_then(Value::as_str)?;
                    let destination = mount.get("Destination").and_then(Value::as_str)?;
                    let mode = mount.get("Mode").and_then(Value::as_str).unwrap_or("rw");
                    let rw = mount.get("RW").and_then(Value::as_bool).unwrap_or(true);
                    let access = if rw { "rw" } else { "ro" };
                    Some(format!("{source} -> {destination} ({mode}, {access})"))
                })
                .collect()
        })
        .unwrap_or_default()
}

fn json_ports(value: &Value) -> Vec<String> {
    let Some(ports) = value
        .get("NetworkSettings")
        .and_then(|network| network.get("Ports"))
        .and_then(Value::as_object)
    else {
        return Vec::new();
    };

    let mut entries = Vec::new();

    for (port, bindings) in ports {
        let mut rendered = Vec::new();

        if let Some(array) = bindings.as_array() {
            for binding in array {
                let host_ip = binding
                    .get("HostIp")
                    .and_then(Value::as_str)
                    .filter(|ip| !ip.is_empty())
                    .unwrap_or("0.0.0.0");
                let host_port = binding
                    .get("HostPort")
                    .and_then(Value::as_str)
                    .filter(|port| !port.is_empty())
                    .unwrap_or("-");
                rendered.push(format!("{host_ip}:{host_port} -> {port}"));
            }
        }

        if rendered.is_empty() {
            entries.push(format!("{port} (unpublished)"));
        } else {
            entries.extend(rendered);
        }
    }

    entries
}

fn inspect_container_inner(id: &str) -> Result<ContainerDetails, DockerError> {
    let raw = run_docker(&["inspect", "--type", "container", id])?;
    let mut parsed: Vec<Value> = serde_json::from_str(&raw)
        .map_err(|error| docker_error(format!("Failed to parse docker inspect output: {error}")))?;
    let Some(item) = parsed.pop() else {
        return Err(docker_error("docker inspect returned no container details"));
    };

    let state = item.get("State").cloned().unwrap_or(Value::Null);
    let config = item.get("Config").cloned().unwrap_or(Value::Null);
    let path = item
        .get("Path")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let args = item
        .get("Args")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let command = if args.is_empty() {
        path.clone()
    } else {
        format!("{path} {}", args.join(" "))
    };

    Ok(ContainerDetails {
        id: json_string(&item, "Id").unwrap_or_else(|| id.to_string()),
        name: json_string(&item, "Name")
            .unwrap_or_else(|| id.to_string())
            .trim_start_matches('/')
            .to_string(),
        image: nested_string(&config, &["Image"])
            .or_else(|| json_string(&item, "Image"))
            .unwrap_or_default(),
        command,
        created: json_string(&item, "Created").unwrap_or_default(),
        status: state
            .get("Status")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string(),
        state: state
            .get("Status")
            .and_then(Value::as_str)
            .map(state_from_status)
            .unwrap_or_else(|| "unknown".to_string()),
        running: json_bool(&state, "Running"),
        paused: json_bool(&state, "Paused"),
        restarting: json_bool(&state, "Restarting"),
        dead: json_bool(&state, "Dead"),
        pid: json_u64(&state, "Pid"),
        exit_code: json_i64(&state, "ExitCode"),
        started_at: state
            .get("StartedAt")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned)
            .filter(|value| !value.is_empty()),
        finished_at: state
            .get("FinishedAt")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned)
            .filter(|value| !value.is_empty()),
        health: state
            .get("Health")
            .and_then(|health| health.get("Status"))
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        network_mode: item
            .get("HostConfig")
            .and_then(|config| config.get("NetworkMode"))
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        ip_address: item
            .get("NetworkSettings")
            .and_then(|network| network.get("IPAddress"))
            .and_then(Value::as_str)
            .map(ToOwned::to_owned)
            .filter(|value| !value.is_empty()),
        labels: json_labels(&item),
        environment: json_environment(&item),
        mounts: json_mounts(&item),
        ports: json_ports(&item),
    })
}

#[tauri::command]
fn list_containers(all: Option<bool>) -> Result<Vec<ContainerSummary>, String> {
    let mut args = vec!["ps", "--no-trunc", "--format", "{{json .}}"];

    if all.unwrap_or(true) {
        args.insert(1, "-a");
    }

    let raw = run_docker(&args).map_err(|error| error.to_string())?;

    if raw.trim().is_empty() {
        return Ok(Vec::new());
    }

    raw.lines()
        .filter(|line| !line.trim().is_empty())
        .map(parse_summary_line)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn inspect_container(id: String) -> Result<ContainerDetails, String> {
    inspect_container_inner(&id).map_err(|error| error.to_string())
}

#[tauri::command]
fn start_container(id: String) -> Result<ContainerDetails, String> {
    run_docker(&["start", &id]).map_err(|error| error.to_string())?;
    inspect_container_inner(&id).map_err(|error| error.to_string())
}

#[tauri::command]
fn stop_container(id: String) -> Result<ContainerDetails, String> {
    run_docker(&["stop", "--time", "10", &id]).map_err(|error| error.to_string())?;
    inspect_container_inner(&id).map_err(|error| error.to_string())
}

#[tauri::command]
fn tail_logs(id: String, tail: Option<u32>) -> Result<String, String> {
    let tail_arg = tail.unwrap_or(200).to_string();
    run_docker(&["logs", "--tail", &tail_arg, "--timestamps", &id])
        .map_err(|error| error.to_string())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            list_containers,
            inspect_container,
            start_container,
            stop_container,
            tail_logs
        ])
        .run(tauri::generate_context!())
        .expect("failed to run tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_docker_ps_row_with_uppercase_id_field() {
        let row = r#"{"ID":"1234567890abcdef","Image":"alpine:latest","Command":"\"sleep 1000\"","CreatedAt":"2026-05-29 12:00:00 +0000 UTC","RunningFor":"5 minutes ago","Ports":"0.0.0.0:8080->80/tcp","Status":"Up 5 minutes","Names":"demo"}"#;

        let summary = parse_summary_line(row).expect("row should parse");

        assert_eq!(summary.id, "1234567890abcdef");
        assert_eq!(summary.name, "demo");
        assert_eq!(summary.state, "running");
    }
}
