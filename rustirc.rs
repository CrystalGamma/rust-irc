#![experimental="rustirc is not yet feature-complete"]
#![crate_name="rustirc"]
#![crate_type="lib"]
#![crate_type="bin"]

#![feature(macro_rules)]

use std::io::{TcpStream, IoResult, BufferedReader};

macro_rules! assume(
    ($e:expr, $msg:expr) => (match $e { Some(ref mut e) => e, None => fail!($msg) })
)
macro_rules! ioassume(
    ($e:expr, $msg:expr) => (match $e { Ok(e) => e, Err(msg) => fail!($msg, msg) })
)


pub struct Connection {
	write: Box<TcpStream>,
	read: Box<BufferedReader<TcpStream>>,
	names: Option<Box<Iterator<String> + 'static>>,
	nick: Option<String>,
	user_name: String,
	real_name: String
}

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


impl Connection {
	fn nick_message(&mut self) -> IoResult<()> {
		self.nick = assume!(self.names, "nick_message called after successful nickname selection").next();
		let nick = assume!(self.nick,"nick name generator did not generate enough nicks");
		assert!(nick.is_valid_nick());
		let out = format!("NICK {}\r\nUSER {} - - 8 :{}\r\n", nick, self.user_name, self.real_name);
		print!("{}",out);
		try!(self.write.write_str(out.as_slice()));
		Ok(())
	}
	pub fn connect(conn: TcpStream, names: Box<Iterator<String> + 'static>, user_name: String, real_name: String) -> IoResult<Connection> {
		assert!(user_name.is_valid_nick() && real_name.no_newline());
		let mut irc = Connection {
			read: box BufferedReader::new(conn.clone()),
			write: box conn,
			names: Some(names),
			nick: None,
			user_name: user_name,
			real_name: real_name};
		try!(irc.nick_message());
		Ok(irc)
	}
	
	pub fn eventloop(&mut self) -> IoResult<()> {
		loop {
			let line = match self.read.lines().next() {Some(x)=>x,None=>break};
			match self.names {
				Some(_) => {try!(self.nick_message());},
				_ =>{}};
			println!("{}", line);
		}
		Ok(())
	}
}

struct NickGenerator {
	basename: &'static str,
	attempt: uint
}

impl Iterator<String> for NickGenerator {
	fn next(&mut self) -> Option<String> {
		self.attempt += 1;
		Some(if self.attempt > 1 {
			format!("{}{}", self.basename, self.attempt)
		} else {
			self.basename.to_string()
		})
	}
}

pub fn main() {
	let mut conn = ioassume!(Connection::connect(
		ioassume!(TcpStream::connect("chat.freenode.net", 6667), "TCP Connection failed: {}"),
		box NickGenerator {basename: "CrystalGBot", attempt:0},
		"CrystalGBot".to_string(),
		"CrystalGamma experimental chat bot implemented in Rust".to_string()), "IRC connection failed: {}");
	conn.eventloop();
}