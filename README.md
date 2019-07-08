# kak-recent-files
WIP kakoune plugin for managing recent files

This plugin is sort of session management, sort of easy buffer switching. I've written most of the
code in Rust, since that's what I'm most comfortable with.

If you want to use this plugin, file an issue! It's currently fairly specialized to my work flow,
and could use with some good generalization if anyone else is interested in using it.

### Overview

For each session, kak-recent-files keeps a sorted set of files in the current directory, ordered by
most recently used. The "using" here is switching to a kakoune buffer with the file open.

For named sessions, this set is kept as a newline-delimited list in
`~/.local/share/kak-recent-files/$session_name`. It will be initially populated with all
non-gitignored files in the directory kakoune was first opened in.

For unnamed sessions, or sessions with only numbers for their name, the set is kept in kakoune's
memory and will not be auto-populated.

When you load `recent-files.kak`, you enable:

- a hook to add files on WinDisplay (any change of what buffer is displayed)
- a hook on startup which, if in a named session, will open the last opened file from the last time
  this session was created
- various commands:
  - `krf-open-menu` opens `rofi` (or configured command) selecting from the recent file set
  - `krf-delete-file` removes a particular file from the set
  - `krf-reset` resets the set of known files

### Options

- `krf_use_temp_storage`

  true if using temp storage, false if storing on disk.
  set on buffer startpu

- `krf_menu_command`

  this is the command that will be executed in `krf-open-menu`. the recent file list will be passed
  in via stdin, with the most recent first. the command should output either no lines, or one line
  containing one of the input filenames.
