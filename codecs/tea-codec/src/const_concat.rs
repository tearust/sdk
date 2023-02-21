const BUFFER_SIZE: usize = 8192;

pub struct ConstStr {
	data: [u8; BUFFER_SIZE],
	len: usize,
}

impl ConstStr {
	pub const fn empty() -> ConstStr {
		ConstStr {
			data: [0u8; BUFFER_SIZE],
			len: 0,
		}
	}

	pub const fn append_str(mut self, s: &str) -> Self {
		let b = s.as_bytes();
		let mut index = 0;
		while index < b.len() {
			self.data[self.len] = b[index];
			self.len += 1;
			index += 1;
		}

		self
	}

	pub const fn as_str(&self) -> &str {
		let mut data: &[u8] = &self.data;
		let mut n = data.len() - self.len;
		while n > 0 {
			n -= 1;
			match data.split_last() {
				Some((_, rest)) => data = rest,
				None => panic!(),
			}
		}
		unsafe { std::str::from_utf8_unchecked(data) }
	}
}
