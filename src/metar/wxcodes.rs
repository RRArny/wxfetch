use std::fmt::Display;

use serde_json::Value;

use super::MetarField;

#[derive(PartialEq, Eq, Debug)]
/// Standardised codes for weather phenomena.
pub enum WxCode {
    /// Rain.
    Ra,
    /// Drizzle.
    Dz,
    /// Hail (Diameter >= 5mm).
    Gr,
    /// Small Hail (Diameter < 5mm).
    Gs,
    /// Ice crystals.
    Ic,
    /// Ice pellets.
    Pl,
    /// Snow grains.
    Sg,
    /// Snow.
    Sn,
    /// Unknown precipitation (automated reports only).
    Up,
    /// Mist (visibility >= 1000m).
    Br,
    /// Widespread dust.
    Du,
    /// Fog (visibility >= 1000m).
    Fg,
    /// Smoke.
    Fu,
    /// Haze.
    Hz,
    /// Spray.
    Py,
    /// Sand.
    Sa,
    /// Volcanic ash.
    Va,
    /// Dust storm.
    Ds,
    /// Funnel clouds.
    Fc,
    /// Well-developed sand or dust whirls.
    Po,
    /// Squalls.
    Sq,
    /// Sandstorm.
    Ss,
}

#[derive(PartialEq, Eq, Debug)]
/// Used to specify a weather phenomenon's intensity.
pub enum WxCodeIntensity {
    Moderate,
    Light,
    Heavy,
}

#[derive(PartialEq, Eq, Debug)]
/// Used to specify a weather phenomenon's distance from reporting staion.
pub enum WxCodeProximity {
    /// On station.
    OnStation,
    /// In vicinity of the station (distance 5 - 10 miles).
    Vicinity,
    /// More than 10 miles from station.
    Distant,
}

#[derive(PartialEq, Eq, Debug)]
/// Used to further specify a weather phenomenon.
pub enum WxCodeDescription {
    /// No description.
    None,
    /// Thunderstorm.
    Ts,
    /// Patches.
    Bc,
    /// Blowing.
    Bl,
    /// Low drifting.
    Dr,
    /// Freezing.
    Fz,
    /// Shallow.
    Mi,
    /// Partial.
    Pr,
    /// Shower(s).
    Sh,
}

impl Display for WxCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str_repr: &str = match self {
            WxCode::Ra => "RA",
            WxCode::Dz => "DZ",
            WxCode::Gr => "GR",
            WxCode::Gs => "GS",
            WxCode::Ic => "IC",
            WxCode::Pl => "PL",
            WxCode::Sg => "SG",
            WxCode::Sn => "SN",
            WxCode::Up => "UP",
            WxCode::Br => "BR",
            WxCode::Du => "DU",
            WxCode::Fg => "FG",
            WxCode::Fu => "FU",
            WxCode::Hz => "HZ",
            WxCode::Py => "PY",
            WxCode::Sa => "SA",
            WxCode::Va => "VA",
            WxCode::Ds => "DS",
            WxCode::Fc => "FC",
            WxCode::Po => "PO",
            WxCode::Sq => "SQ",
            WxCode::Ss => "SS",
        };
        write!(f, "{}", str_repr)
    }
}

impl Display for WxCodeIntensity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                WxCodeIntensity::Moderate => "",
                WxCodeIntensity::Light => "-",
                WxCodeIntensity::Heavy => "+",
            }
        )
    }
}

impl Display for WxCodeProximity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str_repr: &str = match self {
            WxCodeProximity::OnStation => "",
            WxCodeProximity::Vicinity => "VC",
            WxCodeProximity::Distant => "DSNT",
        };
        write!(f, "{}", str_repr)
    }
}

impl Display for WxCodeDescription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str_repr: &str = match self {
            WxCodeDescription::None => "",
            WxCodeDescription::Ts => "TS",
            WxCodeDescription::Bc => "BC",
            WxCodeDescription::Bl => "BL",
            WxCodeDescription::Dr => "DR",
            WxCodeDescription::Fz => "FZ",
            WxCodeDescription::Mi => "MI",
            WxCodeDescription::Pr => "PR",
            WxCodeDescription::Sh => "SH",
        };
        write!(f, "{}", str_repr)
    }
}

pub(crate) fn wxcode_from_str(_repr: &str) -> MetarField {
    // TODO
    MetarField::WxCode(
        WxCode::Ra,
        WxCodeIntensity::Light,
        WxCodeProximity::OnStation,
        WxCodeDescription::None,
    )
}

// impl From<&str> for FromIterator

pub fn get_wxcodes(json: &Value) -> Vec<MetarField> {
    let mut result: Vec<MetarField> = Vec::new();
    if let Some(wxcodes) = json.get("wx_codes").and_then(|x| x.as_array()) {
        for code in wxcodes {
            if let Some(repr) = code.get("repr").and_then(|x| x.as_str()) {
                result.push(wxcode_from_str(repr));
            }
        }
    }
    result
}
