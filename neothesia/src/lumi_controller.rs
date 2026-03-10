/// ROLI LUMI Keys SysEx controller.
/// Byte sequences taken from https://github.com/benob/LUMI-lights/blob/master/SYSEX.txt
///
/// All SysEx commands follow the format:
/// F0 00 21 10  77 37  <8 cmd bytes>  <checksum>  F7
use crate::output_manager::OutputConnection;

pub const ROLI_MANUFACTURER_ID: [u8; 3] = [0x00, 0x21, 0x10];

/// Compute the LUMI checksum over the 8-byte command payload.
/// From SYSEX.txt:  c = size; for b in bytes: c = (c*3 + b) & 0xff;  return c & 0x7f
pub fn compute_checksum(values: &[u8]) -> u8 {
    let mut sum: usize = values.len();
    for &val in values {
        sum = (sum * 3 + val as usize) & 0xFF;
    }
    (sum & 0x7F) as u8
}

// ---------------------------------------------------------------------------
// BitArray - used for the global-color (single-color-mode) encoding only
// ---------------------------------------------------------------------------
#[derive(Debug, Default)]
pub struct BitArray {
    values: Vec<u8>,
    num_bits: usize,
}

impl BitArray {
    pub fn new() -> Self { Self::default() }

    pub fn append(&mut self, mut value: u32, mut size: usize) {
        let mut used_bits = self.num_bits % 7;
        let mut packed = if used_bits > 0 { self.values.pop().unwrap_or(0) } else { 0 };
        self.num_bits += size;
        while size > 0 {
            let space_left = 7 - used_bits;
            packed |= ((value << used_bits) & 0x7F) as u8;
            if size >= space_left {
                size -= space_left;
                value >>= space_left;
                self.values.push(packed);
                packed = 0;
                used_bits = 0;
            } else {
                self.values.push(packed);
                break;
            }
        }
    }

    pub fn get_padded_8bytes(&mut self) -> Vec<u8> {
        while self.values.len() < 8 { self.values.push(0); }
        self.values.clone()
    }
}

// ---------------------------------------------------------------------------
// Standalone helpers — send a single command immediately, no LumiController
// needed. Used from the menu settings page so the hardware responds in real time.
// ---------------------------------------------------------------------------

fn build_sysex_msg(payload: &[u8; 8]) -> Vec<u8> {
    let checksum = compute_checksum(payload);
    let mut msg = Vec::with_capacity(13);
    msg.push(0xF0);
    msg.extend_from_slice(&ROLI_MANUFACTURER_ID); // 00 21 10
    msg.push(0x77);
    msg.push(0x37);
    msg.extend_from_slice(payload);
    msg.push(checksum);
    msg.push(0xF7);
    msg
}

/// Send a Night-Mode / color-mode command immediately to the connected LUMI.
/// `mode`: 0=Rainbow, 1=Single Color, 2=Piano, 3=Night
pub fn lumi_send_color_mode(connection: &crate::output_manager::OutputConnection, mode: u8) {
    let mut bits = BitArray::new();
    bits.append(0x10, 7);
    bits.append(0x40, 7);
    bits.append(0b00010, 5);
    bits.append((mode & 3) as u32, 2);
    let v = bits.get_padded_8bytes();
    let mut arr = [0u8; 8];
    arr.copy_from_slice(&v[..8]);
    connection.send_sysex(&build_sysex_msg(&arr));
}

/// Send a brightness command immediately.
/// `value`: 0-100 (percent), matching SYSEX.txt range.
/// Use `lumi_brightness_from_u8(raw)` to convert from a 0-127 UI slider value.
pub fn lumi_send_brightness(connection: &crate::output_manager::OutputConnection, value_0_100: u8) {
    let mut bits = BitArray::new();
    bits.append(0x10, 7);
    bits.append(0x40, 7);
    bits.append(0b00100, 5);
    bits.append(value_0_100 as u32, 7);
    let v = bits.get_padded_8bytes();
    let mut arr = [0u8; 8];
    arr.copy_from_slice(&v[..8]);
    connection.send_sysex(&build_sysex_msg(&arr));
}

/// Convert a UI slider value (0-127) to the 0-100 range expected by the LUMI hardware.
pub fn lumi_brightness_from_u8(raw: u8) -> u8 {
    (raw as u32 * 100 / 127) as u8
}

// ---------------------------------------------------------------------------
// LumiController
// ---------------------------------------------------------------------------
pub struct LumiController {
    connection: OutputConnection,
    /// Cache: last RGB sent per MIDI note (None = off)
    key_colors: [Option<(u8, u8, u8)>; 128],
    current_mode: u8,       // 255 = uninitialized
    current_brightness: u8, // 255 = uninitialized
}

impl LumiController {
    pub fn new(connection: OutputConnection) -> Self {
        let mut ctrl = Self {
            connection,
            key_colors: [None; 128],
            current_mode: 255,
            current_brightness: 255,
        };
        ctrl.set_color_mode(3); // Night mode - clear default rainbow
        ctrl
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    /// Build and send a full LUMI SysEx message.
    ///   F0 00 21 10  77 37  <payload 8 bytes>  <checksum>  F7
    fn send_sysex_command(&mut self, payload: &[u8; 8]) {
        let checksum = compute_checksum(payload);
        let mut msg = Vec::with_capacity(13);
        msg.push(0xF0);
        msg.extend_from_slice(&ROLI_MANUFACTURER_ID);  // 00 21 10
        msg.push(0x77);
        msg.push(0x37);
        msg.extend_from_slice(payload);
        msg.push(checksum);
        msg.push(0xF7);
        self.connection.send_sysex(&msg);
    }

    // -----------------------------------------------------------------------
    // Global device configuration
    // -----------------------------------------------------------------------

    /// Set the keyboard color mode using BitArray encoding.
    /// 0 = Rainbow, 1 = Single Color, 2 = Piano, 3 = Night
    /// Verified against SYSEX.txt:
    ///   0 → 10 40 02 00 … (rainbow)
    ///   3 → 10 40 62 00 … (night)
    pub fn set_color_mode(&mut self, mode: u8) {
        if self.current_mode == mode { return; }
        self.current_mode = mode;

        let mut bits = BitArray::new();
        bits.append(0x10, 7);
        bits.append(0x40, 7);
        bits.append(0b00010, 5);
        bits.append((mode & 3) as u32, 2);
        let payload = bits.get_padded_8bytes();
        let mut arr = [0u8; 8];
        arr.copy_from_slice(&payload[..8]);
        self.send_sysex_command(&arr);
    }

    /// Set the keyboard LED brightness.
    /// `value` is 0-100 (percent), matching the ranges from SYSEX.txt.
    /// Verified: 0→`10 40 04 00`, 25→`10 40 24 06`, 50→`10 40 44 0C`, 100→`10 40 04 19`
    pub fn set_brightness(&mut self, value: u8) {
        // Map 0-127 UI range to 0-100 hardware range
        let hw_val = (value as u32 * 100 / 127) as u32;
        let hw_key = hw_val as u8;
        if self.current_brightness == hw_key { return; }
        self.current_brightness = hw_key;

        let mut bits = BitArray::new();
        bits.append(0x10, 7);
        bits.append(0x40, 7);
        bits.append(0b00100, 5);
        bits.append(hw_val, 7);
        let payload = bits.get_padded_8bytes();
        let mut arr = [0u8; 8];
        arr.copy_from_slice(&payload[..8]);
        self.send_sysex_command(&arr);
    }

    // -----------------------------------------------------------------------
    // Per-key LED color (uses BitArray encoding from lumiSysexLib.js)
    // `set_color` there encodes `0x20 + 0x10 * (id & 1)` as the second field
    // -----------------------------------------------------------------------

    /// Send a per-key RGB color using the same BitArray encoding as `lumiSysexLib.js`.
    /// `key_index` is 0-based index within the LUMI block (0 for first key).
    fn send_key_color_cmd(&mut self, key_index: u8, r: u8, g: u8, b: u8) {
        let mut bits = BitArray::new();
        bits.append(0x10, 7);
        bits.append(0x20 + 0x10 * (key_index as u32 & 1), 7);
        bits.append(0b00100, 5);
        bits.append(b as u32, 8);
        bits.append(g as u32, 8);
        bits.append(r as u32, 8);
        bits.append(0b11111111, 8);
        let payload = bits.get_padded_8bytes();
        let mut arr = [0u8; 8];
        arr.copy_from_slice(&payload[..8]);
        self.send_sysex_command(&arr);
    }

    /// Set an individual MIDI note's LED to an exact RGB color.
    /// Maps `note` to a 0-23 LUMI block key index (base note = 48, one octave block).
    pub fn set_key_color(&mut self, note: u8, r: u8, g: u8, b: u8) {
        if note > 127 { return; }

        let new_col = (r, g, b);
        if self.key_colors[note as usize] == Some(new_col) { return; }
        self.key_colors[note as usize] = Some(new_col);

        // Map absolute MIDI note to LUMI 0-based key index.
        // LUMI default octave start = 48 (C3)
        if note < 48 || note > 71 { return; } // single block: C3-B4
        let key_index = note - 48;
        self.send_key_color_cmd(key_index, r, g, b);
    }

    /// Same as `set_key_color` but at ~40% brightness (soft hint).
    pub fn set_key_dim(&mut self, note: u8, r: u8, g: u8, b: u8) {
        self.set_key_color(note, r.saturating_mul(2) / 5, g.saturating_mul(2) / 5, b.saturating_mul(2) / 5);
    }

    /// Turn off an individual key LED.
    pub fn clear_key(&mut self, note: u8) {
        if note > 127 { return; }
        if self.key_colors[note as usize].is_none() { return; }
        // Temporarily allow the cache to accept (0,0,0)
        self.key_colors[note as usize] = Some((1, 1, 1)); // force re-send
        self.set_key_color(note, 0, 0, 0);
        self.key_colors[note as usize] = None; // mark truly off
    }

    /// Clear all LEDs (send black to every key).
    pub fn clear_all(&mut self) {
        for note in 48u8..=71 {
            self.clear_key(note);
        }
    }
}

// ---------------------------------------------------------------------------
// Unit tests — verify BitArray matches SYSEX.txt byte sequences exactly
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    fn make_color_mode_payload(mode: u8) -> [u8; 8] {
        let mut bits = BitArray::new();
        bits.append(0x10, 7);
        bits.append(0x40, 7);
        bits.append(0b00010, 5);
        bits.append((mode & 3) as u32, 2);
        let v = bits.get_padded_8bytes();
        let mut arr = [0u8; 8];
        arr.copy_from_slice(&v[..8]);
        arr
    }

    fn make_brightness_payload(pct: u8) -> [u8; 8] {
        let mut bits = BitArray::new();
        bits.append(0x10, 7);
        bits.append(0x40, 7);
        bits.append(0b00100, 5);
        bits.append(pct as u32, 7);
        let v = bits.get_padded_8bytes();
        let mut arr = [0u8; 8];
        arr.copy_from_slice(&v[..8]);
        arr
    }

    /// Color mode – from SYSEX.txt:
    ///   rainbow:       10 40 02 00 …
    ///   single color:  10 40 22 00 …
    ///   piano:         10 40 42 00 …
    ///   night:         10 40 62 00 …
    #[test]
    fn color_mode_rainbow()       { assert_eq!(&make_color_mode_payload(0)[..4], &[0x10, 0x40, 0x02, 0x00]); }
    #[test]
    fn color_mode_single_color()  { assert_eq!(&make_color_mode_payload(1)[..4], &[0x10, 0x40, 0x22, 0x00]); }
    #[test]
    fn color_mode_piano()         { assert_eq!(&make_color_mode_payload(2)[..4], &[0x10, 0x40, 0x42, 0x00]); }
    #[test]
    fn color_mode_night()         { assert_eq!(&make_color_mode_payload(3)[..4], &[0x10, 0x40, 0x62, 0x00]); }

    /// Brightness – from SYSEX.txt (input is 0-100 percent directly):
    ///   0%   → 10 40 04 00 …
    ///   25%  → 10 40 24 06 …
    ///   50%  → 10 40 44 0C …
    ///   75%  → 10 40 64 12 …
    ///   100% → 10 40 04 19 …
    #[test]
    fn brightness_0pct()   { assert_eq!(&make_brightness_payload(0)[..4],   &[0x10, 0x40, 0x04, 0x00]); }
    #[test]
    fn brightness_25pct()  { assert_eq!(&make_brightness_payload(25)[..4],  &[0x10, 0x40, 0x24, 0x06]); }
    #[test]
    fn brightness_50pct()  { assert_eq!(&make_brightness_payload(50)[..4],  &[0x10, 0x40, 0x44, 0x0C]); }
    #[test]
    fn brightness_75pct()  { assert_eq!(&make_brightness_payload(75)[..4],  &[0x10, 0x40, 0x64, 0x12]); }
    #[test]
    fn brightness_100pct() { assert_eq!(&make_brightness_payload(100)[..4], &[0x10, 0x40, 0x04, 0x19]); }
}
