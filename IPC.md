# ugdb IPC

ugdb creates a unix domain socket at `$XDG_RUNTIME_DIR/ugdb/$RANDOM_CHARACTER_SEQUENCE` (or `/tmp/ugdb/$RANDOM_CHARACTER_SEQUENCE` if `$XDG_RUNTIME_DIR` is not set) which can be used to control ugdb from other applications.

## Message structure

The IPC message structure is inspired by the [i3 ipc format](https://i3wm.org/docs/ipc.html) but is not completely identical:
Each message starts with a 12 byte header of which the first 8 bytes are fixed to "ugdb-ipc" and the following 4 bytes describe the length of the message as a 32 bit little endian unsigned integer.
The message follows immediately after the header.

The message body itself is a utf8 string that encodes a json object where the exact structure depends on the type of message (see below).

## Requests

Requests have the fields `function` and `parameters` where the structure of `parameters` depends on the value selected for function.
Currently, 3 functions are available:

### `get_instance_info`

Get information about the ugdb instance that controls this socket.
Parameters are unused.

```json
{
    "function": "get_instance_info",
    "parameters": {}
}
```

Currently the response only contains the working directory of the gdb instance:

```json
{
    "type": "success",
    "result": {
        "working_directory": "/some/path"
    }
}
```
```

### `show_file`

Show the specified file and at the specified line in the ugdb pager.
Parameters are given as follows:

```json
{
    "function": "show_file",
    "parameters": {
        "file": "/path/to/some/file.c",
        "line": 42
    }
}
```

On success it returns a string that describes the action that was performed.

### `set_breakpoint`

Try to insert a breakpoint at the given line in the given file
Parameters are given as follows:
```json
{
    "function": "set_breakpoint",
    "parameters": {
        "file": "/path/to/some/file.c",
        "line": 42
    }
}
```

On success it returns a string that describes the action that was performed.

## Responses

Responses are objects that always contain a String describing the `type`.
The type is always either `success` or `error.

### Success

On success `result` contains either return information or info about the action that was performed.

```json
{
    "type": "success",
    "result": ...
}
```

### Error

An error contains the `reason` for the failure as well as `details` that can provide some context for interpreting the failure (e.g., malformed parameters will be returned to the sender as details).

```json
{
    "type": "error",
    "reason": "Malformed (non-object) request",
    "details": "{definitely not json"
}
```
