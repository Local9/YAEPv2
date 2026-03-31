#[derive(Default)]
pub struct HotkeyService;

impl HotkeyService {
    pub fn capture_start(&self, capture_type: String, target_id: Option<i64>) {
        #[cfg(target_os = "windows")]
        crate::global_hotkeys::begin_hotkey_capture(capture_type, target_id);
        #[cfg(not(target_os = "windows"))]
        {
            let _ = (capture_type, target_id);
        }
    }

    pub fn capture_stop(&self) {
        #[cfg(target_os = "windows")]
        crate::global_hotkeys::end_hotkey_capture();
    }

    pub fn validate_hotkey(&self, hotkey: &str) -> Result<String, String> {
        validate_hotkey_string(hotkey)
    }
}

fn validate_hotkey_string(raw: &str) -> Result<String, String> {
    let value = raw.trim();
    if value.is_empty() {
        return Ok(String::new());
    }

    let tokens: Vec<String> = value
        .split('+')
        .map(|p| p.trim())
        .filter(|p| !p.is_empty())
        .map(|p| p.to_string())
        .collect();
    if tokens.is_empty() {
        return Err("Invalid hotkey format".to_string());
    }

    let mut has_ctrl = false;
    let mut has_alt = false;
    let mut has_shift = false;
    let mut has_win = false;
    let mut key_token: Option<String> = None;

    for token in &tokens {
        let lower = token.to_lowercase();
        match lower.as_str() {
            "ctrl" | "control" => has_ctrl = true,
            "alt" => has_alt = true,
            "shift" => has_shift = true,
            "win" | "windows" => has_win = true,
            _ => {
                if key_token.is_some() {
                    return Err("Hotkey can only contain one non-modifier key".to_string());
                }
                key_token = Some(normalize_key_token(token)?);
            }
        }
    }

    let Some(key) = key_token else {
        return Err("Hotkey must include a key".to_string());
    };

    let mut out = Vec::new();
    if has_ctrl {
        out.push("Ctrl".to_string());
    }
    if has_alt {
        out.push("Alt".to_string());
    }
    if has_shift {
        out.push("Shift".to_string());
    }
    if has_win {
        out.push("Win".to_string());
    }
    out.push(key);
    Ok(out.join("+"))
}

fn normalize_key_token(token: &str) -> Result<String, String> {
    let key = token.trim();
    let upper = key.to_uppercase();

    if upper.len() == 1 {
        let ch = upper.chars().next().unwrap_or_default();
        if ch.is_ascii_uppercase() || ch.is_ascii_digit() {
            return Ok(upper);
        }
    }

    if let Some(num) = upper.strip_prefix('F') {
        if let Ok(value) = num.parse::<u8>() {
            if (1..=24).contains(&value) {
                return Ok(format!("F{value}"));
            }
        }
    }

    // Decimal virtual-key code (RegisterHotKey / LL hook vkCode), e.g. Vk134 == F23.
    if let Some(rest) = upper.strip_prefix("VK") {
        if let Ok(code) = rest.parse::<u32>() {
            if (1..=255).contains(&code) {
                return Ok(format!("Vk{code}"));
            }
        }
    }

    if let Some(num) = upper.strip_prefix("NUMPAD") {
        if let Ok(value) = num.parse::<u8>() {
            if value <= 9 {
                return Ok(format!("NumPad{value}"));
            }
        }
    }

    let canonical = match upper.as_str() {
        "SPACE" => "Space",
        "ENTER" => "Enter",
        "TAB" => "Tab",
        "ESC" | "ESCAPE" => "Escape",
        "BACK" | "BACKSPACE" => "Backspace",
        "DEL" | "DELETE" => "Delete",
        "INS" | "INSERT" => "Insert",
        "HOME" => "Home",
        "END" => "End",
        "PGUP" | "PAGEUP" => "PageUp",
        "PGDN" | "PAGEDOWN" => "PageDown",
        "UP" => "Up",
        "DOWN" => "Down",
        "LEFT" => "Left",
        "RIGHT" => "Right",
        _ => return Err(format!("Unsupported hotkey token: {key}")),
    };
    Ok(canonical.to_string())
}

#[cfg(test)]
mod tests {
    use super::validate_hotkey_string;

    #[test]
    fn normalizes_hotkey_with_modifiers() {
        let result = validate_hotkey_string("control + alt + f13");
        assert_eq!(result.expect("expected normalized hotkey"), "Ctrl+Alt+F13");
    }

    #[test]
    fn allows_single_key_hotkey() {
        let result = validate_hotkey_string("space");
        assert_eq!(result.expect("expected normalized hotkey"), "Space");
    }

    #[test]
    fn rejects_multiple_non_modifier_keys() {
        let result = validate_hotkey_string("Ctrl+A+B");
        assert!(result.is_err());
    }

    #[test]
    fn rejects_unknown_key() {
        let result = validate_hotkey_string("Ctrl+Foo");
        assert!(result.is_err());
    }

    #[test]
    fn normalizes_vk_decimal_token() {
        let result = validate_hotkey_string("Vk134");
        assert_eq!(result.expect("vk token"), "Vk134");
        let with_mod = validate_hotkey_string("Ctrl+Vk135");
        assert_eq!(with_mod.expect("vk with mod"), "Ctrl+Vk135");
    }
}
