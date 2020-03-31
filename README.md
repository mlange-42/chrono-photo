# chrono-photo

Chronophotography command line tool and library in [Rust](https://www.rust-lang.org/).

![A simple Chronophotography example](https://user-images.githubusercontent.com/44003176/77975353-236da480-72fa-11ea-9ff9-5c110895fe5d.jpg)
<sup>_A simple Chronophotography example_</sup>

This tool creates chrono-photos like [Xavi Bou's "Ornithographies"](http://www.xavibou.com/) from video footage or photo series.

_Warning:_ I just started this project, and there is nothing useful is available yet. However, the image above shows a proof of concept for the intended algorithm. 

## Command line tool

### Installation

* Download the [latest binaries](https://github.com/mlange-42/chrono-photo/releases/latest).
* Unzip somewhere with write privileges (only required for running examples in place).

### Usage

* To view the full list of options, run `chrono-photo --help`

## Library / crate

To use this crate as a library, add the following to your `Cargo.toml` dependencies section:
```
chrono-photo = { git = "https://github.com/mlange-42/chrono-photo.git" }
```
