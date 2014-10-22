#![experimental="rustirc is not yet feature-complete"]
#![crate_name="rustirc"]
#![crate_type="lib"]
#![crate_type="bin"]

#![feature(macro_rules)]


use std::io::{TcpStream, IoResult, BufferedReader, Writer, IoError, OtherIoError};
use string_tests::StringTests;
mod string_tests;

macro_rules! assume(
    ($e:expr, $msg:expr) => (match $e { Some(ref mut e) => e, None => fail!($msg) })
)
macro_rules! assume_cpy(
    ($e:expr, $msg:expr) => (match $e { Some(e) => e, None => fail!($msg) })
)
macro_rules! ioassume(
    ($e:expr, $msg:expr) => (match $e { Ok(e) => e, Err(msg) => fail!($msg, msg) })
)
macro_rules! tryopt( ($e:expr, $default:expr) => (match $e {Some(e)=>e, None=>return $default}))

trait IrcLine{
	fn decode_irc_event<'a>(&'a self) -> Option<IrcEvent<'a>>;
}
struct IrcEvent<'a> {
	prefix: &'a str,
	cmd: &'a str,
	args: Vec<&'a str>
}
impl<'t> IrcLine for &'t str {
	fn decode_irc_event<'a>(&'a self) -> Option<IrcEvent<'a>> {
		let line = self.trim_right_chars(|c: char| c == '\r' || c == '\n');
		let bytes = line.as_bytes();
		let hasprefix = bytes[0] == 58; // ':'
		let mut parts = line.splitn(if hasprefix {2} else {1}, |c: char| c == ':');
		if hasprefix { parts.next(); }
		let mut segments = tryopt!(parts.next(), None).split(|c: char| c == ' ');
		let prefix = if hasprefix {tryopt!(segments.next(), None)} else {""};
		let cmd = tryopt!(segments.next(), None);
		let mut args: Vec<&str> = segments.filter(|st: &&str|st.len()>0).collect();
		match parts.next() {
			Some(x)=>args.push_all([x]),
			None=>{}
		};
		Some(IrcEvent {prefix: prefix, cmd: cmd, args:args})
	}
}
/*impl IrcLine for String {
	fn decode_irc_event<'a>(&'a self) -> Option<IrcEvent<'a>> {
		self.as_slice().decode_irc_event()
	}
}*/

struct IrcReader<T> {
	read: BufferedReader<T>
}

impl<T: Reader> Iterator<String> for IrcReader<T> {
	fn next(&mut self) -> Option<String> {
		let st = ioassume!(match self.read.lines().next() {Some(x)=>x, None=>return None}, "could not read event: {}");
		Some(st)
	}
}
impl<T: Reader> IrcReader<T> {
	pub fn new(read: T) -> IrcReader<T> { IrcReader {read: BufferedReader::new(read)} }
}

trait CloseWrite {
	fn close_write(&mut self) -> IoResult<()>;
}

impl CloseWrite for TcpStream {
	fn close_write(&mut self) -> IoResult<()> { self.close_write() }
}

struct IrcWriter<T> {
	write: T
}

impl<T: Writer + CloseWrite> IrcWriter<T> {
	pub fn new(write: T) -> IrcWriter<T> { IrcWriter{write: write} }
//	pub fn get_inner(&self) -> &T { &self.write }
	fn nick_message(&mut self, nick: &str, user_name: &str, real_name: &str) -> IoResult<()> {
		let out = format!("NICK {}\r\nUSER {} 8 * :{}\r\n", nick, user_name, real_name);
		print!("{}",out);
		try!(self.write.write_str(out.as_slice()));
		Ok(())
	}
	fn quit(&mut self) -> IoResult<()> {
		try!(self.write.write_str("QUIT\r\n"));
		self.write.close_write()
	}
	fn join(&mut self, channel: &str) -> IoResult<()> {
		assert!(channel.no_newline());
		self.write.write_str(format!("JOIN :{}\r\n", channel).as_slice())
	}
	fn pong(&mut self, data: &str) -> IoResult<()> {
		assert!(data.no_newline());
		self.write.write_str(format!("PONG :{}\r\n", data).as_slice())
	}
}

pub struct Connection {
	read: IrcReader<TcpStream>,
	write: IrcWriter<TcpStream>,
	nick: String,
	nick_status: NickStatus,
	user_name: String,
	real_name: String
}

enum NickStatus {
	Registering(Box<Iterator<String> + 'static>),
	Accepted
}


impl Connection {
	pub fn connect<T: Iterator<String> + 'static>(conn: TcpStream, mut names: T, user_name: String, real_name: String) -> IoResult<Connection> {
		assert!(user_name.is_valid_nick() && real_name.no_newline());
		let mut irc = Connection {
			read: IrcReader::new(conn.clone()),
			write: IrcWriter::new(conn),
			nick: match names.next() {Some(x)=>x, None=>fail!("nick name generator did not generate enough nicks")},
			nick_status: Registering(box names),
			user_name: user_name,
			real_name: real_name};
		assert!(irc.nick.is_valid_nick());
		try!(irc.write.nick_message(irc.nick.as_slice(), irc.user_name.as_slice(), irc.real_name.as_slice()));
		Ok(irc)
	}
	
	pub fn eventloop(&mut self) -> IoResult<()> {
		//let mut tries = 5u;
		for event in self.read {
			/*match self.nick_status {
				Registering(_) => {
					try!(self.write.nick_message(self.nick.as_slice(), self.user_name.as_slice(), self.real_name.as_slice()));
				},
				Accepted =>{}};*/
			let slice = event.as_slice();
			let ev = tryopt!(slice.decode_irc_event(), Err(IoError{kind: OtherIoError, desc: "malformed IRC event received", detail: None}));
			println!("{}  prefix:{}\n  command: {}\n  args:{}", event, ev.prefix, ev.cmd, ev.args);
			match ev.cmd {
			"PRIVMSG" => {
				if ev.args.len() != 2 {
					return Err(IoError{kind: OtherIoError, desc: "malformed PRIVMSG command received", detail: None})
				}
				let text = match ev.args.last() {None => unreachable!(), Some(x) => x};
				println!("message recieved: {}", text);
				if *text == "!kill" {
					try!(self.write.quit());
				}
			},
			"PING" => {
				if ev.args.len() != 1 {
					return Err(IoError{kind: OtherIoError, desc: "malformed PING command received", detail: None})
				}
				try!(self.write.pong(ev.args[0]));
			},
			"001" => {
				self.nick_status = Accepted;
				println!("Server accepted nickname {}", self.nick);
				try!(self.write.join("#Deathmic"));
			},
			"433" | "436" => {
				self.nick = match self.nick_status {
					Registering(ref mut iter) => assume_cpy!(iter.next(),"nick name generator did not generate enough nicks"),
					Accepted => return Err(IoError{kind: OtherIoError, desc: "unexpected 433 ERR_NICKNAMEINUSE or 436 ERR_NICKCOLLISION received", detail: None})};
				assert!(self.nick.is_valid_nick());
				println!("Rejected, trying nickname {}", self.nick);
				try!(self.write.nick_message(self.nick.as_slice(), self.user_name.as_slice(), self.real_name.as_slice()));
			},
			_ => {}
			}
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
		ioassume!(TcpStream::connect("irc.quakenet.org", 6667), "TCP Connection failed: {}"),
		NickGenerator {basename: "CrystalGBot", attempt:0},
		"CrystalGBot".to_string(),
		"CrystalGamma experimental chat bot implemented in Rust".to_string()), "IRC connection failed: {}");
	ioassume!(conn.eventloop(),"main loop failed: {}");
}