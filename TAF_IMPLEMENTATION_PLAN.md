# TAF Feature — Implementation Plan

> Generated from codebase analysis on 2026-05-11.
> Branch: `feature/taf` | Commit: `e529cb9`

---

## Summary of Current State

The TAF feature is **~70% complete**. Core parsing, display, API routing, config, and tests all work. The remaining work fills in planned-but-unimplemented fields, hardens edge cases, and polishes the UX.

### What Works

- [x] `Taf` struct with full parse/display pipeline
- [x] Forecast period parsing (FM, BECMG, TEMPO, PROB)
- [x] METAR field reuse: wind, visibility, weather codes, clouds
- [x] Time-aware colorization (age-based green/yellow/red)
- [x] Change-group colorized display (FM=yellow, BECMG=magenta, TEMPO=blue, PROB=red)
- [x] API endpoint routing (`taf/` vs `metar/`, `summary` option)
- [x] `--taf` CLI flag
- [x] Config struct with 4 TAF-specific fields + TOML parsing
- [x] 7 unit tests with 4 test fixtures (kjfk, eddf-prob, egkk-simple, malformed)
- [x] Nearest-station fallback for TAF API calls

### What Is Missing

- [ ] **Wind shear** field (architecture doc + parser both missing)
- [ ] **Max/Min temperature** fields (architecture doc + parser both missing)
- [ ] **`taf_highlight_probability`** config flag declared but never checked in display code
- [ ] **`units` field** on `Taf` struct (architecture doc specifies it, struct omits it)
- [ ] **`[taf]` section** missing from shipped `config.toml`
- [ ] **Raw output mode** (only colorized terminal output exists)
- [ ] **PROB time window display** — PROB groups without start/end times render without any time info

---

## Phased Implementation

### Phase 1 — Missing TAF-Specific Fields (2 tasks)

#### 1.1 Add `WxField::WindShear` variant and parser

**Why:** Wind shear is a TAF-specific phenomenon that METAR does not report. The architecture doc already defines the struct shape.

**Changes:**

- `src/metar.rs` — Add to `WxField` enum:
  ```rust
  WindShear { altitude: i64, direction: i64, strength: i64, unit: SpeedUnit },
  ```
- `src/metar.rs` — Add `colourise` match arm:
  ```rust
  WxField::WindShear { altitude, direction, strength, unit: _ } => {
      format!("WS{altitude:03}/{direction:03}{strength:02}KT")
          .bright_red().bold()
  }
  ```
- `src/taf.rs` — Add `parse_wind_shear(json, units)` function; call it inside `parse_change_group()`.
- **Test:** Add unit test for `WindShear` parsing and colorization.

**Difficulty:** Low — pattern is identical to existing wind/visibility parsers.

#### 1.2 Add `WxField::MaxTemperature` / `MinTemperature` variants and parser

**Why:** TAFs can include TX (max) and TN (min) temperature forecasts. These are TAF-only; METAR uses observed temperature.

**Changes:**

- `src/metar.rs` — Add two variants to `WxField`:
  ```rust
  MaxTemperature { temp: i64, time: DateTime<FixedOffset>, unit: TemperatureUnit },
  MinTemperature { temp: i64, time: DateTime<FixedOffset>, unit: TemperatureUnit },
  ```
- `src/metar.rs` — Add `colourise` match arms:
  ```rust
  WxField::MaxTemperature { temp, time, unit: _ } =>
      format!("TX{temp:02}/{}", time.format("%d%H%M")).bright_yellow(),
  WxField::MinTemperature { temp, time, unit: _ } =>
      format!("TN{temp:02}/{}", time.format("%d%H%M")).bright_blue(),
  ```
- `src/taf.rs` — Add `parse_max_temp()` and `parse_min_temp()` functions; call from `parse_change_group()`.
- **Test:** Add unit tests for both temperature field parsing.

**Difficulty:** Low-medium — requires `DateTime` handling (already used elsewhere).

---

### Phase 2 — Unfinished Config & Display Polish (3 tasks)

#### 2.1 Wire up `taf_highlight_probability` in display code

**Why:** The config field exists and defaults to `true`, but `colourise_forecast_period()` never reads it. PROB groups should only be bright-red when this flag is enabled; otherwise rendered in default color.

**Changes:**

- `src/taf.rs` — `colourise_forecast_period()`: check `config.taf_highlight_probability` before applying `.bright_red()` to PROB indicators.
- `src/config.rs` — No changes needed (field already exists).

**Difficulty:** Trivial.

#### 2.2 Store `Units` in `Taf` struct and propagate

**Why:** The architecture doc specifies `units: Units` on `Taf`. Currently the struct drops units after parsing, which means unit-aware display (e.g., showing "KT" vs "KPH") cannot vary per-forecast.

**Changes:**

- `src/taf.rs` — Add `units: Units` field to `Taf` struct.
- `src/taf.rs` — Parse units from `Units::from_json(json)` in `from_json()` and store.
- `src/taf.rs` — Pass `self.units` to `colourise_forecast_period()` so field colorization can be unit-aware.

**Difficulty:** Low — 3-4 line additions plus plumbing.

#### 2.3 Add `[taf]` section to shipped `config.toml`

**Why:** The parser supports it; the default values are reasonable; but users can't discover or customize it without documentation.

**Changes:**

```toml
[taf]
taf_age_maximum = 86400      # 24 hours in seconds
taf_age_marginal = 21600     # 6 hours in seconds
taf_highlight_probability = true
taf_show_change_times = true
```

Also document each key with inline comments.

**Difficulty:** Trivial.

---

### Phase 3 — Raw/Structured Output Mode (1 task)

#### 3.1 Add `--output-format` (or `--raw`) option

**Why:** Current output is always ANSI-colorized terminal text. Power users and scripts need machine-readable output (plain text or JSON).

**Design decision needed:** Choose one of:
- **Option A:** `--raw` flag — strips ANSI codes, prints plain TAF string.
- **Option B:** `--output-format {text,json}` — `text` = no ANSI, `json` = serialized struct.

**Recommended:** Option A for simplicity. `--raw` can be implemented with `colored::control::SHOULD_COLORIZE.set_override(false)` or by collecting the `ColoredString` into a plain `String`. Option B can be a follow-up.

**Changes:**

- `src/main.rs` — Add `--raw` flag to `Args`.
- `src/main.rs` — When `--raw` is set, either disable color globally or render via `.to_string()` on the colored string (which strips ANSI by default when piped).

**Difficulty:** Low.

---

### Phase 4 — Edge Cases & Hardening (3 tasks)

#### 4.1 Handle PROB groups without time windows

**Why:** Current `colourise_forecast_period()` renders PROB groups with `{start}/{end}` times, but some TAFs issue PROB without explicit time ranges (only a validity window). The code falls through to bare `PROB30` without time info, which is correct — but verify this doesn't panic.

**Changes:** Verify with a test fixture (`probnodt-taf.json`). No code change likely needed since `start_time`/`end_time` are `Option`.

**Difficulty:** Trivial — just add a test.

#### 4.2 Test with real AVWX API responses

**Why:** Test data was hand-crafted. Real API responses may have extra fields, different nesting, or missing optional keys (e.g., `wind_gust` absent).

**Changes:**

- Capture a real API response for a known station (e.g., KJFK) and save as `tests/testdata/kjfk-taf-real.json`.
- Add a test that parses the real response and asserts no panic.
- Run `cargo test` and fix any parse failures.

**Difficulty:** Medium — depends on network/API availability for capture.

#### 4.3 Fuzz/panic test with adversarial JSON

**Why:** Ensure `from_json()` never panics on garbage input.

**Changes:**

- Add a test with completely random JSON blobs.
- Verify `Taf::from_json()` returns `None` (never panics).

**Difficulty:** Low.

---

## Task List (Ordered)

| # | Task | File(s) | Est. Effort |
|---|------|---------|-------------|
| 1 | Add `WindShear` variant + parser | `src/metar.rs`, `src/taf.rs` | 30 min |
| 2 | Add `MaxTemperature`/`MinTemperature` + parser | `src/metar.rs`, `src/taf.rs` | 30 min |
| 3 | Wire `taf_highlight_probability` into display | `src/taf.rs` | 5 min |
| 4 | Store `Units` in `Taf` struct | `src/taf.rs` | 10 min |
| 5 | Add `[taf]` to `config.toml` | `config.toml` | 5 min |
| 6 | Add `--raw` output flag | `src/main.rs` | 15 min |
| 7 | Add edge-case PROB test fixture + test | `tests/testdata/`, `src/taf.rs` | 15 min |
| 8 | Test with real AVWX response | `tests/testdata/`, `src/taf.rs` | 20 min |
| 9 | Fuzz/panic test | `src/taf.rs` | 10 min |
| 10 | Run full `cargo test && cargo clippy` | — | 10 min |
| 11 | Update `taf_architecture.md` | `taf_architecture.md` | 15 min |

**Total estimated effort: ~2.5 hours**

---

## Architecture Doc Refinements

The existing `taf_architecture.md` is good but has these corrections needed:

1. **Remove `WindShear` from `WxField`** — move it to a separate note or mark as "optional future enhancement" since it's rarely used and adds complexity.
2. **Remove `ChangeIndicator` from `WxField`** — the actual implementation embeds period type in `ForecastPeriod`, not as a field variant. Update the doc to match.
3. **Mark `MaxTemperature`/`MinTemperature` as Phase 1 TODOs** — they're real TAF features but not yet implemented.
4. **Update Phase status** — Phases 1-2 are mostly done; rename remaining items to reflect actual gaps.
5. **Add a "Known Divergences" section** documenting where implementation intentionally differs from the original design doc.

Would you like me to apply these refinements to `taf_architecture.md` now?