use wasmparser::Operator;

pub fn pricing(_op: &Operator) -> u64 {
	1
}

pub const MEMORY_LIMIT: Option<u64> = None;
