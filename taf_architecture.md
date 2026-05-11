# TAF Architecture Document

> **Last revised:** 2026-05-11 — aligned with implementation on branch `feature/taf` (commit `1f69184`)

## Overview

This document describes the Terminal Aerodrome Forecast (TAF) implementation for the wxfetch Rust aviation weather utility. The implementation reuses the existing METAR infrastructure while accommodating TAF-specific requirements like forecast periods, change groups, and extended validity windows.

## Current Implementation Status

| Area | Status |
|------|--------|
| Core data structures (`Taf`, `ForecastPeriod`, `PeriodType`) | ✅ Complete |
| JSON parsing from AVWX API response | ✅ Complete |
| Colorized terminal display | ✅ Complete |
| API routing (METAR vs TAF endpoint) | ✅ Complete |
| Nearest-station fallback | ✅ Complete |
| CLI `--taf` flag | ✅ Complete |
| TAF-specific config fields + TOML parsing | ✅ Complete |
| Wind shear (`WxField::WindShear`) | ❌ Not yet implemented |
| Max/Min temperature forecasts (`Tx/TN`) | ❌ Not yet implemented |
| `taf_highlight_probability` in display code | ❌ Config field exists, display logic missing |
| `units` field on `Taf` struct | ❌ Omitted from struct (used transiently) |
| Raw/non-colorized output mode | ❌ Not yet implemented |
| `[taf]` section in shipped `config.toml` | ❌ Not yet added |

---

## 1. API Integration

### 1.1 Endpoints

| Report | Endpoint | Options |
|--------|----------|---------|
| METAR | `https://avwx.rest/api/metar/{station}?onfail=nearest&options=info` | `info` |
| TAF | `https://avwx.rest/api/taf/{station}?onfail=nearest&options=info,summary` | `info,summary` |

### 1.2 API Module (`src/api.rs`)

The `request_wx()` function selects the endpoint and options based on `config.print_taf`:

```rust
pub async fn request_wx(config: &Config, secrets: &Secrets) -> Option<Value> {
    let position = config.position.get_location_str().await;
    let endpoint = if config.print_taf { "taf" } else { "metar" };
    let options = if config.print_taf {
        "info,summary"
    } else {
        "info"
    };

    let resp = send_api_call(position, endpoint, options, secrets).await.ok()?;
    let status = resp.status().as_u16();

    if status == 200 {
        resp.json().await.ok()
    } else if status == 401 {
        error!("Weather request failed. Provide a valid AvWx API key.");
        None
    } else if let Some(nearest_station_code) = get_nearest_station(config, secrets).await {
        send_api_call(nearest_station_code, endpoint, options, secrets)
            .await
            .ok()?
            .json::<Value>()
            .await
            .ok()
    } else {
        println!("No nearest station...");
        None
    }
}
```

The `send_api_call()` helper constructs the URL, sets the `BEARER` auth header, and returns the raw `reqwest::Response`.

The `get_nearest_station()` fallback performs a two-step lookup:
1. Resolve the user's position to latitude/longitude via the `/api/station/` endpoint
2. Find the nearest reporting station via `/api/station/near/{lat},{lon}?n=1&reporting=true`

### 1.3 TAF-Specific API Considerations

- **Multiple Forecast Periods**: TAFs return a `"forecast"` array with time-bounded segments
- **Summary Data**: The `summary` option provides additional parsed fields not available in METAR responses

---

## 2. Data Structures

### 2.1 Core TAF Structure (`src/taf.rs`)

```rust
use chrono::{DateTime, FixedOffset, Utc};
use colored::{Color, ColoredString, Colorize};
use serde_json::Value;

use crate::config::Config;
use crate::metar::{WxField, clouds::get_clouds_from_json, wxcodes::get_wxcodes_from_json};
use crate::metar::{get_visibility, get_winds, is_exact_match};

pub struct Taf {
    /// ICAO code of the issuing station
    icao_code: String,
    /// Issue time of the forecast
    issue_time: DateTime<FixedOffset>,
    /// Validity period start and end
    validity_start: DateTime<FixedOffset>,
    validity_end: DateTime<FixedOffset>,
    /// Forecast periods and change groups
    forecast_periods: Vec<ForecastPeriod>,
    /// True if this TAF was issued by the exact station requested
    exact_match: bool,
}

/// Represents a forecast period or change group
pub struct ForecastPeriod {
    /// Type of period (FM, BECMG, TEMPO, or initial forecast)
    period_type: PeriodType,
    /// Start time of this period
    start_time: Option<DateTime<FixedOffset>>,
    /// End time of this period (if applicable)
    end_time: Option<DateTime<FixedOffset>>,
    /// Weather fields for this period
    fields: Vec<WxField>,
    /// Probability for PROB groups (30 or 40)
    probability: Option<u8>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum PeriodType {
    Initial,     // Initial forecast period
    From,        // FM - From (permanent change)
    Becoming,    // BECMG - Becoming (gradual change)
    Temporary,   // TEMPO - Temporary fluctuation
    Probability, // PROBxx - Probability
}
```

> **Design note — `units` field:** The original architecture included `units: Units` on `Taf`, but it was dropped from the implementation. `Units` is parsed transiently via `Units::from_json(json)` inside shared helper functions (`get_winds()`, `get_visibility()`) and is not stored. If unit-aware display customization is needed (e.g., showing "KT" vs "KPH"), re-add this field.

### 2.2 Weather Fields (Inherited from METAR)

TAF reuses all METAR weather fields via `WxField` in `src/metar.rs`:

- `Wind { direction, strength, gusts, unit }`
- `WindVariability { low_dir, hi_dir }`
- `Visibility(i64)`
- `Temperature { temp, dewpoint, unit }`
- `Qnh(i64, PressureUnit)`
- `Clouds(Clouds, i64)`
- `WxCode(WxCode, WxCodeIntensity, WxCodeProximity, WxCodeDescription)`
- `TimeStamp(DateTime<FixedOffset>)`
- `Remarks(String)`

The `WxField` enum lives in `src/metar.rs` and its `colourise()` method handles all rendering, including TAF-specific extensions (see §2.3).

### 2.3 Planned TAF-Specific Fields (Not Yet Implemented)

The following are defined in the architecture but **not yet added** to the codebase:

```rust
// MARKED TODO — add to WxField enum in src/metar.rs when implementing:
//
// /// TAF-specific: Forecast wind shear (e.g., "WS010/31022KT")
// WindShear {
//     altitude: i64,   // Height in hundreds of feet AGL (e.g., 010 = 1000 ft)
//     direction: i64,
//     strength: i64,
//     unit: SpeedUnit,
// },
//
// /// TAF-specific: Maximum temperature forecast (e.g., "TX35/2118")
// MaxTemperature {
//     temp: i64,
//     time: DateTime<FixedOffset>,
//     unit: TemperatureUnit,
// },
//
// /// TAF-specific: Minimum temperature forecast (e.g., "TN25/2204")
// MinTemperature {
//     temp: i64,
//     time: DateTime<FixedOffset>,
//     unit: TemperatureUnit,
// },
```

> **Design decision — change indicators:** Change indicators (FM, BECMG, TEMPO, PROB) are handled structurally via `ForecastPeriod.period_type` rather than as a `WxField::ChangeIndicator` variant. This is a deliberate divergence from the original architecture doc — the period type controls the display prefix, not a field within `fields`.

---

## 3. TAF Parsing Logic (`src/taf.rs`)

### 3.1 Main Parser

`Taf::from_json()` extracts the station, issue time, validity period, and forecast array from the AVWX API JSON response:

```rust
impl Taf {
    pub fn from_json(json: &Value, config: &Config) -> Option<Self> {
        let station = json.get("station")?.as_str()?.to_string();

        let issue_time = parse_issue_time(json)?;
        let (validity_start, validity_end) = parse_validity_period(json)?;
        let forecast_periods = parse_forecast_periods(json)?;

        let exact_match = is_exact_match(&station, config);

        Some(Taf {
            icao_code: station,
            issue_time,
            validity_start,
            validity_end,
            forecast_periods,
            exact_match,
        })
    }
}
```

### 3.2 Forecast Period Parsing

The key design choice: the **first element** in the `"forecast"` array is always the initial forecast (no `"type"` field). Subsequent elements have a `"type"` field (`"FM"`, `"BECMG"`, `"TEMPO"`, `"PROB"`).

```rust
fn parse_forecast_periods(json: &Value) -> Option<Vec<ForecastPeriod>> {
    let mut periods = Vec::new();

    if let Some(forecast_array) = json.get("forecast")?.as_array() {
        for (i, change_group) in forecast_array.iter().enumerate() {
            if let Some(period) = parse_change_group(change_group, i == 0) {
                periods.push(period);
            }
        }
    }

    Some(periods)
}

fn parse_change_group(json: &Value, is_initial: bool) -> Option<ForecastPeriod> {
    let period_type = if is_initial {
        PeriodType::Initial
    } else {
        match json.get("type")?.as_str()? {
            "FM" => PeriodType::From,
            "BECMG" => PeriodType::Becoming,
            "TEMPO" => PeriodType::Temporary,
            "PROB" => PeriodType::Probability,
            _ => return None,
        }
    };

    let start_time = json
        .get("start_time")
        .and_then(|t| t.get("dt"))
        .and_then(|t| t.as_str())
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok());

    let end_time = json
        .get("end_time")
        .and_then(|t| t.get("dt"))
        .and_then(|t| t.as_str())
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok());

    let probability = json
        .get("probability")
        .and_then(|p| p.get("value"))
        .and_then(|p| p.as_u64())
        .map(|p| p as u8);

    let mut fields = Vec::new();
    if let Some(wind) = get_winds(json) { fields.push(wind); }
    if let Some(vis) = get_visibility(json) { fields.push(vis); }
    fields.append(&mut get_wxcodes_from_json(json));
    fields.append(&mut get_clouds_from_json(json));

    // MARKED TODO: add wind_shear, max_temp, min_temp parsing here

    Some(ForecastPeriod {
        period_type,
        start_time,
        end_time,
        fields,
        probability,
    })
}
```

Key differences from METAR parsing:
- First forecast array element is `Initial` (no `"type"` field)
- Subsequent elements get their type from the `"type"` string
- `start_time`/`end_time` are nested objects with `"dt"` RFC 3339 strings
- `probability` is an object `{"value": 30}`, not a bare integer

---

## 4. Display Formatting (`src/taf.rs`)

### 4.1 Colorization Strategy

The `colourise()` method on `Taf` builds the output string:

```
TAF [STATION] [ISSUE_TIME] [VALIDITY] [INITIAL_FORECAST]
     [CHANGE_INDICATOR] [TIME] [FIELDS]
     [CHANGE_INDICATOR] [TIME] [FIELDS]
     ...
```

- **Station**: Bright white on blue (exact match) or black on yellow (nearest station)
- **Issue time**: Green (< 6h) / Yellow (< 24h) / Red (> 24h) based on age
- **Validity period**: Cyan, formatted as `DDHH/DDHH`
- **Change indicators**: FM=yellow, BECMG=magenta, TEMPO=blue, PROB=red
- **Weather fields**: Delegated to `WxField::colourise()` (inherited from METAR)

### 4.2 `taf_show_change_times` Config

When `true` (default), change groups display their time windows:
- FM → `FM211900`
- BECMG → `BECMG 2122/2200`
- TEMPO → `TEMPO 2120/2122`
- PROB → `PROB30 2204/2207`

When `false`, only the indicator text is shown (without time windows).

### 4.3 `taf_highlight_probability` Config

This config field (default: `true`) is now checked in the `colourise_forecast_period()` function:
- `true`: PROB indicators render in bright red (default)
- `false`: PROB indicators render in bright black (dimmed)

Implementation is in `src/taf.rs` lines ~230–240.

### 4.4 Planned TAF Field Colorization

When `WindShear`, `MaxTemperature`, and `MinTemperature` fields are added to `WxField`:

| Field | Format | Color |
|-------|--------|-------|
| `WindShear` | `WS{alt:03}/{dir:03}{spd:02}KT` | Bright red, bold |
| `MaxTemperature` | `TX{temp:02}/{ddHHMM}` | Bright yellow |
| `MinTemperature` | `TN{temp:02}/{ddHHMM}` | Bright blue |

---

## 5. Configuration (`src/config.rs`)

### 5.1 TAF-Specific Fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `taf_age_maximum` | `TimeDelta` | 24 hours | Max age before TAF turns red |
| `taf_age_marginal` | `TimeDelta` | 6 hours | Marginal age for yellow warning |
| `taf_highlight_probability` | `bool` | `true` | Colorize PROB groups red (display pending) |
| `taf_show_change_times` | `bool` | `true` | Show time windows for FM/BECMG/TEMPO/PROB |

### 5.2 TOML Configuration

The parser reads a `[taf]` section from `~/.config/wxfetch/config.toml`. Current shipped config does **not** include this section — defaults are hardcoded. Add it to `config.toml`:

```toml
[taf]
taf_age_maximum = 86400       # 24 hours in seconds
taf_age_marginal = 21600      # 6 hours in seconds
taf_highlight_probability = true
taf_show_change_times = true
```

---

## 6. Testing

### 6.1 Test Data Files (`tests/testdata/`)

| File | Description | Change Groups |
|------|-------------|---------------|
| `kjfk-taf.json` | KJFK, initial + BECMG | Initial, BECMG |
| `eddf-taf-prob.json` | EDDF, initial + PROB30 | Initial, PROB |
| `egkk-taf-simple.json` | EGKK, empty forecast | Initial only |
| `malformed-taf.json` | Invalid/minimal JSON | — |

The `malformed-taf.json` file was created for testing but the corresponding test was not yet added to `src/taf.rs`.

### 6.2 Existing Unit Tests (7 tests)

| Test | File | What it checks |
|------|------|----------------|
|| `test_taf_prob_highlighting` | `taf.rs` | PROB30 rendering with EDDF |
| `test_taf_wind_shear_parsing` | `taf.rs` | Parses WS from kjfk-taf-windshear.json |
| `test_taf_temperature_extremes_parsing` | `taf.rs` | Parses TX/TN from kjfk-taf-temp-extremes.json |
| `test_taf_comprehensive` | `taf.rs` | Full field coverage (EDDF, incl. units) |
| `test_taf_missing_forecast` | `taf.rs` | Returns `None` when no forecast key |
| `test_taf_empty_forecast_array` | `taf.rs` | Parses EGKK with empty forecast[] |
| `test_taf_missing_fields_on_type` | `taf.rs` | Handles partial change groups gracefully |
| `test_taf_with_units` | `taf.rs` | Parses units block, verifies TemperatureUnit/SpeedUnit |
| `test_taf_fuzz_malformed_inputs` | `taf.rs` | 17 malformed inputs, must never panic |

### 6.3 Tests to Add

|| Test | Description |
||------|-------------|
|| PROB without time window | Edge case where PROB has no start/end |
|| Real AVWX API response | Capture and parse a live response |

---

## 7. Remaining Tasks (Post-Recovery)

Completed tasks have been struck through. Remaining work as of 2026-05-11 (all complete):

~~1. **Add `WindShear` to `WxField`**~~ — ✅ Done
~~2. **Add `MaxTemperature`/`MinTemperature` to `WxField`**~~ — ✅ Done
~~3. **Wire `taf_highlight_probability` into display**~~ — ✅ Done
~~4. **Add `units: Units` to `Taf` struct**~~ — ✅ Done
~~5. **Add `[taf]` section to `config.toml`**~~ — ✅ Already present
~~6. **Run full CI validation**~~ — ✅ 147 tests, 0 clippy warnings
~~7. **Add `--raw` output flag**~~ — ✅ Committed `c0d9339`
~~9. **Add missing test fixtures and fuzz/panic tests**~~ — ✅ Committed `074ea7c` (17 malformed inputs, never panic)
~~10. **Update README**~~ — ✅ Committed `81150d0` (documented `--taf`, `--raw`, `[taf]` config)
~~11. **Add PROB-without-time-window test**~~ — ✅ Committed `0b6b501` (PROB40 with no start/end time)

### Task 8: `--raw` flag — ✅ Done

Added `-R`/`--raw` CLI flag (`src/main.rs`) that strips ANSI escape codes from output using `OnceLock<Regex>`. Useful for piping, logging, and CI/CD pipelines. Committed as `c0d9339`.

### Task 9: Additional tests — ✅ Done

Added `test_taf_fuzz_malformed_inputs` with 17 deterministic malformed JSON inputs (empty, null, wrong types, garbage fields, unicode edge cases, extreme/negative values, unrecognized change types). All run under `catch_unwind` — parser must never panic. Committed as `074ea7c`.

### Task 10: README update — ✅ Done

Documented `--taf` flag, `-R`/`--raw` flag, and `[taf]` config section (all 4 fields with defaults). Committed as `81150d0`.

---

## 8. Code Reuse Summary

| Component | Source | Reuse |
|-----------|--------|-------|
| `Units` struct + parsing | `src/metar/units.rs` | Transient use in `get_winds()` / `get_visibility()` |
| `WxCode` enum + regex parsing | `src/metar/wxcodes.rs` | `get_wxcodes_from_json()` called from TAF parser |
| `Clouds` enum + parsing | `src/metar/clouds.rs` | `get_clouds_from_json()` called from TAF parser |
| `get_winds()` / `get_visibility()` | `src/metar.rs` | Called directly from `parse_change_group()` |
| `is_exact_match()` | `src/metar.rs` | Called from `Taf::from_json()` |
| Color thresholds (age, wind, cloud) | `src/metar.rs` | TAF uses same age-based thresholds from `Config` |
| Auth / HTTP client | `src/api.rs` | Shared endpoint with fallback logic |

---

## 9. Error Handling

- **Missing fields**: All parsers use `Option` chaining — missing JSON fields yield `None` at the appropriate level
- **Malformed datetimes**: RFC 3339 parse failures silently yield `None` times
- **Unknown change group types**: `parse_change_group()` returns `None` for unrecognized `"type"` values
- **Invalid station**: `Config::get_config()` validates via `check_icao_code()` and falls back to GeoIP
- **API failure**: `request_wx()` falls back to nearest station, then returns `None`

---

## 10. Known Divergences from Original Design

| Original Plan | Actual Implementation | Reason |
|--------------|----------------------|--------|
| `ChangeIndicator` variant in `WxField` | `PeriodType` drives display structurally via `ForecastPeriod` | Cleaner separation — period type is structural, not a display field |
| `units: Units` stored on `Taf` | Added | Units now stored and propagated through to `WxField::colourise()` for unit suffix display |
| `parse_wind()` defined in TAF | Uses `get_winds()` from METAR directly | Identical logic — no reason to duplicate |
| `parse_initial_forecast()` separate function | Initial period is first array element, parsed by same `parse_change_group()` | Simpler — the only difference is the `"type"` field is absent |

---

## Conclusion

The TAF implementation is functional and well-tested, reusing ~80% of the existing METAR infrastructure. The remaining work consists of adding TAF-specific field types (wind shear, temperature extremes), wiring up one unused config flag, and adding a few robustness tests. The architecture doc now accurately reflects the state of the code.