# KiloJoule

`kj` is a command line tool that aims to make it as easy as possible to interact with the shell.

The name means nothing and was chosen because it is easy to type and doesn't conflict with any other popular cli commands that I'm aware of.

This tool is heavily inspired by the fantastic [`jq`](https://github.com/jqlang/jq). If you are already familiar with `jq`, here is a short list of differences between the two tools.

- `jq` is primarily focused on processing streams of JSON. `kj` is more focused on text and shell interactions.

### Status

This project is an early MVP and the main focus is developing the core expression syntax and features.

This is a hobby project for me so I might be slow to respond to bugs or feature requests.

While the intention is to keep the syntax stable, the language will change as it evolves and there are no guarantees that scripts that run today will continue working in the future.

You can build your own version of `kj` by cloning the repo, running `cargo build --release`, and then running the binary at `target/release/kilojoule`. It's a static binary so no need to worry about external dependencies or libraries.

I currently installed `kj` on my system with the following bash alias:

```sh
alias kj="/path/to/kilojoule/target/release/kilojoule"
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
> kj '[100, 200, 300] | map(. + 1)'
# [101, 201, 301]

# Filter elements of a list
> kj '[1, 2, 3, 4, 5] | filter(. > 3)'
# [4, 5]

# Group elements
> kj '[1,2,3,1,3,4] | group(.)'
# [
#   {
#     "key":1,
#     "vals":[1,  1]
#   },
#   {
#     "key":2,
#     "vals":[2]
#   },
#   {
#     "key":3,
#     "vals":[3,3]
#   },
#   {
#     "key":4,
#     "vals":[4]
#   }
# ]

# Remove duplicate values
> kj '[1,2,3,1,3,4] | group(.) | map(.key)'
# [1, 2, 3, 4]

# Or just use the pre-built unique() function
> kj '[1,2,3,1,3,4] | unique()'
# [1, 2, 3, 4]

# Print out directories in the PATH
> kj 'env().PATH | split(":") | unique() | sort()'
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
> export PATH=$(kj 'env().PATH | split(":") | unique() | join(":") | bytes()')
```

Pipes (`|`) are a critical part of the language because they make it possible to iteratively write queries. Consider the changes required to filter an expression:

```
expr | filter(.value)
    ^^^^^^^^^^^^^^^^^

filter(expr, .value)
^^^^^^^    ^^^^^^^^^

(filter expr, .value)
^^^^^^^^    ^^^^^^^^^

[elem for elem in expr if elem.value]
^^^^^^^^^^^^^^^^^^    ^^^^^^^^^^^^^^^
```

Pipes are the only way to avoid having to add characters to the left of the expression which is especially difficult to do in the shell. The result is that expressions can be developed one "stage" at a time.

```sh
# pipe a shell command into kj
> ls | kj 'in() | lines()'
# [
#   "Cargo.lock",
#   "Cargo.toml",
#   "LICENSE",
#   "README.md",
#   "src",
#   "target",
#   "tests"
# ]

# Or call a shell command in kj
> kj 'call("ls") | lines()'

# Or pass a kj string into another command
> kj 'call("ls") | lines() | filter(len() > 5) | join_lines() | call("wc", "-l")'
"5\n"
```

There are a ton more functions and features so take a look at the tests `tests/test_parse_and_eval.rs` for a fairly comprehensive list of examples.
While tests aren't the best form of documentation, they are guaranteed to always be correct.

### Setup and Testing

Most features should be tested and test can be run with `cargo test`.

### Folder Structure

```
src/
  main.rs -- The entry point of the executable
  parser.rs -- The parser
  val.rs -- The value type representing all objects
tests/ # All tests go here
```
