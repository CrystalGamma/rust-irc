#![experimental="rustirc is not yet feature-complete"]
#![crate_name="rustirc"]
#![crate_type="lib"]

#![feature(macro_rules)]

use std::io::TcpStream;
use std::io::IoResult;

macro_rules! assume(
    ($e:expr) => (match $e { Some(ref mut e) => e, None => unreachable!() })
)

pub struct Connection {
	conn: Box<TcpStream>,
	names: Option<Box<Iterator<String> + 'static>>,
	nick: Option<String>
}

pub trait NickVerifier {
	fn is_valid_nick(&self) -> bool;
}

impl<'a> NickVerifier for &'a str {
	fn is_valid_nick(&self) -> bool {
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
}

impl NickVerifier for String {
	fn is_valid_nick(&self) -> bool {
		self.as_slice().is_valid_nick()
	}
}


impl Connection {
	fn nick_message(&mut self) -> IoResult<()> {
		self.nick = assume!(self.names).next();
		assert!(assume!(self.nick).as_slice().is_valid_nick());
		try!(self.conn.write_str(format!("NICK {}\n", self.nick).as_slice()));
		Ok(())
	}
	pub fn connect(conn: Box<TcpStream>, names: Box<Iterator<String> + 'static>) -> IoResult<Connection> {
		let mut irc = Connection {conn: conn, names: Some(names), nick: None};
		try!(irc.nick_message());
		Ok(irc)
	}
}