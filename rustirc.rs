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

#![experimental="rustirc is not yet feature-complete"]
#![crate_name="rustirc"]
#![license="GPLv3"]
#![crate_type="lib"]

#![feature(macro_rules)]


use std::io::{TcpStream, IoResult, Buffer, Writer, IoError, OtherIoError};
pub use string_tests::StringTests;
use cmd_parser::IrcLine;
pub use cmd_parser::IrcEvent;
mod string_tests;

macro_rules! assume(
    ($e:expr, $msg:expr) => (match $e { Some(ref mut e) => e, None => fail!($msg) })
)
macro_rules! assume_cpy(
    ($e:expr, $msg:expr) => (match $e { Some(e) => e, None => fail!($msg) })
)
macro_rules! tryopt( ($e:expr, $default:expr) => (match $e {Some(e)=>e, None=>return $default}))

mod cmd_parser;

pub trait CloseWrite {
	fn close_write(&mut self) -> IoResult<()>;
}

impl CloseWrite for TcpStream {
	fn close_write(&mut self) -> IoResult<()> { self.close_write() }
}

pub trait IrcWriter: Clone + Send {
	fn login(&mut self, nick: &str, user_name: &str, real_name: &str) -> IoResult<()>;
	fn quit(&mut self) -> IoResult<()>;
	fn join(&mut self, channel: &str) -> IoResult<()>;
	fn pong(&mut self, data: &str) -> IoResult<()>;
	fn notice(&mut self, target: &str, text: &str) -> IoResult<()>;
	fn channel_notice(&mut self, target: &str, text: &str) -> IoResult<()>;
	fn message(&mut self, target: &str, text: &str) -> IoResult<()>;
}

impl<T: Writer + CloseWrite + Clone + Send> IrcWriter for T {
	fn login(&mut self, nick: &str, user_name: &str, real_name: &str) -> IoResult<()> {
		let out = format!("NICK {}\r\nUSER {} 8 * :{}\r\n", nick, user_name, real_name);
		print!("{}",out);
		try!(self.write_str(out.as_slice()));
		Ok(())
	}
	fn quit(&mut self) -> IoResult<()> {
		try!(self.write_str("QUIT\r\n"));
		self.close_write()
	}
	fn join(&mut self, channel: &str) -> IoResult<()> {
		assert!(channel.no_newline());
		self.write_str(format!("JOIN :{}\r\n", channel).as_slice())
	}
	fn pong(&mut self, data: &str) -> IoResult<()> {
		assert!(data.no_newline());
		self.write_str(format!("PONG :{}\r\n", data).as_slice())
	}
	fn notice(&mut self, target: &str, text: &str) -> IoResult<()> {
		assert!(text.no_newline() && target.no_newline()); // TODO: make a string test for target lists
		let out = format!("NOTICE {} :{}\r\n", target, text);
		print!("{}", out);
		self.write_str(out.as_slice())
	}
	fn message(&mut self, target: &str, text: &str) -> IoResult<()> {
		assert!(text.no_newline() && target.no_newline()); // TODO: make a string test for target lists
		let out = format!("PRIVMSG {} :{}\r\n", target, text);
		print!("{}", out);
		self.write_str(out.as_slice())
	}
	fn channel_notice(&mut self, target: &str, text: &str) -> IoResult<()> {
		if cfg!(channel_notice) {
			self.notice(target, text)
		} else {
			self.message(target, text)
		}
	}
}

#[cfg(test)]
mod test_writer;

pub struct Connection<In, Out, Handler> {
	read: In,
	write: Out,
	nick: String,
	nick_status: NickStatus,
	user_name: String,
	real_name: String,
	event_handler: Handler
}

enum NickStatus {
	Registering(Box<Iterator<String> + 'static>),
	Accepted
}

#[allow(unused_variable)]
pub trait IrcEventHandler {
	fn on_registered<W: IrcWriter>(&mut self, &mut W) -> IoResult<()> {Ok(())}
	fn on_privmsg<'a, W: IrcWriter>(&mut self, text: &str, ev: &IrcEvent<'a>, write: &mut W) -> IoResult<()> {Ok(())}
}

pub struct LinesTolerant<'a, T: 'a> {
	read: &'a mut T
}

impl<'a, T: TolerantLineReader> Iterator<IoResult<String>> for LinesTolerant<'a, T> {
	fn next(&mut self) -> Option<IoResult<String>> {
		self.read.read_line_tolerant()
	}
}

pub trait TolerantLineReader {
	fn read_line_tolerant(&mut self) -> Option<IoResult<String>>;
	fn lines_tolerant<'a>(&'a mut self) -> LinesTolerant<'a, Self> {
		LinesTolerant {read: self}
	}
}

impl<T: Buffer> TolerantLineReader for T {
	fn read_line_tolerant(&mut self) -> Option<IoResult<String>> {
		Some(Ok(String::from_utf8_lossy(match self.read_until(b'\n') {
			Ok(x) => x,
			Err(e) => return Some(Err(e))
		}.as_slice()).into_string()))
	}
}


impl<In: Buffer, Out: IrcWriter, Handler: IrcEventHandler> Connection<In, Out, Handler> {
	pub fn connect<T: Iterator<String> + 'static>(read: In, write: Out, mut names: T, user_name: String, real_name: String, event_handler: Handler) -> IoResult<Connection<In, Out, Handler>> {
		assert!(user_name.is_valid_nick() && real_name.no_newline());
		let mut irc = Connection {
			read: read,
			write: write,
			nick: match names.next() {Some(x)=>x, None=>fail!("nick name generator did not generate enough nicks")},
			nick_status: Registering(box names),
			user_name: user_name,
			real_name: real_name,
			event_handler: event_handler};
		assert!(irc.nick.is_valid_nick());
		try!(irc.write.login(irc.nick.as_slice(), irc.user_name.as_slice(), irc.real_name.as_slice()));
		Ok(irc)
	}
	
	pub fn eventloop(&mut self) -> IoResult<()> {
		for line in self.read.lines_tolerant() {
			if cfg!(debug) {
				print!("{}",line);
			}
			let event = match line {
				Ok(x) => x,
				Err(ref e) if e.kind == std::io::EndOfFile => return Ok(()),
				Err(e) => return Err(e)
			};
			let slice = event.as_slice();
			let ev = tryopt!(slice.decode_irc_event(), Err(IoError{kind: OtherIoError, desc: "malformed IRC event received", detail: None}));
			if cfg!(debug_verbose) {
				println!("prefix:{}\n  command: {}\n  args:{}", ev.prefix, ev.cmd, ev.args);
			}
			match ev.cmd {
			"PRIVMSG" => {
				if ev.args.len() != 2 {
					return Err(IoError{kind: OtherIoError, desc: "malformed PRIVMSG command received", detail: None})
				}
				let text = match ev.args.last() {None => unreachable!(), Some(x) => x};
				println!("message received: {}", text);
				try!(self.event_handler.on_privmsg(*text, &ev, &mut self.write));
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
				try!(self.event_handler.on_registered(&mut self.write));
			},
			"433" | "436" => {
				self.nick = match self.nick_status {
					Registering(ref mut iter) => assume_cpy!(iter.next(),"nick name generator did not generate enough nicks"),
					Accepted => return Err(IoError{kind: OtherIoError, desc: "unexpected 433 ERR_NICKNAMEINUSE or 436 ERR_NICKCOLLISION received", detail: None})};
				assert!(self.nick.is_valid_nick());
				println!("Rejected, trying nickname {}", self.nick);
				try!(self.write.login(self.nick.as_slice(), self.user_name.as_slice(), self.real_name.as_slice()));
			},
			_ => {}
			}
		}
		unreachable!();
	}
}