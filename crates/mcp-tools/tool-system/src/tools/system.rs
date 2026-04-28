use mcp_core::{Tool, ToolEntry, ToolError};
use serde_json::Value;
use sysinfo::System;
use std::fmt::Write;

const GB: f64 = 1_073_741_824.0;

pub struct GetRamUsageTool;

impl Tool for GetRamUsageTool {
    fn name(&self) -> &'static str { "get_ram_usage" }
    fn description(&self) -> &'static str { "Get the current RAM usage of the system." }
    fn input_schema(&self) -> Value {
        serde_json::json!({ "type": "object", "properties": {}, "required": [] })
    }
    fn execute(&self, _args: Value) -> Result<String, ToolError> {
        let mut sys = System::new_all();
        sys.refresh_all();
        let total = sys.total_memory();
        let used = sys.used_memory();
        let total_gb = total as f64 / GB;
        let used_gb = used as f64 / GB;
        let percent = (used as f64 / total as f64) * 100.0;
        Ok(format!("RAM Usage: {:.2} GB / {:.2} GB ({:.1}%)", used_gb, total_gb, percent))
    }
}

inventory::submit! { ToolEntry { tool: &GetRamUsageTool } }

pub struct GetCpuUsageTool;

impl Tool for GetCpuUsageTool {
    fn name(&self) -> &'static str { "get_cpu_usage" }
    fn description(&self) -> &'static str { "Get the current CPU usage of the system." }
    fn input_schema(&self) -> Value {
        serde_json::json!({ "type": "object", "properties": {}, "required": [] })
    }
    fn execute(&self, _args: Value) -> Result<String, ToolError> {
        let mut sys = System::new_all();
        sys.refresh_all();
        let cpus = sys.cpus();
        let mut result = String::new();
        writeln!(result, "CPU Usage ({} cores):", cpus.len()).unwrap();
        for (i, cpu) in cpus.iter().enumerate() {
            writeln!(result, "  Core {}: {:.1}%", i, cpu.cpu_usage()).unwrap();
        }
        let avg = cpus.iter().map(|c| c.cpu_usage() as f64).sum::<f64>() / cpus.len() as f64;
        writeln!(result, "Average: {:.1}%", avg).unwrap();
        Ok(result)
    }
}

inventory::submit! { ToolEntry { tool: &GetCpuUsageTool } }

pub struct GetDiskUsageTool;

impl Tool for GetDiskUsageTool {
    fn name(&self) -> &'static str { "get_disk_usage" }
    fn description(&self) -> &'static str { "Get the current disk space usage." }
    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "The path to check disk usage for (default: root)" }
            },
            "required": []
        })
    }
    fn execute(&self, _args: Value) -> Result<String, ToolError> {
        use sysinfo::Disks;
        let disks = Disks::new_with_refreshed_list();
        let mut result = String::new();
        writeln!(result, "Disk Usage:").unwrap();
        for disk in &disks {
            let name = disk.name().to_string_lossy();
            let total = disk.total_space();
            let available = disk.available_space();
            let used = total - available;
            let percent = (used as f64 / total as f64) * 100.0;
            let total_gb = total as f64 / GB;
            let used_gb = used as f64 / GB;
            writeln!(result, "  {} ({:?}): {:.2} GB / {:.2} GB ({:.1}%)", name, disk.mount_point(), used_gb, total_gb, percent).unwrap();
        }
        Ok(result)
    }
}

inventory::submit! { ToolEntry { tool: &GetDiskUsageTool } }

pub struct GetProcessesTool;

impl Tool for GetProcessesTool {
    fn name(&self) -> &'static str { "get_processes" }
    fn description(&self) -> &'static str { "List running processes on the system." }
    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "limit": { "type": "integer", "description": "Maximum number of processes to return (default: 20)" }
            },
            "required": []
        })
    }
    fn execute(&self, args: Value) -> Result<String, ToolError> {
        let limit = args["limit"].as_u64().unwrap_or(20) as usize;
        let mut sys = System::new_all();
        sys.refresh_all();
        let mut processes: Vec<_> = sys.processes().iter().collect();
        processes.sort_by(|a, b| {
            b.1.cpu_usage().partial_cmp(&a.1.cpu_usage()).unwrap_or(std::cmp::Ordering::Equal)
        });
        let mut result = String::new();
        writeln!(result, "Top {} processes by CPU usage:\n", limit.min(processes.len())).unwrap();
        for (pid, process) in processes.iter().take(limit) {
            writeln!(result, "{:<10} {:<30} CPU: {:5.1}%  MEM: {:5.1}%", 
                format!("[{}]", pid), 
                process.name().to_string_lossy(), 
                process.cpu_usage(), 
                (process.memory() as f64 / GB) * 100.0).unwrap();
        }
        Ok(result)
    }
}

inventory::submit! { ToolEntry { tool: &GetProcessesTool } }