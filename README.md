# API Template Generator

This tool helps to generate client/server code from OpenAPI documents using templates allowing for easy customisation.

There is still quite a lot to do make it feature complete but it can already help generate the code as it stands.

To see the list of to do features visit [TODO.md](TODO.md)


## Usage

**Command Overview:**

* [`api-gen`↴](#api-gen)
* [`api-gen generate`↴](#api-gen-generate)
* [`api-gen init`↴](#api-gen-init)

## `api-gen`

**Usage:** `api-gen <COMMAND>`

###### **Subcommands:**

* `generate` — Generate based off the template
* `init` — Initialize a new project



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
