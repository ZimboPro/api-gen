# Command-Line Help for `api-gen`

This document contains the help content for the `api-gen` command-line program.

**Command Overview:**

* [`api-gen`↴](#api-gen)
* [`api-gen generate`↴](#api-gen-generate)
* [`api-gen init`↴](#api-gen-init)
* [`api-gen context`↴](#api-gen-context)

## `api-gen`

**Usage:** `api-gen <COMMAND>`

###### **Subcommands:**

* `generate` — Generate based off the template
* `init` — Initialize a new project
* `context` — Outputs the context as JSON



## `api-gen generate`

Generate based off the template

**Usage:** `api-gen generate [OPTIONS] --api <API> --output <OUTPUT>`

###### **Options:**

* `-a`, `--api <API>` — OpenAPI file(s) to generate from. It can be a folder
* `-o`, `--output <OUTPUT>` — Output file
* `-c`, `--config <FILE>` — Sets a custom config file
* `-v`, `--verbose` — Verbose mode (-v, -vv, -vvv, etc.)
* `-q`, `--quiet` — Quiet mode, only displays warnings and errors



## `api-gen init`

Initialize a new project

**Usage:** `api-gen init`



## `api-gen context`

Outputs the context as JSON

**Usage:** `api-gen context [OPTIONS] --api <API>`

###### **Options:**

* `-a`, `--api <API>` — OpenAPI file(s) to generate from. It can be a folder
* `-c`, `--config <FILE>` — Sets a custom config file
* `-v`, `--verbose` — Verbose mode (-v, -vv, -vvv, etc.)
* `-q`, `--quiet` — Quiet mode, only displays warnings and errors



<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>

