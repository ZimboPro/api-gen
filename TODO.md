# TODO

## General

- [ ] Documentation
- [ ] Unit tests
- [ ] Determine if there is a better way to map types (preprocess the mapped types?)
- [ ] Document/Generate available data in the context
- [ ] Self update
- [ ] Caching
    - [ ] Hashing
- [ ] Benchmarking
    - [ ] Large files
        - [ ] Pet store
    - [ ] GHA scenario

## Data to be extracted/generated from OpenAPI

- [x] Check if request body is required
- [x] Generate request data
- [ ] Generate path parameter
- [ ] Generate query parameter
- [x] Base urls
- [x] Patterns
- [x] Validation rules (min, max, max length etc)
    - [x] Min
    - [x] Max
    - [x] Min length
    - [x] Max length

## Generation

- [ ] Test different scenarios
    - [x] Array of strings
- [x] Specify output
- [x] Generate multiple templates
- [x] Generate multiple output files
    - [x] classes/models
        - [x] if model template exists
        - [x] file names
- [ ] Tera macros
    - [ ] Property layout
    - [ ] Array layout
    - [ ] Comments
- [ ] Tera include files

## Initialisation

- [ ] init mode
    - [x] create config
    - [x] create templates dir
    - [ ] generate templates based on lang/option

## Config

- [x] Config
    - [x] Property typing
    - [x] Extended data

## Templates

- [ ] Query parameters example
- [ ] Rust example
- [ ] TS example
- [ ] Python example
