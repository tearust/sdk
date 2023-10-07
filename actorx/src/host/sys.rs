use procfs::{process::Process, ProcResult};
use sysinfo::SystemExt;
use tabled::{Table, Tabled};

pub fn dump_sys_usages() -> String {
	let mut result = String::new();
	result.push_str(&format!("general info:\n{}\n", general_info()));
	if let Ok(current) = process_info() {
		result.push_str(&format!("process info:\n{}\n", current));
	}
	result
}

pub fn get_memory_usage() -> ProcResult<(u64, u64)> {
	let mut sys = sysinfo::System::new_all();
	sys.refresh_all();
	let total = sys.total_memory();
	let free = sys.free_memory();
	Ok((total, free))
}

fn general_info() -> String {
	let mut sys = sysinfo::System::new_all();
	sys.refresh_all();

	let mut result = String::new();
	result.push_str(&format!(
		"total memory: {}M bytes\n",
		sys.total_memory() / 1024 / 1024
	));
	result.push_str(&format!(
		"used memory: {}M bytes\n",
		sys.used_memory() / 1024 / 1024
	));
	result.push_str(&format!(
		"total swap: {}M bytes\n",
		sys.total_swap() / 1024 / 1024
	));
	result.push_str(&format!(
		"used swap: {}M bytes\n",
		sys.used_swap() / 1024 / 1024
	));

	let load_avg = sys.load_average();
	result.push_str(&format!(
		"load average. one minute: {}%, five minutes: {}%, fifteen minutes: {}%\n",
		load_avg.one, load_avg.five, load_avg.fifteen
	));

	result.push_str(&format!("process count: {}\n", sys.processes().len()));

	result
}

#[derive(Tabled)]
struct ProcessInfo {
	pid: i32,
	fd_count: usize,
	memory: String,
	app: String,
}

impl ProcessInfo {
	fn new(pro: &Process, memory_m_bytes: u64, app_pid: i32) -> ProcResult<Self> {
		Ok(Self {
			pid: pro.pid,
			fd_count: pro.fd_count().unwrap_or(0),
			memory: format!("{memory_m_bytes}M"),
			app: if pro.pid == app_pid { "*" } else { "" }.to_string(),
		})
	}
}

fn process_info() -> ProcResult<String> {
	let app = Process::myself()?;

	let mut processes = Vec::new();
	for prc in procfs::process::all_processes()? {
		let prc = prc?;
		let m_bytes = prc.stat()?.rss_bytes() / 1024 / 1024;
		if m_bytes == 0 {
			continue;
		}
		let info = ProcessInfo::new(&prc, m_bytes, app.pid);
		if let Ok(info) = info {
			processes.push(info);
		}
	}
	Ok(Table::new(processes).to_string())
}
