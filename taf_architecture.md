# TAF Architecture Document

## Overview

This document outlines the comprehensive architecture for adding Terminal Aerodrome Forecast (TAF) capabilities to the wxfetch Rust aviation weather utility. The implementation will leverage the existing METAR infrastructure while accommodating TAF-specific requirements.

## Current State Analysis

### Existing Infrastructure
- **METAR Module**: Complete parsing, display, and colorization system in `src/metar.rs`
- **API Module**: Fetches data from avwx.rest METAR endpoint in `src/api.rs`
- **Configuration System**: Threshold-based weather condition evaluation in `src/config.rs`
- **CLI Interface**: Already includes `--taf` flag in `src/main.rs`
- **TAF Stub**: Basic struct placeholder in `src/taf.rs`

### Key Components to Reuse
1. **Weather Field Types**: `WxField` enum can be extended for TAF-specific fields
2. **Colorization System**: Existing threshold-based coloring logic
3. **Unit Handling**: `Units` struct and conversion logic
4. **API Client**: HTTP client and authentication mechanisms
5. **Configuration**: TOML-based configuration system

## 1. API Integration Changes

### 1.1 Modified API Endpoints

**Current METAR endpoint:**
```
https://avwx.rest/api/metar/{station}?onfail=nearest&options=info
```

**New TAF endpoint:**
```
https://avwx.rest/api/taf/{station}?onfail=nearest&options=info,summary
```

### 1.2 API Module Modifications (`src/api.rs`)

```rust
/// Modified request function to handle both METAR and TAF
pub async fn request_wx(config: &Config, secrets: &Secrets) -> Option<Value> {
    let position = config.position.get_location_str().await;
    let endpoint = if config.print_taf { "taf" } else { "metar" };
    let options = if config.print_taf { 
        "info,summary" 
    } else { 
        "info" 
    };
    
    let resp = send_api_call(position, endpoint, options, secrets).await.ok()?;
    // ... existing error handling logic
}

/// Updated API call function
async fn send_api_call(
    position: String, 
    endpoint: &str, 
    options: &str,
    secrets: &Secrets
) -> Result<Response, Error> {
    let uri = format!(
        "https://avwx.rest/api/{}/{position}?onfail=nearest&options={}",
        endpoint, options
    );
    // ... existing client logic
}
```

### 1.3 TAF-Specific API Considerations

- **Multiple Forecast Periods**: TAFs contain multiple time-based forecasts
- **Change Groups**: `FROM`, `BECMG`, `TEMPO` groups require special handling
- **Validity Periods**: Start/end times for each forecast segment
- **Summary Information**: Additional summary data from AVWX API

## 2. TAF Data Structures

### 2.1 Core TAF Structure (`src/taf.rs`)

```rust
use chrono::{DateTime, FixedOffset};
use colored::ColoredString;
use serde_json::Value;
use std::ops::Mul;

use crate::config::Config;
use crate::metar::{WxField, Units, get_wxcodes_from_json, get_clouds_from_json};

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
    /// Units used in the forecast
    units: Units,
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

#[derive(Debug, PartialEq)]
pub enum PeriodType {
    Initial,    // Initial forecast period
    From,       // FM - From (permanent change)
    Becoming,   // BECMG - Becoming (gradual change)
    Temporary,  // TEMPO - Temporary fluctuation
    Probability, // PROBxx - Probability
}
```

### 2.2 TAF-Specific Weather Fields

```rust
// Extend WxField enum in src/metar.rs
#[derive(PartialEq, Eq, Debug)]
pub enum WxField {
    // ... existing METAR fields ...
    
    /// TAF-specific: Forecast wind shear
    WindShear {
        altitude: i64,  // Height in feet
        direction: i64,
        strength: i64,
        unit: SpeedUnit,
    },
    
    /// TAF-specific: Maximum temperature forecast
    MaxTemperature {
        temp: i64,
        time: DateTime<FixedOffset>,
        unit: TemperatureUnit,
    },
    
    /// TAF-specific: Minimum temperature forecast
    MinTemperature {
        temp: i64,
        time: DateTime<FixedOffset>,
        unit: TemperatureUnit,
    },
    
    /// TAF-specific: Change indicator (FM, BECMG, TEMPO, PROB)
    ChangeIndicator(PeriodType, Option<u8>),
}
```

## 3. TAF Parsing Logic

### 3.1 Main TAF Parser

```rust
impl Taf {
    pub fn from_json(json: &Value, config: &Config) -> Option<Self> {
        let station = json.get("station")?.as_str()?.to_string();
        let units = Units::from_json(json);
        
        // Parse issue time
        let issue_time = parse_issue_time(json)?;
        
        // Parse validity period
        let (validity_start, validity_end) = parse_validity_period(json)?;
        
        // Parse forecast periods
        let forecast_periods = parse_forecast_periods(json, units)?;
        
        let exact_match = is_exact_match(&station, config);
        
        Some(Taf {
            icao_code: station,
            issue_time,
            validity_start,
            validity_end,
            forecast_periods,
            exact_match,
            units,
        })
    }
}
```

### 3.2 Forecast Period Parsing

```rust
fn parse_forecast_periods(json: &Value, units: Units) -> Option<Vec<ForecastPeriod>> {
    let mut periods = Vec::new();
    
    // Parse initial forecast
    if let Some(initial) = parse_initial_forecast(json, units) {
        periods.push(initial);
    }
    
    // Parse change groups from forecast data
    if let Some(forecast) = json.get("forecast") {
        if let Some(forecast_array) = forecast.as_array() {
            for change_group in forecast_array {
                if let Some(period) = parse_change_group(change_group, units) {
                    periods.push(period);
                }
            }
        }
    }
    
    Some(periods)
}

fn parse_change_group(json: &Value, units: Units) -> Option<ForecastPeriod> {
    let period_type = match json.get("type")?.as_str()? {
        "FROM" => PeriodType::From,
        "BECMG" => PeriodType::Becoming,
        "TEMPO" => PeriodType::Temporary,
        "PROB" => PeriodType::Probability,
        _ => return None,
    };
    
    let start_time = json.get("start_time")
        .and_then(|t| t.as_str())
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok());
    
    let end_time = json.get("end_time")
        .and_then(|t| t.as_str())
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok());
    
    let probability = json.get("probability")
        .and_then(|p| p.as_u64())
        .map(|p| p as u8);
    
    let mut fields = Vec::new();
    
    // Parse standard weather fields (wind, visibility, etc.)
    if let Some(wind) = parse_wind(json, units) {
        fields.push(wind);
    }
    
    if let Some(vis) = parse_visibility(json, units) {
        fields.push(vis);
    }
    
    // Parse weather codes and clouds
    fields.append(&mut get_wxcodes_from_json(json));
    fields.append(&mut get_clouds_from_json(json));
    
    // Parse TAF-specific fields
    if let Some(wind_shear) = parse_wind_shear(json, units) {
        fields.push(wind_shear);
    }
    
    Some(ForecastPeriod {
        period_type,
        start_time,
        end_time,
        fields,
        probability,
    })
}
```

## 4. Display Formatting

### 4.1 TAF Colorization Strategy

```rust
impl Taf {
    pub fn colourise(&self, config: &Config) -> ColoredString {
        let mut output = if self.exact_match {
            self.icao_code.bright_white().on_blue()
        } else {
            self.icao_code.black().on_yellow()
        };
        
        // Add issue time
        let issue_str = format!(" {}", self.issue_time.format("%d%H%MZ"))
            .color(get_time_color(&self.issue_time, config));
        output = format!("{}{}", output, issue_str).into();
        
        // Add validity period
        let validity_str = format!(" {}/{}", 
            self.validity_start.format("%d%H%M"),
            self.validity_end.format("%d%H%M")
        ).bright_cyan();
        output = format!("{}{}", output, validity_str).into();
        
        // Add forecast periods
        for period in &self.forecast_periods {
            output = format!("{} {}", output, colourise_forecast_period(period, config)).into();
        }
        
        output
    }
}

fn colourise_forecast_period(period: &ForecastPeriod, config: &Config) -> ColoredString {
    let mut period_output = ColoredString::default();
    
    // Add change indicator
    match period.period_type {
        PeriodType::From => period_output = "FM".bright_yellow().into(),
        PeriodType::Becoming => period_output = "BECMG".bright_magenta().into(),
        PeriodType::Temporary => period_output = "TEMPO".bright_blue().into(),
        PeriodType::Probability => {
            if let Some(prob) = period.probability {
                period_output = format!("PROB{}", prob).bright_red().into();
            }
        },
        PeriodType::Initial => {}, // No indicator for initial period
    }
    
    // Add time if present
    if let Some(start_time) = period.start_time {
        let time_str = format!(" {}", start_time.format("%d%H%M"))
            .color(get_time_color(&start_time, config));
        period_output = format!("{}{}", period_output, time_str).into();
    }
    
    // Add weather fields
    for field in &period.fields {
        period_output = format!("{} {}", period_output, field.colourise(config)).into();
    }
    
    period_output
}
```

### 4.2 TAF-Specific Colorization Functions

```rust
fn get_time_color(datetime: &DateTime<FixedOffset>, config: &Config) -> Color {
    let now = Utc::now();
    let utctime = datetime.to_utc();
    let dt = now.signed_duration_since(utctime);
    
    if dt < config.age_marginal {
        Color::Green
    } else if dt < config.age_maximum {
        Color::Yellow
    } else {
        Color::Red
    }
}

// Extend WxField::colourise to handle TAF-specific fields
impl WxField {
    pub fn colourise(&self, config: &Config) -> ColoredString {
        match self {
            // ... existing METAR field handling ...
            
            WxField::WindShear { altitude, direction, strength, unit: _ } => {
                format!("WS{altitude:03}/{direction:03}{strength:02}KT")
                    .bright_red().bold()
            },
            
            WxField::MaxTemperature { temp, time, unit: _ } => {
                format!("TX{temp:02}/{}", time.format("%d%H%M"))
                    .bright_yellow()
            },
            
            WxField::MinTemperature { temp, time, unit: _ } => {
                format!("TN{temp:02}/{}", time.format("%d%H%M"))
                    .bright_blue()
            },
            
            WxField::ChangeIndicator(period_type, prob) => {
                match period_type {
                    PeriodType::From => "FM".bright_yellow(),
                    PeriodType::Becoming => "BECMG".bright_magenta(),
                    PeriodType::Temporary => "TEMPO".bright_blue(),
                    PeriodType::Probability => {
                        if let Some(p) = prob {
                            format!("PROB{}", p).bright_red()
                        } else {
                            "PROB".bright_red()
                        }
                    },
                    PeriodType::Initial => " ".white(),
                }
            },
        }
    }
}
```

## 5. Configuration Integration

### 5.1 TAF-Specific Configuration Fields

```rust
// Extend Config struct in src/config.rs
#[derive(PartialEq, Debug)]
pub struct Config {
    // ... existing fields ...
    
    /// TAF-specific: Maximum age for TAF forecasts (hours)
    pub taf_age_maximum: TimeDelta,
    /// TAF-specific: Marginal age for TAF forecasts (hours)
    pub taf_age_marginal: TimeDelta,
    /// TAF-specific: Highlight probability groups
    pub taf_highlight_probability: bool,
    /// TAF-specific: Show change group times
    pub taf_show_change_times: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            // ... existing defaults ...
            
            taf_age_maximum: TimeDelta::hours(24),  // TAFs valid longer
            taf_age_marginal: TimeDelta::hours(6),  // TAFs age slower
            taf_highlight_probability: true,
            taf_show_change_times: true,
        }
    }
}
```

### 5.2 Configuration File Support

Add to `config.toml` parsing:

```toml
[taf]
taf_age_maximum = 86400  # 24 hours in seconds
taf_age_marginal = 21600  # 6 hours in seconds
taf_highlight_probability = true
taf_show_change_times = true
```

## 6. Testing Strategy

### 6.1 Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use serde_json::Value;

    #[tokio::test]
    async fn test_taf_from_json_basic() {
        let json = r#"
        {
            "station": "KJFK",
            "time": {"dt": "2024-06-21T12:00:00Z"},
            "validity": {"start": "2024-06-21T13:00:00Z", "end": "2024-06-22T12:00:00Z"},
            "forecast": []
        }
        "#;
        
        let value: Value = Value::from_str(json).unwrap();
        let config = Config::default();
        let taf = Taf::from_json(&value, &config);
        
        assert!(taf.is_some());
        assert_eq!(taf.unwrap().icao_code, "KJFK");
    }

    #[tokio::test]
    async fn test_forecast_period_parsing() {
        let json = r#"
        {
            "type": "TEMPO",
            "start_time": "2024-06-21T18:00:00Z",
            "end_time": "2024-06-21T22:00:00Z",
            "wind_direction": {"value": 270},
            "wind_speed": {"value": 15}
        }
        "#;
        
        let value: Value = Value::from_str(json).unwrap();
        let period = parse_change_group(&value, Units::default());
        
        assert!(period.is_some());
        let period = period.unwrap();
        assert_eq!(period.period_type, PeriodType::Temporary);
    }

    #[tokio::test]
    async fn test_taf_colorization() {
        let taf = create_test_taf();
        let config = Config::default();
        let colored = taf.colourise(&config);
        
        // Verify the colored string contains expected elements
        assert!(colored.to_string().contains("KJFK"));
    }
}
```

### 6.2 Integration Tests

```rust
#[tokio::test]
async fn test_api_taf_request() {
    let secrets = Secrets {
        avwx_api_key: "test_key".to_string(),
    };
    
    let config = Config {
        position: Position::Airfield("KJFK".to_string()),
        print_taf: true,
        ..Default::default()
    };
    
    // Mock the API call or use test data
    let json = load_test_taf_data();
    let taf = Taf::from_json(&json, &config);
    
    assert!(taf.is_some());
}

#[tokio::test]
async fn test_cli_taf_flag() {
    // Test that --taf flag properly sets config.print_taf
    let args = Args {
        taf: true,
        ..Default::default()
    };
    
    let secrets = get_secrets(None);
    let config = Config::get_config(&secrets, &args).await;
    
    assert!(config.print_taf);
}
```

### 6.3 Test Data

Create test files in `tests/testdata/`:

- `kjfk-taf.json` - Complete TAF with multiple change groups
- `egkk-taf-simple.json` - Simple TAF without change groups
- `eddf-taf-prob.json` - TAF with probability groups

## 7. Implementation Approach

### 7.1 Phase 1: Core Structure (Week 1)
1. **Extend API Module**: Add TAF endpoint support
2. **Basic TAF Struct**: Implement core data structures
3. **Simple Parser**: Parse basic TAF fields (station, times)
4. **Basic Display**: Simple text output without colorization

### 7.2 Phase 2: Forecast Periods (Week 2)
1. **Change Group Parsing**: Implement FM, BECMG, TEMPO parsing
2. **Period Structure**: Complete ForecastPeriod implementation
3. **Weather Field Integration**: Reuse METAR field parsing
4. **Time Handling**: Proper datetime parsing for all periods

### 7.3 Phase 3: Advanced Features (Week 3)
1. **TAF-Specific Fields**: Wind shear, temperature forecasts
2. **Colorization System**: Complete colorization logic
3. **Configuration Integration**: Add TAF-specific config options
4. **Error Handling**: Robust error handling for malformed data

### 7.4 Phase 4: Testing & Polish (Week 4)
1. **Comprehensive Tests**: Unit and integration tests
2. **Test Data**: Create comprehensive test dataset
3. **Documentation**: Update README and inline documentation
4. **Performance**: Optimize parsing and display performance

## 8. Code Reuse Strategy

### 8.1 Direct Reuse
- **Units System**: Use existing `Units` struct and conversion logic
- **Weather Codes**: Reuse `WxCode` enums and parsing from `metar/wxcodes.rs`
- **Cloud Parsing**: Use existing cloud layer parsing from `metar/clouds.rs`
- **Color Utilities**: Leverage existing colorization functions and thresholds

### 8.2 Adapted Reuse
- **Wind Parsing**: Adapt METAR wind parsing for TAF forecast winds
- **Visibility**: Use METAR visibility parsing with TAF-specific considerations
- **Temperature**: Extend temperature parsing for forecast min/max values
- **API Client**: Modify existing HTTP client for TAF endpoints

### 8.3 New Implementation
- **Forecast Periods**: New concept not present in METAR
- **Change Indicators**: FM, BECMG, TEMPO, PROB parsing
- **Validity Periods**: TAF-specific time range handling
- **Wind Shear**: TAF-specific wind shear reporting

## 9. Error Handling Considerations

### 9.1 API Errors
- **Invalid Station**: Handle stations without TAF service
- **Rate Limiting**: Respect AVWX API rate limits
- **Network Issues**: Graceful degradation for connectivity problems

### 9.2 Data Validation
- **Malformed JSON**: Handle unexpected API response formats
- **Missing Fields**: Graceful handling of optional TAF fields
- **Time Parsing**: Robust datetime parsing with fallbacks

### 9.3 User Experience
- **Clear Error Messages**: Informative error messages for users
- **Fallback Options**: Attempt nearest station when specific fails
- **Configuration Validation**: Validate TAF-specific configuration

## 10. Performance Considerations

### 10.1 Parsing Efficiency
- **Lazy Parsing**: Only parse fields that will be displayed
- **Memory Management**: Efficient string handling for large TAFs
- **Caching**: Consider caching parsed TAF data for repeated requests

### 10.2 Display Optimization
- **String Building**: Efficient string concatenation for output
- **Color Overhead**: Minimize colorization performance impact
- **Large Forecasts**: Handle TAFs with many change groups efficiently

## 11. Future Enhancements

### 11.1 Advanced Features
- **TAF Amending**: Handle amended TAFs (TAF AMD)
- **Conditional Groups**: Support for conditional weather groups
- **Climatology**: Historical TAF data comparison

### 11.2 Display Options
- **Compact Mode**: Condensed display for terminal usage
- **Detailed Mode**: Verbose display with all available data
- **JSON Output**: Machine-readable output option

### 11.3 Integration
- **METAR/TAF Combined**: Display both reports together
- **Flight Planning**: Integration with flight planning tools
- **Alerts**: Weather condition alerts based on TAF data

## Conclusion

This architecture provides a comprehensive foundation for implementing TAF support in wxfetch while maximizing code reuse from the existing METAR implementation. The phased approach ensures manageable development cycles with working functionality at each stage.

The design maintains consistency with the existing codebase patterns while accommodating TAF-specific requirements like forecast periods, change groups, and extended validity periods. The modular structure allows for future enhancements and maintains the clean, testable architecture of the current implementation.