# WXfetch

Console utility for accessing aviation weather information from the command line.

## Parameters

If you provide no flags to WXfetch at all, it will try and fetch weather info from your closest airfield according to your IP based position (geoip).

With `-a` or `--airfield` you can provide the ICAO or IATA code for a reporting station. Alternatively, with `--lat` and `--lon` you can provide geographical coordinates. WXfetch will then try and find a reporting station close to that position. Please make sure to provide both parameters.

With `-c` or `--config-file` you can specify a configuration file as described below.

If there is any problem with the provided arguments WXfetch will print an error message and default to geoip.

## Configuration

The configuration is loaded from `~/.config/wxfetch/config.toml` or from a TOML-file as specified by the `-c` flag. If no configuration file is found it will default to sensible values.

Except for position, these options present personal minima and reflect the colours used. For instance a cloud layer with an altitude lower than the specified minimum will be rendered red.

It is advisable to just copy and modify the sample file from the git repository. The program expects the file to be structured as follows:

### Position

  - `airfield`: ICAO or IATA code of the reporting aerodrome.
  - `lat` & `lon`: Latitude and longitude. The program will look for the nearest reporting station to these coordinates.

If none of the options above are supplies the program defaults to geoip. Positions supplied as command line parameters override the options from the config file.

### Clouds

  - `cloud_minimum`: Minimum altitude for cloud layers.
  - `cloud_marginal`: Altitude at which cloud layers will still be considered marginal.
  
### Temperature

  - `temp_minimum`: Minimum temperature.
  - `spread_minimum`: Minimum spread.

### Wind

  - `wind_var_maximum`: Maximum wind variability.
  - `wind_maximum`: Maximum wind speed.
  - `gust_maximum`: Maximum gust difference.
  
### Age

  - `age_maximum`: Maximum age of the report.
  - `age_marginal`: Marginal age of the report.
  
### Visibility

  - `visibility_minimum`: Minimum visibility.
  - `visibility_marginal`: Marginal visibility.
  
## Providing API keys

For wxfetch to work you will need a free account on https://avwx.rest/. Once you have created an account, go ahead and set the environment variable `AVWX_API_KEY` to your API key.
