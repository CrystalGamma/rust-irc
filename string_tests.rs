pub trait StringTests {
	fn is_valid_nick(&self) -> bool;
	fn no_newline(&self) -> bool;
}

impl<'a> StringTests for &'a str {
	fn is_valid_nick(&self) -> bool {
		println!("Nickname test for: {}", self);
		let mut iter = self.bytes();
		let b = iter.next();
		match  b {
		Some(48u8...57u8) | None => {return false;} // 0-9
		_ => {}
		}

		for c in self.bytes() {
			match c {
			65u8...90u8 | 97u8...122u8 | 48u8 ...57u8 => continue, // A-Z, a-z, 0-9
			_ => return false
			}
		}
		true
	}
	
	fn no_newline(&self) -> bool {
		println!("No newline test for: {}", self);
		!self.chars().any(|c| c=='\r' || c=='\n')
	}
}

impl StringTests for String {
	fn is_valid_nick(&self) -> bool {
		self.as_slice().is_valid_nick()
	}
	fn no_newline(&self) -> bool {
		self.as_slice().no_newline()
	}
}