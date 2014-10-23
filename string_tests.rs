/*
    This file is part of rust-irc - a Rust Library for connecting to IRC servers
    Copyright (C) 2014 Jona Stubbe

    rust-irc is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    rust-irc is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with rust-irc.  If not, see <http://www.gnu.org/licenses/>.*/

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

#[test]
fn test() {
	assert!("blah".is_valid_nick());
	assert!(!"Ümläüt".is_valid_nick());
	assert!(!"no spaces in nicknames".is_valid_nick());
	assert!(!"no@special-chars.in+nicks".is_valid_nick())
	assert!("no newline".no_newline());
	assert!(!"has\nnewline".no_newline());
	assert!(!"telnet-style newline\r\n".no_newline());
}