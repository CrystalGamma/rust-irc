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
use std::io::{MemWriter, IoResult};
use IrcWriter;
use CloseWrite;

impl CloseWrite for MemWriter {
	fn close_write(&mut self) -> IoResult<()> { Ok(()) }
}

#[test]
fn test_irc_login() {
	let mut irc = IrcWriter::new(MemWriter::new());
	assert_eq!(irc.login("TestNick", "testuser", "Test Real Name"), Ok(()));
	assert_eq!(irc.get_inner().get_ref(), b"NICK TestNick\r\nUSER testuser 8 * :Test Real Name\r\n");
}
#[test]
fn test_irc_pong() {
	let mut irc = IrcWriter::new(MemWriter::new());
	assert_eq!(irc.pong("123456789"), Ok(()));
	assert_eq!(irc.get_inner().get_ref(), b"PONG :123456789\r\n");
}
#[test]
fn test_irc_join() {
	let mut irc = IrcWriter::new(MemWriter::new());
	assert_eq!(irc.join("#testchannel"), Ok(()));
	assert_eq!(irc.get_inner().get_ref(), b"JOIN :#testchannel\r\n");
}
#[test]
fn test_irc_quit() {
	let mut irc = IrcWriter::new(MemWriter::new());
	assert_eq!(irc.quit(), Ok(()));
	assert_eq!(irc.get_inner().get_ref(), b"QUIT\r\n");
}