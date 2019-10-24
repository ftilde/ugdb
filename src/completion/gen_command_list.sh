#!/bin/sh

# Use this script to regenrate the gdb_commands.rs module, which contains a static
# list of possible gdb commands. This is required as (as far as I know) there is no
# way to query the list of available commands using gdbmi.

SCRIPT_DIR=$(dirname "$0")
FILE="$SCRIPT_DIR/gdb_commands.rs"

echo 'pub const GDB_COMMANDS: &[&str] = &[' > $FILE
gdb -batch -ex "help all" | grep '\-\-' | sed 's/^\(.*\) --.*$/    "\1",/' >> $FILE
echo '];' >> $FILE
