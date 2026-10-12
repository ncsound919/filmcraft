//! Camera slate parsed from a clip's own metadata.
//!
//! Cameras and phone camera apps write a slate — camera, reel, scene, take, shot,
//! ISO, aperture, lens — into the file, so an editor can sort and group footage
//! without anyone typing it. FilmCraft parses these bytes already (see
//! [`crate::Metadata`], filled by the `keys`/`ilst` key table and the QuickTime
//! `cnam` tag), but the parsed pairs are raw reverse-DNS strings scattered over
//! vendors. This module turns them into one typed, vendor-neutral record.
//!
//! It only *reads* what a file already carries. It never writes metadata back
//! into media, and it never invents a value: a field is `None` unless the file
//! says so.
//!
//! ## Keys it understands
//!
//! Two namespaces appear in the wild and both are read:
//!
//! - `com.blackmagic-design.*` — Blackmagic Camera (phone) and Blackmagic cameras.
//! - `com.apple.proapps.*` — the Apple ProRes convention Blackmagic also writes.
//! - `com.apple.quicktime.*` — standard QuickTime keys (`creationdate`, `model`,
//!   `software`, `location.ISO6709`).
//!
//! Several keys are aliases of the same fact (`cameraId` and `cameraNumber`,
//! `scene` under both namespaces). The first match in the table wins, so the
//! order below is the precedence.

use crate::Metadata;

/// The slate a clip carries. Every field is optional: a file that records only a
/// creation date yields exactly one `Some`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CameraSlate {
    /// Slate camera letter/name, e.g. `A`.
    pub camera: Option<String>,
    /// Numeric camera identifier as the recorder assigned it.
    pub camera_id: Option<String>,
    pub reel: Option<String>,
    pub scene: Option<String>,
    pub take: Option<String>,
    pub shot: Option<String>,
    /// "Day" / "Night".
    pub day_night: Option<String>,
    /// "Interior" / "Exterior".
    pub environment: Option<String>,
    /// The recorder's own good-take flag ("1"/"0" in the file, here a bool).
    pub good_take: Option<bool>,
    /// Lens as reported, e.g. `samsung SM-S938U 22mm`.
    pub lens: Option<String>,
    pub manufacturer: Option<String>,
    pub camera_model: Option<String>,
    /// ISO as a number, when it parses.
    pub iso: Option<u32>,
    /// Aperture as written, e.g. `f1.7`.
    pub aperture: Option<String>,
    /// Shutter speed as written, e.g. `1 / 59.0` or `1/2000`.
    pub shutter_speed: Option<String>,
    pub shutter_angle: Option<String>,
    pub white_balance_kelvin: Option<u32>,
    pub white_balance_tint: Option<i32>,
    pub gamma: Option<String>,
    pub color_transfer: Option<String>,
    pub color_standard: Option<String>,
    /// Free-form log note written by the operator.
    pub log_note: Option<String>,
    /// ISO 6709 location, e.g. `+35.7497816-78.4631977/`.
    pub location: Option<String>,
    /// When the clip was recorded, as written in the file.
    pub date_recorded: Option<String>,
    /// The app or program that wrote the file.
    pub software: Option<String>,
}

impl CameraSlate {
    /// True when the file recorded nothing this struct understands.
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }

    /// The fields that are set, as `(name, value)` for display or JSON. Values are owned
    /// strings so numeric fields can be formatted without borrowing a temporary.
    pub fn entries(&self) -> Vec<(&'static str, String)> {
        fn put(out: &mut Vec<(&'static str, String)>, k: &'static str, v: &Option<String>) {
            if let Some(s) = v {
                out.push((k, s.clone()));
            }
        }
        let mut out: Vec<(&'static str, String)> = Vec::new();
        put(&mut out, "camera", &self.camera);
        put(&mut out, "cameraId", &self.camera_id);
        put(&mut out, "reel", &self.reel);
        put(&mut out, "scene", &self.scene);
        put(&mut out, "take", &self.take);
        put(&mut out, "shot", &self.shot);
        put(&mut out, "dayNight", &self.day_night);
        put(&mut out, "environment", &self.environment);
        if let Some(g) = self.good_take {
            out.push(("goodTake", if g { "1".to_string() } else { "0".to_string() }));
        }
        put(&mut out, "lens", &self.lens);
        put(&mut out, "manufacturer", &self.manufacturer);
        put(&mut out, "model", &self.camera_model);
        if let Some(v) = self.iso {
            out.push(("iso", v.to_string()));
        }
        put(&mut out, "aperture", &self.aperture);
        put(&mut out, "shutterSpeed", &self.shutter_speed);
        put(&mut out, "shutterAngle", &self.shutter_angle);
        if let Some(v) = self.white_balance_kelvin {
            out.push(("whiteBalanceKelvin", v.to_string()));
        }
        if let Some(v) = self.white_balance_tint {
            out.push(("whiteBalanceTint", v.to_string()));
        }
        put(&mut out, "gamma", &self.gamma);
        put(&mut out, "colorTransfer", &self.color_transfer);
        put(&mut out, "colorStandard", &self.color_standard);
        put(&mut out, "logNote", &self.log_note);
        put(&mut out, "location", &self.location);
        put(&mut out, "dateRecorded", &self.date_recorded);
        put(&mut out, "software", &self.software);
        out
    }
}

/// Build a slate from a demuxer's raw metadata pairs.
pub fn slate_from_metadata(md: &Metadata) -> CameraSlate {
    let get = |k: &str| md.get(k).map(str::trim).filter(|v| !v.is_empty()).map(str::to_string);
    let get_u = |k: &str| get(k).and_then(|v| v.parse::<u32>().ok());
    let get_i = |k: &str| get(k).and_then(|v| v.parse::<i32>().ok());
    let get_bool = |k: &str| {
        get(k).and_then(|v| match v.as_str() {
            "1" | "true" | "True" | "yes" => Some(true),
            "0" | "false" | "False" | "no" => Some(false),
            _ => None,
        })
    };

    CameraSlate {
        // cameraNumber is the human letter; cameraId the recorder's number.
        camera: get("com.blackmagic-design.camera.cameraNumber").or_else(|| get("com.apple.proapps.cameraName")),
        camera_id: get("com.blackmagic-design.camera.cameraId"),
        reel: get("com.blackmagic-design.camera.reelNumber").or_else(|| get("com.apple.proapps.reel")),
        scene: get("com.apple.proapps.scene").or_else(|| get("com.blackmagic-design.camera.scene")),
        take: get("com.blackmagic-design.camera.take"),
        shot: get("com.apple.proapps.shot"),
        day_night: get("com.blackmagic-design.camera.dayNight"),
        environment: get("com.blackmagic-design.camera.environment"),
        good_take: get_bool("com.blackmagic-design.camera.goodTake").or_else(|| get_bool("com.apple.proapps.isGood")),
        lens: get("com.blackmagic-design.camera.lensType"),
        manufacturer: get("com.blackmagic-design.camera.manufacturer").or_else(|| get("com.apple.proapps.manufacturer")),
        camera_model: get("com.apple.quicktime.model"),
        iso: get_u("com.blackmagic-design.camera.iso"),
        aperture: get("com.blackmagic-design.camera.aperture"),
        shutter_speed: get("com.blackmagic-design.shutterSpeed"),
        shutter_angle: get("com.blackmagic-design.camera.shutterAngle"),
        white_balance_kelvin: get_u("com.blackmagic-design.camera.whiteBalanceKelvin"),
        white_balance_tint: get_i("com.blackmagic-design.camera.whiteBalanceTint"),
        gamma: get("com.blackmagic-design.camera.gamma"),
        color_transfer: get("com.blackmagic-design.camera.video.colorTransfer"),
        color_standard: get("com.blackmagic-design.camera.video.colorStandard"),
        log_note: get("com.blackmagic-design.camera.logNote"),
        location: get("com.apple.quicktime.location.ISO6709").or_else(|| get("©xyz")),
        date_recorded: get("com.blackmagic-design.camera.dateRecorded").or_else(|| get("com.apple.quicktime.creationdate")),
        software: get("com.apple.quicktime.software"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn md(pairs: &[(&str, &str)]) -> Metadata {
        Metadata { entries: pairs.iter().map(|(k, v)| ((*k).to_string(), (*v).to_string())).collect() }
    }

    #[test]
    fn empty_metadata_is_an_empty_slate() {
        let s = slate_from_metadata(&Metadata { entries: Vec::new() });
        assert!(s.is_empty());
        assert!(s.entries().is_empty());
    }

    #[test]
    fn blackmagic_camera_slate_reads_every_field() {
        let m = md(&[
            ("com.blackmagic-design.camera.cameraNumber", "A"),
            ("com.blackmagic-design.camera.cameraId", "0"),
            ("com.blackmagic-design.camera.reelNumber", "1"),
            ("com.apple.proapps.scene", "12"),
            ("com.blackmagic-design.camera.take", "3"),
            ("com.apple.proapps.shot", "2A"),
            ("com.blackmagic-design.camera.dayNight", "Day"),
            ("com.blackmagic-design.camera.environment", "Interior"),
            ("com.blackmagic-design.camera.goodTake", "1"),
            ("com.blackmagic-design.camera.lensType", "samsung SM-S938U 22mm"),
            ("com.blackmagic-design.camera.manufacturer", "samsung"),
            ("com.apple.quicktime.model", "samsung SM-S938U 22mm"),
            ("com.blackmagic-design.camera.iso", "482"),
            ("com.blackmagic-design.camera.aperture", "f1.7"),
            ("com.blackmagic-design.shutterSpeed", "1 / 59.0"),
            ("com.blackmagic-design.camera.whiteBalanceKelvin", "6093"),
            ("com.blackmagic-design.camera.whiteBalanceTint", "-12"),
            ("com.blackmagic-design.camera.gamma", "BT.2020 HLG10"),
            ("com.blackmagic-design.camera.video.colorTransfer", "HLG"),
            ("com.apple.quicktime.location.ISO6709", "+35.7497816-78.4631977/"),
            ("com.apple.quicktime.software", "Blackmagic Cam 3.5.0.0066"),
        ]);
        let s = slate_from_metadata(&m);
        assert_eq!(s.camera.as_deref(), Some("A"));
        assert_eq!(s.camera_id.as_deref(), Some("0"));
        assert_eq!(s.scene.as_deref(), Some("12"));
        assert_eq!(s.take.as_deref(), Some("3"));
        assert_eq!(s.shot.as_deref(), Some("2A"));
        assert_eq!(s.iso, Some(482));
        assert_eq!(s.aperture.as_deref(), Some("f1.7"));
        assert_eq!(s.white_balance_kelvin, Some(6093));
        assert_eq!(s.white_balance_tint, Some(-12));
        assert_eq!(s.good_take, Some(true));
        assert_eq!(s.day_night.as_deref(), Some("Day"));
        assert_eq!(s.environment.as_deref(), Some("Interior"));
        assert_eq!(s.gamma.as_deref(), Some("BT.2020 HLG10"));
        assert_eq!(s.location.as_deref(), Some("+35.7497816-78.4631977/"));
        assert_eq!(s.software.as_deref(), Some("Blackmagic Cam 3.5.0.0066"));
        assert!(!s.is_empty());
    }

    #[test]
    fn aliases_fall_through_and_numeric_fields_reject_garbage() {
        let m = md(&[
            ("com.blackmagic-design.camera.cameraNumber", "B"),
            ("com.apple.proapps.cameraName", "A"),
            ("com.apple.proapps.reel", "7"),
            ("com.blackmagic-design.camera.iso", "not-a-number"),
            ("com.blackmagic-design.camera.goodTake", "maybe"),
            ("com.blackmagic-design.camera.whiteBalanceTint", "x"),
        ]);
        let s = slate_from_metadata(&m);
        // cameraNumber wins over the ProApps alias.
        assert_eq!(s.camera.as_deref(), Some("B"));
        // reelNumber absent, so the ProApps reel is used.
        assert_eq!(s.reel.as_deref(), Some("7"));
        // unparsable numbers become None, not zero.
        assert_eq!(s.iso, None);
        assert_eq!(s.good_take, None);
        assert_eq!(s.white_balance_tint, None);
    }

    #[test]
    fn entries_are_displayable_strings() {
        let m = md(&[("com.blackmagic-design.camera.iso", "800"), ("com.blackmagic-design.camera.goodTake", "0")]);
        let s = slate_from_metadata(&m);
        let e = s.entries();
        assert!(e.contains(&("iso", "800".to_string())));
        assert!(e.contains(&("goodTake", "0".to_string())));
    }
}
