use std::io;
use std::io::{ Read, Write };
use std::fs::File;

use editor::{ Editor, Displacement, Cursor, Command };

use crossterm::terminal::{
    enable_raw_mode,
    disable_raw_mode,
    Clear,
    ClearType
};
use crossterm::cursor::MoveTo;
use crossterm::style::Print;
use crossterm::execute;
use crossterm::terminal;

mod editor;

struct RawModeHandle;
impl Drop for RawModeHandle {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
    }
}
impl RawModeHandle {
    pub fn new() -> Self {
        enable_raw_mode()
            .expect("Your terminal does not support raw mode");
        Self
    }
}

fn main() -> Result<(), &'static str>{
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        return Err("Please provide exactly one argument (file path).");
    }

    let mut content = String::new();

    {
        let mut file_stream = File::options()
            .read(true)
            .write(true)
            .create(true)
            .open(&args[1])
            .expect("Unable to open file.");
        
        file_stream.read_to_string(&mut content)
            .expect("Unable to read file.");
    }


    let _raw_mode = RawModeHandle::new();

    let mut editor = Editor::from(&content[..]);
    let mut save_status = String::from("Not modified");

    loop {

        let scroll = (editor.get_cursor().line  - (terminal::size().unwrap().1-4) as isize).max(0);
        let unsigned_cursor = editor.get_unsigned_cursor();
        let column = u16::try_from(unsigned_cursor.0).unwrap_or(u16::MAX);
        let line   = u16::try_from(unsigned_cursor.1).unwrap_or(u16::MAX);

        let _ = execute!(
            io::stdout(),
            Clear(ClearType::All),
            MoveTo(0, 0),
            Print(
                (scroll..(scroll + terminal::size().unwrap().1 as isize - 3))
                    .map(|index| editor.get_line_display(index as isize, terminal::size().unwrap().0 as usize))
                    .reduce(|x, y| format!("{x}\r\n{y}"))
                    .unwrap()
            ),
            MoveTo(0, terminal::size().unwrap().1 - 3),
            Print((0..terminal::size().unwrap().0).map(|_| '_').collect::<String>()),
            MoveTo(0, terminal::size().unwrap().1 - 2),
            Print("Exit: Ctrl+C | Save: Ctrl+S | Copy: Ctrl+Shift+C | Paste: Ctrl+Shift+V"),
            MoveTo(0, terminal::size().unwrap().1),
            Print(&save_status),
            MoveTo(column, line.min(terminal::size().unwrap().1 - 4))
        );

        let mut keypress = [0u8; 4];
        io::stdin().read(&mut keypress)
            .expect("Could not read from stdin.");

        match keypress {
            [26 | 3, 0, 0, 0] => break,
            [27, 91, 65..69, 0] => {
                // Arrow key
                let direction = match keypress[2] {
                    65 => Displacement { line: -1, column:  0 },
                    66 => Displacement { line:  1, column:  0 },
                    67 => Displacement { line:  0, column:  1 },
                    _  => Displacement { line:  0, column: -1 }
                };

                editor.apply_command(Command::Offset(direction));
            },
            [127, 0, 0, 0] => {
                editor.execute(
                    &[
                        Command::Backspace,
                        Command::Offset (
                            Displacement {
                                line: 0, column: -1
                            }
                        )
                    ]
                );
            },
            [13, 0, 0, 0] => {
                editor.execute(
                    &[
                        Command::BreakLine,
                        Command::Offset (
                            Displacement {
                                line: 1, column: 0
                            }
                        ),
                        Command::LineStart
                    ]
                );
            },
            [27, 91, 72, 0] => {
                editor.apply_command(Command::LineStart);
            },
            [27, 91, 70, 0] => {
                editor.apply_command(Command::LineEnd);
            },
            [9, 0, 0, 0] => {
                insertion(String::from("    "), &mut editor);
            },
            [19, 0, 0, 0] => {

               let file_stream = File::options()
                    .read(false)
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(&args[1]);

                if let Ok(mut file_stream) = file_stream {                
                    if let Err(_) = file_stream.write(format!("{editor}").as_bytes()) {
                        let _ = io::stderr()
                        .write(
                            format!("Could not save file !\n")
                                .as_bytes()
                        );
                        save_status = String::from("Error while saving : Could not write in file.");
                    } else {
                        save_status = String::from("Saved !");
                    }
                } else {
                    let _ = io::stderr()
                    .write(
                        format!("Could not save file !\n")
                            .as_bytes()
                    );
                    save_status = String::from("Error while saving : Could not open file.");
                }
            }
            _ => {
                // io::stderr().write(format!("{keypress:?}\n").as_bytes());
                save_status = String::from("Not saved.");
                let to_insert: String = std::str::from_utf8(&keypress)
                    .unwrap_or("_")
                    .chars()
                    .filter(|x| *x != '\0')
                    .collect();
                insertion(to_insert, &mut editor);
            }
        }
    }

    let _ = disable_raw_mode();
    
    let window_size = terminal::size().unwrap();
    let _ = execute!(
        io::stdout(),
        MoveTo(window_size.0, window_size.1)
    );
    println!("\r\nFinished !");

    Ok(())
}

fn insertion(to_insert: String, editor: &mut Editor) {

    editor.execute(
        &[
            Command::Insert ( to_insert.clone() ),
            Command::Offset (
                Displacement {
                    line: 0,
                    column: to_insert.len() as isize
                }
            )
        ]
    );
}