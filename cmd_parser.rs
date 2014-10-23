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


pub trait IrcLine{
	fn decode_irc_event<'a>(&'a self) -> Option<IrcEvent<'a>>;
}
pub struct IrcEvent<'a> {
	pub sender: &'a str,
	pub prefix: &'a str,
	pub cmd: &'a str,
	pub args: Vec<&'a str>
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
		let sender = match prefix.find(|c: char| c=='!') {
			Some(x) => prefix.slice_to(x),
			None => prefix};
		let cmd = tryopt!(segments.next(), None);
		let mut args: Vec<&str> = segments.filter(|st: &&str|st.len()>0).collect();
		match parts.next() {
			Some(x)=>args.push_all([x]),
			None=>{}
		};
		Some(IrcEvent {sender: sender, prefix: prefix, cmd: cmd, args:args})
	}
}