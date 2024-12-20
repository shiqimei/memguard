use clap::Parser;
use sysinfo::{ProcessExt, Signal, System, SystemExt, PidExt};
use std::{time::Duration, fs::File, io::Write, process::Command};
use byte_unit::Byte;
use log::{info, warn, error};

#[derive(Parser, Debug)]
#[command(author, version, about = "Memory guard daemon")]
struct Args {
    /// Maximum memory allowed per process (e.g., "16GB", "1000MB")
    #[arg(long, default_value = "16GB")]
    max_memory: String,

    /// Check interval in seconds
    #[arg(long, default_value = "1")]
    interval: u64,
}

struct ProcessInfo {
    pid: u32,
    uid: String,
    name: String,
}

fn get_process_info(process: &sysinfo::Process) -> Option<ProcessInfo> {
    let uid = process.user_id()?.to_string();
    
    Some(ProcessInfo {
        pid: process.pid().as_u32(),
        uid,
        name: process.name().to_string(),
    })
}

fn find_port_8000_process(uid: &str) -> Option<u32> {
    // Run lsof to find process using port 8000
    let output = Command::new("lsof")
        .args(["-i", ":8000", "-t"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    // Convert output to string and parse PID
    let pid_str = String::from_utf8_lossy(&output.stdout);
    for line in pid_str.lines() {
        if let Ok(pid) = line.trim().parse::<u32>() {
            // Check if this process belongs to the specified user
            if let Ok(output) = Command::new("ps")
                .args(["-o", "uid=", "-p", &pid.to_string()])
                .output()
            {
                let proc_uid = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if proc_uid == uid {
                    return Some(pid);
                }
            }
        }
    }
    None
}

fn write_message_to_proc(pid: u32, memory: u64, max_memory: u64) -> std::io::Result<()> {
    let proc_fd_path = format!("/proc/{}/fd/2", pid);
    if let Ok(mut file) = File::create(proc_fd_path) {
        let message = format!(
            "\n\x1b[31mProcess killed by memguard: Memory usage {} exceeds limit {}\x1b[0m\n",
            format_bytes(memory),
            format_bytes(max_memory)
        );
        let _ = file.write_all(message.as_bytes());
    }
    Ok(())
}

fn kill_process(sys: &System, pid: u32, msg: &str) {
    if let Some(process) = sys.process(sysinfo::Pid::from_u32(pid)) {
        // Try to write message to process stderr
        let _ = write_message_to_proc(pid, process.memory(), 0);
        
        match process.kill_with(Signal::Term) {
            Some(true) => {
                info!("{} - Sent SIGTERM to process {}", msg, pid);
                std::thread::sleep(Duration::from_millis(100));
            },
            Some(false) | None => {
                if process.kill() {
                    info!("{} - Sent SIGKILL to process {}", msg, pid);
                } else {
                    error!("{} - Failed to kill process {}", msg, pid);
                }
            }
        }
    }
}

fn parse_memory(mem_str: &str) -> Result<u64, String> {
    Byte::from_str(mem_str)
        .map(|b| b.get_bytes() as u64)
        .map_err(|e| format!("Failed to parse memory size: {}", e))
}

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1024 * 1024 * 1024 {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    } else if bytes >= 1024 * 1024 {
        format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.2} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let args = Args::parse();

    let max_memory_bytes = parse_memory(&args.max_memory)?;
    let interval = Duration::from_secs(args.interval);

    info!("Starting memguard...");
    info!("Maximum allowed memory per process: {}", format_bytes(max_memory_bytes));
    info!("Check interval: {:?}", interval);

    let mut sys = System::new_all();

    loop {
        sys.refresh_all();

        for (_pid, process) in sys.processes() {
            let memory = process.memory();
            
            if memory > max_memory_bytes {
                if let Some(proc_info) = get_process_info(process) {
                    let msg = format!(
                        "Process '{}' (PID: {}, UID: {}) killed by memguard: Memory usage {} exceeds limit {}",
                        proc_info.name, proc_info.pid, proc_info.uid, 
                        format_bytes(memory), format_bytes(max_memory_bytes)
                    );
                    warn!("{}", msg);

                    // Kill the memory-exceeding process
                    kill_process(&sys, proc_info.pid, "Memory limit exceeded");

                    // Find and kill related process on port 8000
                    if let Some(port_pid) = find_port_8000_process(&proc_info.uid) {
                        if port_pid != proc_info.pid {
                            kill_process(&sys, port_pid, "Coupled process (port 8000)");
                        }
                    }
                }
            }
        }

        std::thread::sleep(interval);
    }
}