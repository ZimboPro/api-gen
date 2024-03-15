# API Template Generator

This tool helps to generate client/server code from OpenAPI documents using templates allowing for easy customisation.

There is still quite a lot to do make it feature complete but it can already help generate the code as it stands.

To see the list of to do features visit [TODO.md](TODO.md) (The list will probably not be an exhaustive list)

## NOTE

Currently it only supports OpenAPI v3.0.x

## Usage

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


## Template structure

A `templates` folder needs to exists and within it any file found will be rendered. However, any file starting with `_` eg `_fileName` will be ignored.

There are reserved file names such as `model` and `model-endpoint`. The extension is not considered so files names such as `model.rs`, `model.dart` etc will all be part of the reserved files. The output file names will be based on the output of the `modelFileName` variable in the config and will be rendered as well eg `modelFileName: "{{object_name | snake_case}}.dart"`


### `model` Reserved File

This will render and generate a file for each object in both the response and request. The data available will be the object.

### `model-endpoint` Reserved File

This will render and generate a file for each endpoint in both the response and request with all the classes/objects for that request/response grouped together in the file. The data available will be the object. The variable `models` will be an array of all the related data structures.

### Available variables

In the reserved files, the following variables are available:

 * file_name : This will be the resulting file name that was rendered.
 * models : Only available in th `model-endpoint` reserved file and will be an array of all the related data structures for that respective endpoint

## Template Functions

The template used is [Tera](https://keats.github.io/tera) which is inspired by Django and Jinja2. The available functions provided by the engine is available [here](https://keats.github.io/tera/docs/#templates).

### String Formatting

You can change the text from one style to another with the following:
* camel_case - Converts text into CamelCase.
* kebab_case - Converts text into kebab-case.
* lower_case - Converts text into lowercase.
* mixed_case - Converts text into mixedCase.
* snake_case - Converts text into snake_case.
* title_case - Converts text into Title Case.
* upper_case - Converts text into UPPERCASE.

eg `{{ name | camel_case }}` to change the name to CamelCase

### JSON Output

It is possible to output JSON for the request or response body. NOTE It has to be the root body

 * json_typing - Output the JSON with the typing provided in the config eg. `{{json_typing(structure = endpoint.request) | json_encode() | safe }}`
 * json_value - Output the JSON with random data with the full structure including optional fields eg. `{{json_value(structure = endpoint.request) | json_encode() | safe }}`
 * json_min_value - Output the JSON with random data with only the required fields eg. `{{json_min_value(structure = endpoint.request) | json_encode() | safe }}`

### Extra Template functions

 * map_type - Map the OpenAPI type to what was configured eg. `{{map_type(type = field )}}`
 * extended - Get a value in the extended section of the config `{{extended(key="feature")}}`
 * exists - Checks if a key exists in the extended section of the config `{% if exists(key="feature") %}`
