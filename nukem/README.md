# Nukem

![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)
![Build Status](https://img.shields.io/github/actions/workflow/status/redleg101abn/utility/blank.yml?branch=main)
![Version](https://img.shields.io/badge/version-0.26.0--alpha-blue)

Nukem is a command-line tool designed to delete massive amounts of files and directories efficiently. 
It supports various options such as dry-run mode, specifying number of threads and buffers, and 
verbose logging to the console or to a logfile. You can provide multiple paths, and it will process 
each according to the specified options.

## Table of Contents
- [Installation](#installation)
- [Usage](#usage)
- [Miscellaneous Information](#misc)

## Installation

### Prerequisites
- Rust and Cargo installed. You can install Rust using [rustup](https://rustup.rs/).
- For portability reasons, we recommend creating a statically-linked binary using [MUSL](https://musl.libc.org/).

### Steps
1. Clone the repository:
    ```sh
    git clone https://github.com/redleg101abn/utility.git
    ```
2. Change to the project directory:
    ```sh
    cd nukem
    ```
3. Build the project on Linux x86_64:
    ```sh
    cargo build --target=x86_64-unknown-linux-musl --release
    ```
4. The binary will be located in the `target/x86_64-unknown-linux-musl/release` directory.
5. Place the binary in a directory that's in your path, for example, /usr/local/bin/

## Usage

### Basic Usage
The only requirement when running the application is to give it the 
location of the data you want to delete. This can be an entire directory, a single
file, or a group of files, and can contain wildcard characters.
```sh
nukem /path/to/delete
```

### Basic Usage Examples
Nukem has been designed to conform to normal Linux conventions, concerning paths. For example,
both of these commands will delete everything inside the root directory but not the root
directory itself:
```sh
nukem /path/to/delete/
nukem /path/to/delete/*
```
To delete an entire directory:
```sh
nukem /path/to/delete
```
Delete an individual file:
```sh 
nukem foo.bar
nukem /path/to/delete/foo.bar
```
With wildcards:
```sh
nukem /path/to/delete/*.bar
nukem /path/to/delete/foo.*
nukem /path/to/delete/fo*
```
Also, multiple directories/files are allowed:
```sh
nukem /path/to/delete1 /path/to/delete2/ /path/to/delete3/foo.*
```

### Options
Nukem allows these command line options:

-l, --logfile_path <LOGFILE_PATH>
Full path of the directory for the logfile

-t, --threads <THREADS>
Number of threads to use for file and directory deletion. It cannot be zero or greater than 64

-v, --verbose
Enable verbose logging

-b, --buffer <BUFFER_SIZE>
Number of buffers to use for file and directory deletion. Allowable values are between 100 and 2000

-d, --dry-run
Perform a dry run without deleting any files or directories

-h, --help
Print help (see a summary with '-h')

-V, --version
Print version

### Runtime Tuning
Nukem is a multi-threaded application that uses concurrent workers. At runtime the user
can specify the number of threads to spawn and the number of buffers to use. These are 
specified by '-t' and '-b' respectively. If these values are not specified by the user,
the application will use default values.

**Threads:**

_Threads_ 

The user can specify a number between 1 and 64. That number of worker threads will be spawned,
plus one extra thread that is dedicated to logging functionality.

If the user does not specify a number of threads, the application will automatically compute
the optimal number of threads by multiplying the number of physical CPU cores by 10 (which is
a safe but good-performing value) and then adding one extra thread that is dedicated to 
logging functionality.

_Buffers_

The buffers represent the number of simultaneous filesystem objects that the application can work with.

The user can specify a number between 100 and 2000. In our testing with a high-performance
SAN filesystem, the optimal number was 250.

If the user does not specify a number of buffers, the default value of 100 will be used.

## Misc

* Symbolic links will be removed but not followed
* The application is always recursive (identical to the linux rm -r)
* Informational reports will be displayed during various phases of operation
* Generation of a log file is not required, but highly encouraged
