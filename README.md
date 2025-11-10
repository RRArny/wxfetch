# WXfetch

![CI](https://github.com/rrarny/wxfetch/actions/workflows/rust.yml/badge.svg) ![Crates.io](https://img.shields.io/crates/v/wxfetch) ![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)

Console utility for accessing aviation weather information from the command line.

## Parameters

If you provide no flags to WXfetch at all, it will try and fetch weather info from your closest airfield according to your IP based position (geoip).

With `-a` or `--airfield` you can provide the ICAO or IATA code for a reporting station. Alternatively, with `--lat` and `--lon` you can provide geographical coordinates. WXfetch will then try and find a reporting station close to that position. Please make sure to provide both parameters.

With `-c` or `--config-file` you can specify a configuration file as described below.

If you provide the `-f` or `--file` flag followed by a path to a json file, WXfetch will try to load the metar from this instead of from the internet. It expects the data to be formatted as described [here](https://avwx.docs.apiary.io/#reference/0/metar/get-metar-report).

If there is any problem with the provided arguments WXfetch will print an error message and default to geoip.

If the `--taf` flag is present, instead of a METAR, a TAF (Terminal Aerodrome Forecast) is printed.

## Configuration

The configuration is loaded from `~/.config/wxfetch/config.toml` or from a TOML-file as specified by the `-c` flag. If no configuration file is found it will default to sensible values.

Except for position, these options present personal minima and reflect the colours used. For instance a cloud layer with an altitude lower than the specified minimum will be rendered red.

It is advisable to just copy and modify the sample file from the git repository. The program expects the file to be structured as follows:

### Position

  - `airfield`: ICAO or IATA code of the reporting aerodrome.
  - `lat` & `lon`: Latitude and longitude. The program will look for the nearest reporting station to these coordinates.

If none of the options above are supplied the program defaults to geoip. Positions supplied as command line parameters override the options from the config file.

### Clouds

  - `cloud_minimum`: Minimum altitude for cloud layers in hectofeet (similar to flight levels).
  - `cloud_marginal`: Altitude at which cloud layers will still be considered marginal.
  
### Temperature

  - `temp_minimum`: Minimum temperature in °C.
  - `spread_minimum`: Minimum spread.

### Wind

  - `wind_var_maximum`: Maximum wind variability in degrees.
  - `wind_maximum`: Maximum wind speed.
  - `gust_maximum`: Maximum gust difference.
  
### Age

  - `age_maximum`: Maximum age of the report in seconds.
  - `age_marginal`: Marginal age of the report.
  
### Visibility

  - `visibility_minimum`: Minimum visibility in meters.
  - `visibility_marginal`: Marginal visibility.
  
## Providing API keys

For wxfetch to work you will need a free account on https://avwx.rest/. Once you have created an account, go ahead and set the environment variable `AVWX_API_KEY` to your API key. Alternatively, you can provide a key with the `-k` or `--key` flag.

# Building from source

Wxfetch is written in Rust. In order to build it, run `cargo build` for a debug build, for a production build run `cargo build --release`. This will generate a binary file within the `target` directory in the `debug` and `release` subdirectories respectively.

For working on Wxfetch this repository includes a bacon configuration. Run `bacon` to have a variety of jobs at your disposal. `bacon clippy` (or using the 'c' key) will run the clippy linter at a pedantic level. `bacon test` (or 't') will run the unit tests. `bacon tarpaulin` (or 'alt-t') will calculate the code coverage using the `tarpaulin` plugin. You can install bacon using `cargo install bacon` and tarpaulin with `cargo install cargo-tarpaulin`.

# Contributing

This project is open source. If you would like to contribute, just fork the project, make your changes and create a pull request with a short description. I will only accept pull requests that satisfy the following criteria:
- all unit tests are passing,
- code coverage is above 50%,
- all pedantic clippy hints that still occur are explained with a comment (if they are impossible or impractical to fix, I will also be pedantic with this!),
- I deem the contribution worthwhile.
