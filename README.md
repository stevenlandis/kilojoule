# KiloJoule

`kj` is a command line tool that aims to make it as easy as possible to interact with the shell.

The name means nothing and was chosen because it is easy to type and doesn't conflict with any other popular cli commands that I'm aware of.

This tool is heavily inspired by the fantastic [`jq`](https://github.com/jqlang/jq). If you are already familiar with `jq`, here is a short list of differences between the two tools.

- `jq` is primarily focused on processing streams of JSON. `kj` is more focused on text and shell interactions.
- `kj` does not support streams which simplifies certain interactions.

### Status

This project is an early MVP and the main focus is developing the core expression syntax and features.

This is a hobby project for me so I might be slow to respond to bugs or feature requests.

While the intention is to keep the syntax stable, the language will change as it evolves and there are no guarantees that scripts that run today will continue working in the future.

This project isn't released anywhere and the only way to install it is to download the source. I'd mainly like to compile the project as a single static binary before making this more accessible.

I currently installed `kj` on my system with the following bash function:

```sh
function kj() {
  /path/to/python/binary /path/to/kilojoule/main.py $@
}
```

### Examples

`kj` is primarily intended to be called from the shell and inherits much of its syntax from the shell.

```sh
# Pipe the number "42" to the "." echo command wich simply prints out the input
> kj '42 | .'
# 42

# Access a field on a dict
> kj '{a: 1, b: 2} | .b'
# 2

# Access an element from a list
> kj '[100, 200, 300] | .[2]'
# 200

# Map a function over an list
> kj '[100, 200, 300] | map . + 1'
# [101, 201, 301]

# Filter elements of a list
> kj '[1, 2, 3, 4, 5] | filter . > 3'
# [4, 5]

# Group elements
> kj '[1,2,3,1,3,4] | group [., .]'
# [
#   {
#     "key":1,
#     "rows":[1,  1]
#   },
#   {
#     "key":2,
#     "rows":[2]
#   },
#   {
#     "key":3,
#     "rows":[3,3]
#   },
#   {
#     "key":4,
#     "rows":[4]
#   }
# ]

# Remove duplicate values
> kj '[1,2,3,1,3,4] | group [., .key]'
# [1, 2, 3, 4]

# Print out directories in the PATH
> kj '(env).PATH | split ":" | group [., .key] | sort'
# [
#   "/bin",
#   "/home/user/.cargo/bin",
#   "/home/user/.local/bin",
#   "/home/user/.nodenv/shims",
#   "/home/user/.poetry/bin",
#   "/home/user/.yarn/bin",
#   "/home/user/bin",
#   "/snap/bin",
#   "/usr/bin",
# ]

# deduplicate paths in PATH
# while preserving order of first appearance
> export PATH=$(kj '(env).PATH | split ":" | group [., .key] | join ":" | out')
```

Pipes (`|`) are a critical part of the language because they make it possible to iteratively write queries. Consider the changes required to filter an expression:

```
expr | filter .value
    ^^^^^^^^^^^^^^^^

filter(expr, .value)
^^^^^^^    ^^^^^^^^^

(filter expr, .value)
^^^^^^^    ^^^^^^^^^

[elem for elem in expr if elem.value]
^^^^^^^^^^^^^^^^^^    ^^^^^^^^^^^^^^^
```

Pipes are the only way to avoid having to add characters to the left of the expression which is especially difficult to do in the shell. The result is that expressions can be developed one "stage" at a time.

```sh
# pipe a shell command into kj
> ls | kj 'in | lines'
# [
#   "main.py",
#   "poetry.lock",
#   "pyproject.toml",
#   "README.md",
#   "src",
#   "tests",
#   "todo.md"
# ]

# Or call a shell command in kj
> kj 'exec ["ls"] | lines'

# Or pass a kj string into another command
> kj 'exec ["ls"] | lines | filter isfile | joinlines | exec ["wc", "-l"]'
"5\n"
```

There are a ton more functions and features so take a look at the tests `tests/test_run_json_query_expr.py` for a fairly comprehensive list of examples.
While tests aren't the best form of documentation, they are guaranteed to always be correct.

### Setup and Testing

This package uses [`poetry`](https://python-poetry.org/) to manage dependencies.

Most features should be thoroughly tested and test can be run with `poetry run pytest`.

### Folder Structure

```
main.py # entry point for the cli command
src/
  parser_generator.py # An LR(1) parser generator
  parser.py # Definition of the grammar for the language
  run_json_query_expr.py # Implementation of the VM that runs commands
  ...
tests/ # All tests go here
```
