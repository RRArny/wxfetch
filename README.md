# WXfetch

Console utility for accessing aviation weather information from the command line.

## Parameters

If you provide no flags to WXfetch at all, it will try and fetch weather info from your closest airfield according to your IP based position (geoip).

With `-a` or `--airfield` you can provide the ICAO or IATA code for a reporting station. Alternatively, with `--lat` and `--lon` you can provide geographical coordinates. WXfetch will then try and find a reporting station close to that position. Please make sure to provide both parameters.

If there is any problem with the provided arguments WXfetch will print an error message and default to geoip.

## Configuration

So far, no configuration options other than the ones described above are recognised, but they're coming in the future!

## Providing API keys

For wxfetch to work you will need a free account on https://avwx.rest/. Once you have created an account, go ahead and copy `secrets_template.toml` to `secrets.toml` and paste in your API key.

## Todos

- [ ] Personal wx minima
- [ ] Configuration options
