use procfs::{process::Process, ProcResult};
use sysinfo::SystemExt;

pub fn dump_sys_usages() -> String {
	let mut result = String::new();
	result.push_str(&format!("general info:\n{}\n", general_info()));
	if let Ok(current) = current_process_info() {
		result.push_str(&format!("current process info:\n{}\n", current));
	}
	result
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

fn current_process_info() -> ProcResult<String> {
	let me = Process::myself()?;
	let me_sat = me.stat()?;

	let mut result = String::new();

	result.push_str(&format!("PID: {}, fd count: {}\n", me.pid, me.fd_count()?));
	result.push_str(&format!(
		"Memory page size: {} bytes\n",
		procfs::page_size()
	));
	result.push_str(&format!(
		"Total virtual memory: {}M bytes\n",
		me_sat.vsize / 1024 / 1024
	));
	result.push_str(&format!(
		"Resident set size: {}M bytes\n",
		me_sat.rss * 4096 / 1024 / 1024
	));

	Ok(result)
}
