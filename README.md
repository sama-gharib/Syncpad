# Syncpad
Syncpad is a simple terminal text editor.

Syncpad is still a work in progress as it doesn't yet support tabulations (default behaviour is to overwrite them by spaces but this is kind of a horrible hack).

# How to use ?
Use `cargo build --release` to build an executable at `target/release/syncpad`.

Execute the editor with `./syncpad [insert file path here]`.

## Key bindings
 - Ctrl+S .......... Save
 - Ctrl+C .......... Quit
 - Ctrl+Shift+C .... Copy selection
 - Ctrl+Shift+V .... Paste selection
 - Fn+Right Arrow .. Go to end of current line
 - Fn+Left Arrow ... Go to start of current line  

# Roadmap
  - Fix non printable characters insertion
  - Fix tab handling
  - Add modal editor client with vim-like bindings
  - Add basic server/client for text editing in team
