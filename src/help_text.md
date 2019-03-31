Using XV
========

XV is a hex viewer. There is no editing capability.
The file is displayed as a grid of bytes. The width
of the grid is 16 bytes by default, and can be
changed by pressing `w`.

Navigating
----------

The viewport is moved around the file with the
`h`, `j`, `k`, and `l` keys, or the arrow keys.

Pressing `H` (shift-h), or pressing the Home key,
moves the viewport to the left-most edge. And
pressing `L` or End moves the viewport to the
right-most edge.

Pressing `J` or Page Down, moves the viewport one
whole screen down, and pressing `K` or Page Up
moves the viewport one whole screen up.

Press `g` to open the "Go to" dialog, and jump to
arbitrary rows and columns.

Opening files
-------------

Press `o` to open the "Open file" dialog. The
current working directory is shown at the top.
Folders are on the left, and files are on the
right.

Press any letter key with the directory or file
list in focus, to jump to the directories or files
whose names start with that letter.

Press `s` to open the "Switch file" dialog. This
dialog lets you switch between recently opened
files. The last file you had open will be at the
top of the list.

The list of recently opened files is remembered
across restarts of XV.

Press Del in the "Switch file" dialog to remove a
file from the list. This will also forget the
remembered line-width and viewport location.

Other features
--------------

Press `v` to switch the "visual" (text) column
between showing unicode replacement symbols, ASCII
replacement symbols, or not showing the visual
column at all.

Press `t` to switch between light and dark theme.
The theme selection is remembered across restarts.

Press the Esc key to close any dialog.

Press `q` to quit the program. This works even when
a dialog is open. Pressing Esc also quits if no
dialog is open.

Press `r` to reload the data in the viewport.
Press `R` to re-open the file, and then reload the
data.

Press `?` or F1 to show this help text.
