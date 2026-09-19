use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    #[serde(default = "default_locale")]
    pub locale: String,
    #[serde(default = "default_glass")]
    pub glass: bool,
    #[serde(default = "default_transparency")]
    pub transparency: f64,
    pub trigger: String,
    pub game_key: String,
    pub mode: String,
    pub up: String,
    pub down: String,
    pub left: String,
    pub right: String,
    pub press_ms: u64,
    pub gap_ms: u64,
    pub size: f64,
    pub deadzone: f64,
    pub sensitivity: f64,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Node {
    pub id: String,
    pub kind: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub name_key: String,
    #[serde(default)]
    pub stratagem_id: String,
    #[serde(default)]
    pub code: Vec<String>,
    #[serde(default)]
    pub icon: String,
    #[serde(default)]
    pub children: Vec<Node>,
}
#[derive(Clone, Deserialize, Serialize)]
pub struct Config {
    pub settings: Settings,
    pub root: Node,
}
fn default_locale() -> String {
    "en".into()
}
fn default_glass() -> bool {
    true
}
fn default_transparency() -> f64 {
    35.0
}

// Physical scan codes match KeyboardEvent.code, including AZERTY layouts.
pub fn scan(code: &str) -> Option<(u16, bool)> {
    let value = match code {
        "ControlLeft" => (0x1d, false),
        "ControlRight" => (0x1d, true),
        "AltLeft" => (0x38, false),
        "AltRight" => (0x38, true),
        "ShiftLeft" => (0x2a, false),
        "ShiftRight" => (0x36, false),
        "Tab" => (0x0f, false),
        "Space" => (0x39, false),
        "ArrowUp" => (0x48, true),
        "ArrowDown" => (0x50, true),
        "ArrowLeft" => (0x4b, true),
        "ArrowRight" => (0x4d, true),
        "KeyA" => (0x1e, false),
        "KeyB" => (0x30, false),
        "KeyC" => (0x2e, false),
        "KeyD" => (0x20, false),
        "KeyE" => (0x12, false),
        "KeyF" => (0x21, false),
        "KeyG" => (0x22, false),
        "KeyH" => (0x23, false),
        "KeyI" => (0x17, false),
        "KeyJ" => (0x24, false),
        "KeyK" => (0x25, false),
        "KeyL" => (0x26, false),
        "KeyM" => (0x32, false),
        "KeyN" => (0x31, false),
        "KeyO" => (0x18, false),
        "KeyP" => (0x19, false),
        "KeyQ" => (0x10, false),
        "KeyR" => (0x13, false),
        "KeyS" => (0x1f, false),
        "KeyT" => (0x14, false),
        "KeyU" => (0x16, false),
        "KeyV" => (0x2f, false),
        "KeyW" => (0x11, false),
        "KeyX" => (0x2d, false),
        "KeyY" => (0x15, false),
        "KeyZ" => (0x2c, false),
        "Digit1" => (2, false),
        "Digit2" => (3, false),
        "Digit3" => (4, false),
        "Digit4" => (5, false),
        "Digit5" => (6, false),
        "Digit6" => (7, false),
        "Digit7" => (8, false),
        "Digit8" => (9, false),
        "Digit9" => (10, false),
        "Digit0" => (11, false),
        "F11" => (0x57, false),
        "F12" => (0x58, false),
        _ => {
            if let Some(f) = code.strip_prefix('F').and_then(|s| s.parse::<u16>().ok()) {
                if (1..=10).contains(&f) {
                    (0x3a + f, false)
                } else {
                    return None;
                }
            } else {
                return None;
            }
        }
    };
    Some(value)
}
impl Config {
    pub fn validate(&self) -> Result<(), String> {
        let s = &self.settings;
        let keys = [&s.trigger, &s.game_key, &s.up, &s.down, &s.left, &s.right];
        for (i, key) in keys.iter().enumerate() {
            if scan(key).is_none()
                && !(i == 0 && (key.as_str() == "Mouse4" || key.as_str() == "Mouse5"))
            {
                return Err("invalidKey".into());
            }
            if keys[..i].contains(key) {
                return Err("duplicateKeys".into());
            }
        }
        if !(15..=250).contains(&s.press_ms)
            || !s.transparency.is_finite()
            || !(0.0..=80.0).contains(&s.transparency)
            || !(100..=300).contains(&s.gap_ms)
            || !s.size.is_finite()
            || !(400.0..=720.0).contains(&s.size)
            || !s.deadzone.is_finite()
            || !(30.0..=100.0).contains(&s.deadzone)
            || !s.sensitivity.is_finite()
            || !(0.3..=3.0).contains(&s.sensitivity)
            || !["hold", "toggle"].contains(&s.mode.as_str())
        {
            return Err("invalidRange".into());
        }
        fn walk(n: &Node, depth: usize) -> bool {
            depth <= 8
                && if n.kind == "folder" {
                    n.children.len() <= 12 && n.children.iter().all(|n| walk(n, depth + 1))
                } else {
                    n.kind == "stratagem"
                        && !n.code.is_empty()
                        && n.code.len() <= 32
                        && n.code
                            .iter()
                            .all(|s| ["Up", "Down", "Left", "Right"].contains(&s.as_str()))
                }
        }
        if self.root.kind != "folder" || !walk(&self.root, 0) {
            return Err("invalidTree".into());
        }
        Ok(())
    }
}
pub fn sector(x: f64, y: f64, count: usize, deadzone: f64) -> Option<usize> {
    if count == 0 || x.hypot(y) < deadzone {
        return None;
    }
    let step = std::f64::consts::TAU / count as f64;
    Some(
        ((x.atan2(-y) + step / 2.0).rem_euclid(std::f64::consts::TAU) / step).floor() as usize
            % count,
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cardinal_selection() {
        assert_eq!(sector(0., -100., 4, 70.), Some(0));
        assert_eq!(sector(100., 0., 4, 70.), Some(1));
        assert_eq!(sector(0., 100., 4, 70.), Some(2));
        assert_eq!(sector(-100., 0., 4, 70.), Some(3));
        assert_eq!(sector(5., 5., 4, 70.), None);
        assert_eq!(sector(100., 0., 0, 70.), None);
    }
    #[test]
    fn physical_keys() {
        assert_eq!(scan("KeyQ"), Some((0x10, false)));
        assert_eq!(scan("ArrowUp"), Some((0x48, true)));
        assert_eq!(scan("F12"), Some((0x58, false)));
        assert_eq!(scan("F13"), None);
    }
}
