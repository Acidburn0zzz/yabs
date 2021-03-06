# yabs
[![Build Status](https://travis-ci.org/0X1A/yabs.svg?branch=master)](https://travis-ci.org/0X1A/yabs)

Yet another build system. A C and C++ build system, with projects described
using TOML.

**Note**: Yabs is still very much a work in progress and is subject to change.
Until yabs reaches version `1.0.0`, it should be assumed that yabs will continue 
to change.

## Get Yabs
To install yabs simply run `cargo install yabs`. If you are installing from 
crates.io please see the version of documentation available on docs.rs that corresponds to
your output of `yabs --version`. Otherwise, clone this repo and build the release target and add
yabs to your `$PATH`

## Building
To build `yabs` you will need `rustc` and `cargo`. Simply run `cargo build
--release`, this will build a binary called `yabs` in `target/release`.

# Using
Output of `yabs -h`

```
yabs 0.2.0
Alberto Corona <ac@albertocorona.com>
Yet another build system

USAGE:
    yabs [FLAGS] [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
        --sources    Print source files found by yabs or listed in 'src'
    -V, --version    Prints version information

SUBCOMMANDS:
    build    Build a binary or library, builds all binaries and libraries if no options are given
    clean    Removes all object files, binaries and libraries built by yabs
    help     Prints this message or the help of the given subcommand(s)
    new      Create a new yabs project
```

## Defining a Project
`yabs` uses TOML to define projects. For example, a project in C++ using libraries such as SDL2, SDL2-image, SDL2-ttf, and Lua 5.3 would look similar to the following:

```toml
[project]
name = "kuma"
file-extensions = ["cpp"]
compiler = "g++"
compiler-flags = ["std=c++14", "O0", "Wall", "Wextra", "g", "D_DEBUG_"]
include = ["src","`pkg-config --cflags sdl2 lua5.3`", "/usr/include/yaml-cpp", "third_party/sol2/single/sol"]
libraries = [
	"`pkg-config --libs sdl2 SDL2_image SDL2_ttf SDL2_mixer lua5.3 yaml-cpp`",
]
ignore = [
	"tests/",
	"third_party/",
]


[[bin]]
name = "kuma"
path = "./src/main.cpp"


[[lib]]
name = "libkuma"
types = ["static"]
```

### Building a Project
Currently `yabs` builds all targets listed in `[[bin]]` and `[[lib]]` sections

# Keys and Values
The following tables describes what keys are available to yabs project files.

### [project]
| Key    | Value                           | Type |
| ---    | -----                           | ---- |
| `name`   | Name for project                | String |
| `file-extensions`   | Extensions used for source files | Array |
| `version` | Version number | String |
| `compiler` | Compiler to use | String |
| `src` | Source files | Array |
| `libraries` | Libraries to link | Array |
| `librariy-directories` | Library directories to use | Array |
| `include` | Include directories | Array |
| `compiler-flags` | Compiler flags | Array |
| `linker-flags` | Linker flags | Array |
| `ignore` | Directories or files to ignore | Array |
| `before-script` | Scripts to run before a build | Array |
| `after-script` |  Scripts to run after a build | Array |
| `ar` | Archiving tool to use | String |
| `arflags` | Flags for archiving tool | Array |

### [[bin]]
| Key    | Value                           | Type |
| ---    | -----                           | ---- |
| `name` | Name and path for the binary | String |
| `path` | Path for the binary entry point (`main`) | String |

### [[lib]]
| Key    | Value                           | Type |
| ---    | -----                           | ---- |
| `name` | Name of library | String |
| `types` | Library types to create | Array |
