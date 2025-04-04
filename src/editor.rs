//! # What ? 
//! Everything regarding file edition. `Editor` provides a safe non-panicking API to edit an UTF-8 text.


//! # Exemple

//! ```
//! let mut e = Editor::from("Some multiline text\nThat says something.");

//! e.execute(
//! 	&[
//! 		Command::Goto (
//! 			Cursor { column: 3, line: 0 }
//! 		),
//! 		Command::Insert ('€'), // Accepts UTF-8
//! 		Command::Offset (
//! 			Cursor { column: 100, line: 1 } // Cursor gets placed at nearest valid position
//! 		),
//! 		Command::AppendLineTo (0),
//! 		Command::DeleteLine
//! 	]
//! );

//! println!("{e}"); // 'Som€e multiline textThat says something.

//! ```

use std::fmt;

type Line = Vec<char>;
type Text = Vec<Line>;

#[derive(Debug, Copy, Clone)]
pub struct Cursor {
	pub column: isize,
	pub line: isize
}

pub type Displacement = Cursor;

#[derive(Debug, Clone)]
pub enum Command {
	// Edition 
	Insert (String),
	Delete,
	Backspace,
	AppendLineTo (isize),
	DeleteLine,
	BreakLine,

	// Cursor
	Goto (Cursor),
	Offset (Displacement),
	LineStart,
	LineEnd,

	// History
	Undo,
	Redo

}

pub struct Editor {
	cursor: Cursor,
	text: Text

}

impl From<&str> for Editor {
	fn from(s: &str) -> Self {
		let mut result = Self::default();
			
		result.text = s
			.split("\n")
			.map(|x|
				x
				 .chars()
				 .collect::<Line>()
			)
			.collect();

		result
	}
}

impl fmt::Display for Editor {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
		write!(
			f,
			"{}",
			self.text
				.iter()
				.map(|x|
					x
					 .iter()
					 .collect::<String>()
				)
				.reduce(|x, y|
					format!("{x}\r\n{y}")
				)
				.unwrap_or(
					String::from("Could not reduce text !")
				)
		)
	}
}

impl Default for Editor {
	fn default() -> Self {
		Self {
			cursor: Cursor {
				column: 0,
				line: 0
			},
			text: Text::default()
		}
	}
}

impl Editor {
	
	// Getters

	pub fn get_line_display(&self, line: isize, width: usize) -> String {
		let line = if line > -1 && line < self.get_line_count() {
				self.text[line as usize].clone()
			} else {
				vec!['~']
			};

		if line.len() < width {
			line.iter().collect()
		} else {
			let remainder = line.len() % width;

			format!(
				"<{}",
				line
				.iter()
				.skip(line.len() - remainder)
				.collect::<String>()
			)
		}
	}

	pub fn get_cursor(&self) -> Cursor {
		self.cursor
	}

	pub fn get_unsigned_cursor(&mut self) -> (usize, usize) {
		self.validate_cursor();
		(
			self.cursor.column as usize,
			self.cursor.line as usize
		)
	}

	pub fn get_line_count(&self) -> isize {
		self.text.len() as isize
	}

	pub fn get_line_length(&self) -> Option<isize> {
		Some(
			self.text.get(
				usize::try_from(self.cursor.line).unwrap_or(0)
			)?
			.len() as isize
		)
	}

	fn get_line(&mut self, line: isize) -> Option<&mut Line> {
		self.text.get_mut(usize::try_from(line).ok()?)
	}

	fn get_current_line(&mut self) -> Option<&mut Line> {
		self.get_line(self.cursor.line)
	}

	//

	pub fn validate_cursor(&mut self) {
		self.cursor.line   = Self::bounds(self.cursor.line, 0, (self.get_line_count()-1).max(0));
		self.cursor.column = Self::bounds(self.cursor.column, 0, self.get_line_length().unwrap_or(0));
	}

	fn bounds(value: isize, min: isize, max: isize) -> isize {
		value.min(max).max(min)
	}

	pub fn execute(&mut self, program: &[Command]) {
		program.iter().for_each(|x| self.apply_command(x.clone()));
	}

	pub fn apply_command(&mut self, command: Command) {
		match command {
			Command::Insert (c)       => self.insert(c),
			Command::Delete           => self.delete(),
			Command::Backspace        => self.backspace(),
			Command::AppendLineTo (l) => self.append_line_to(l),
			Command::DeleteLine       => self.delete_line(),
			Command::BreakLine        => self.break_line(),
			Command::Goto (c)         => self.cursor_goto(c),
			Command::Offset (d)       => self.cursor_offset(d),
			Command::LineStart        => self.line_start(),
			Command::LineEnd          => self.line_end(),
			Command::Undo             => self.undo(),
			Command::Redo             => self.redo()
		}
	}

	fn insert(&mut self, c: String) {
		self.validate_cursor();
		
		let column = self.cursor.column as usize;

		let line = self.get_current_line()
			.unwrap();

		for (i, character) in c.chars().enumerate() {
			line.insert(column + i, character);
		}
	}
	fn delete(&mut self) {
		todo!();
	}
	fn backspace(&mut self) {
		self.validate_cursor();

		if self.cursor.column == 0 && self.cursor.line != 0{
			
			let old_length = self.get_line(self.cursor.line - 1)
						.unwrap()
						.len() as isize + 1;
			self.append_line_to(self.cursor.line - 1);
			self.delete_line();
			self.cursor_offset(
				Displacement {
					line: -1,
					column: old_length
				}
			);
			
		} else if self.cursor.column != 0 {
			let column = self.cursor.column as usize;
			self.get_current_line()
				.unwrap()
				.remove(
					column - 1
				);
		}
	}
	fn append_line_to(&mut self, l: isize) {
		self.validate_cursor();

		let to_append = self.get_current_line()
			.unwrap()
			.clone();

		if let Some(line) = self.get_line(l) {
			line.extend_from_slice(&to_append);
		}
	}
	fn delete_line(&mut self) {
		self.validate_cursor();

		self.text.remove(self.cursor.line as usize);
	}

	fn break_line(&mut self) {
		self.validate_cursor();

		let column = self.cursor.column as usize;
		
		let left  : Vec<char>;
		let right : Vec<char>;
		{
			let line = self.get_current_line().unwrap();
			let split = line.split_at(column);
			left  = split.0.iter().map(|x| *x).collect();
			right = split.1.iter().map(|x| *x).collect();

			*line = right;
		}

		let place = self.cursor.line as usize;

		self.text.insert(
			place,
			left
		);
	}

	fn cursor_goto(&mut self, c: Cursor) {
		self.cursor = c;
	}

	fn cursor_offset(&mut self, d: Displacement) {
		self.cursor.line += d.line;
		self.cursor.column += d.column;
	}

	fn line_start(&mut self) {
		self.cursor.column = 0;
	}

	fn line_end(&mut self) {
		if let Some(line) = self.get_current_line() {
			self.cursor.column = line.len() as isize;
		}
	}

	fn undo(&mut self, ) {
		todo!();
	}

	fn redo(&mut self, ) {
		todo!();
	}
}