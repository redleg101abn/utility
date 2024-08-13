# Nukem

![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)
![Build Status](https://img.shields.io/github/actions/workflow/status/username/repo/CI.yml?branch=main)
![Version](https://img.shields.io/badge/version-0.26.0--alpha-blue)

Nukem is a command-line tool designed to delete massive amounts of files and directories efficiently. It supports various options such as dry-run mode, specifying number of threads and buffers, and verbose logging to the console or to a logfile. You can provide multiple paths, and it will process each according to the specified options.

## Table of Contents
- [Installation](#installation)
- [Usage](#usage)
- [Contributing](#contributing)
- [License](#license)
- [Authors and Acknowledgements](#authors-and-acknowledgements)
- [Support](#support)
- [Roadmap](#roadmap)

## Installation

### Prerequisites
- Rust and Cargo installed. You can install Rust using [rustup](https://rustup.rs/).

### Steps
1. Clone the repository:
    ```sh
    git clone https://github.com/yourusername/nukem.git
    ```
2. Change to the project directory:
    ```sh
    cd nukem
    ```
3. Build the project:
    ```sh
    cargo build --release
    ```
4. The binary will be located in the `target/release` directory.

## Usage

### Basic Usage
To delete files and directories:
```sh
./nukem /path/to/delete
