use std::sync::LazyLock;

static LOCAL: LazyLock<String> = LazyLock::new(|| {
    let locale = sys_locale::get_locale().unwrap_or_else(|| "en-US".to_string());
    // sys_locale says that this is suposed to be a BCP 47 tag, so I supose this split is fine
    // eg. "pl-PL" -> "pl"
    locale.split('-').next().unwrap_or("en").to_lowercase()
});

pub mod menu {
    use super::LOCAL;

    pub fn exit_question() -> String {
        match LOCAL.as_str() {
            "pl" => "Czy chcesz wyjść?".into(),
            "de" => "Möchtest Du die App verlassen?".into(),
            _ => "Do you want to exit?".into(),
        }
    }

    pub fn no() -> String {
        match LOCAL.as_str() {
            "pl" => "Nie".into(),
            "de" => "Nein".into(),
            _ => "No".into(),
        }
    }

    pub fn yes() -> String {
        match LOCAL.as_str() {
            "pl" => "Tak".into(),
            "de" => "Ja".into(),
            _ => "Yes".into(),
        }
    }

    pub fn loading() -> String {
        match LOCAL.as_str() {
            "pl" => "Ładowanie...".into(),
            "de" => "Wird geladen...".into(),
            _ => "Loading...".into(),
        }
    }

    pub fn select_file() -> String {
        match LOCAL.as_str() {
            "pl" => "Wybierz plik".into(),
            "de" => "Datei auswählen".into(),
            _ => "Select File".into(),
        }
    }

    pub fn settings() -> String {
        match LOCAL.as_str() {
            "pl" => "Ustawienia".into(),
            "de" => "Einstellungen".into(),
            _ => "Settings".into(),
        }
    }

    pub fn exit() -> String {
        match LOCAL.as_str() {
            "pl" => "Wyjdź".into(),
            "de" => "Beenden".into(),
            _ => "Exit".into(),
        }
    }

    pub fn freeplay() -> String {
        match LOCAL.as_str() {
            "pl" => "Swobodna gra".into(),
            "de" => "Freies Spiel".into(),
            _ => "FreePlay".into(),
        }
    }

    pub fn play() -> String {
        match LOCAL.as_str() {
            "pl" => "Graj".into(),
            "de" => "Abspielen".into(),
            _ => "Play".into(),
        }
    }

    pub fn tracks() -> String {
        match LOCAL.as_str() {
            "pl" => "Ścieżki".into(),
            "de" => "Stücke".into(),
            _ => "Tracks".into(),
        }
    }
}

pub mod freeplay {
    use super::LOCAL;
    use std::path::Path;

    pub fn no_note_events_recorded() -> String {
        match LOCAL.as_str() {
            "pl" => "Nie nagrano żadnych nut".into(),
            "de" => "Keine Notenereignisse aufgezeichnet".into(),
            _ => "No note events recorded".into(),
        }
    }

    pub fn failed_to_write_midi_file() -> String {
        match LOCAL.as_str() {
            "pl" => "Nie udało się zapisać pliku MIDI".into(),
            "de" => "MIDI-Datei konnte nicht geschrieben werden".into(),
            _ => "Failed to write MIDI file".into(),
        }
    }

    pub fn recording(seconds: f32) -> String {
        match LOCAL.as_str() {
            "pl" => format!("Nagrywanie {seconds:.1}s"),
            "de" => format!("Aufnahme läuft ({seconds:.1} s)"),
            _ => format!("Recording {seconds:.1}s"),
        }
    }

    pub fn recorded(seconds: f32) -> String {
        match LOCAL.as_str() {
            "pl" => format!("Nagrano {seconds:.1}s"),
            "de" => format!("Aufnahme abgeschlossen ({seconds:.1} s)"),
            _ => format!("Recorded {seconds:.1}s"),
        }
    }

    pub fn saved_recording_to(path: &Path) -> String {
        match LOCAL.as_str() {
            "pl" => format!("Zapisano nagranie w {}", path.display()),
            "de" => format!("Aufnahme gespeichert unter: {}", path.display()),
            _ => format!("Saved recording to {}", path.display()),
        }
    }
}

pub mod settings {
    use super::LOCAL;

    pub fn output() -> String {
        match LOCAL.as_str() {
            "pl" => "Wyjście".into(),
            "de" => "Ausgang".into(),
            _ => "Output".into(),
        }
    }

    pub fn input() -> String {
        match LOCAL.as_str() {
            "pl" => "Wejście".into(),
            "de" => "Eingang".into(),
            _ => "Input".into(),
        }
    }

    pub fn note_range() -> String {
        match LOCAL.as_str() {
            "pl" => "Zakres nut".into(),
            "de" => "Notenumfang".into(),
            _ => "Note Range".into(),
        }
    }

    pub fn range_start() -> String {
        match LOCAL.as_str() {
            "pl" => "Początek".into(),
            "de" => "Start".into(),
            _ => "Start".into(),
        }
    }

    pub fn range_end() -> String {
        match LOCAL.as_str() {
            "pl" => "Koniec".into(),
            "de" => "Ende".into(),
            _ => "End".into(),
        }
    }

    pub fn render() -> String {
        match LOCAL.as_str() {
            "pl" => "Renderowanie".into(),
            "de" => "Aussehen".into(),
            _ => "Render".into(),
        }
    }

    pub fn vertical_guidelines() -> String {
        match LOCAL.as_str() {
            "pl" => "Pionowe linie pomocnicze".into(),
            "de" => "Vertikale Hilfslinien".into(),
            _ => "Vertical Guidelines".into(),
        }
    }

    pub fn vertical_guidelines_subtitle() -> String {
        match LOCAL.as_str() {
            "pl" => "Wyświetlaj znaczniki oktaw".into(),
            "de" => "Oktavindikatoren anzeigen".into(),
            _ => "Display octave indicators".into(),
        }
    }

    pub fn horizontal_guidelines() -> String {
        match LOCAL.as_str() {
            "pl" => "Poziome linie pomocnicze".into(),
            "de" => "Horizontale Hilfslinien".into(),
            _ => "Horizontal Guidelines".into(),
        }
    }

    pub fn horizontal_guidelines_subtitle() -> String {
        match LOCAL.as_str() {
            "pl" => "Wyświetlaj znaczniki taktów".into(),
            _ => "Display measure/bar indicators".into(),
        }
    }

    pub fn glow() -> String {
        match LOCAL.as_str() {
            "pl" => "Poświata".into(),
            "de" => "Leuchteffekt".into(),
            _ => "Glow".into(),
        }
    }

    pub fn glow_subtitle() -> String {
        match LOCAL.as_str() {
            "pl" => "Efekt poświaty klawiszy".into(),
            "de" => "Tasten-Leuchteffekt".into(),
            _ => "Key glow effect".into(),
        }
    }

    pub fn note_labels() -> String {
        match LOCAL.as_str() {
            "pl" => "Nazwy nut".into(),
            "de" => "Notenbeschriftungen".into(),
            _ => "Note Labels".into(),
        }
    }

    pub fn note_labels_subtitle() -> String {
        match LOCAL.as_str() {
            "pl" => "Wyświetlaj nazwy nut na opadających nutach".into(),
            "de" => "Notenbeschriftungen im Wasserfall anzeigen".into(),
            _ => "Display waterfall note labels".into(),
        }
    }

    pub fn chord_identifier() -> String {
        match LOCAL.as_str() {
            "pl" => "Rozpoznawanie akordów".into(),
            "de" => "Akkorde".into(),
            _ => "Chord Identifier".into(),
        }
    }

    pub fn chord_identifier_subtitle() -> String {
        match LOCAL.as_str() {
            "pl" => "Wyświetlaj akord nad klawiaturą".into(),
            "de" => "Akkorde über dem Keyboard anzeigen".into(),
            _ => "Display chord above keyboard".into(),
        }
    }

    pub fn soundfont() -> String {
        match LOCAL.as_str() {
            "pl" => "SoundFont".into(),
            "de" => "SoundFont".into(),
            _ => "SoundFont".into(),
        }
    }

    pub fn select_file() -> String {
        match LOCAL.as_str() {
            "pl" => "Wybierz plik".into(),
            "de" => "Datei auswählen".into(),
            _ => "Select File".into(),
        }
    }

    pub fn audio_gain() -> String {
        match LOCAL.as_str() {
            "pl" => "Wzmocnienie dźwięku".into(),
            "de" => "Audio-Verstärkung".into(),
            _ => "Audio Gain".into(),
        }
    }

    pub fn separate_channels() -> String {
        match LOCAL.as_str() {
            "pl" => "Oddzielne kanały".into(),
            "de" => "Kanäle trennen".into(),
            _ => "Separate Channels".into(),
        }
    }

    pub fn separate_channels_subtitle() -> String {
        match LOCAL.as_str() {
            "pl" => "Przypisz osobny kanał MIDI do każdej ścieżki".into(),
            "de" => "Jeder Spur einen eigenen MIDI-Kanal zuweisen".into(),
            _ => "Assign different MIDI channel to each track".into(),
        }
    }

    pub fn auto_detect() -> String {
        match LOCAL.as_str() {
            "pl" => "Automatyczne wykrywanie".into(),
            "de" => "Automatische Erkennung".into(),
            _ => "Auto-detect".into(),
        }
    }

    pub fn auto_detect_subtitle() -> String {
        match LOCAL.as_str() {
            "pl" => "Automatycznie wykryj zakres podłączonej klawiatury".into(),
            "de" => "Tonumfang vom angeschlossenen Keyboard automatisch erkennen".into(),
            _ => "Auto-detect range from connected keyboard".into(),
        }
    }

    pub fn detect() -> String {
        match LOCAL.as_str() {
            "pl" => "Wykryj".into(),
            "de" => "Erkennen".into(),
            _ => "Detect".into(),
        }
    }

    pub fn detection_subtitle() -> String {
        match LOCAL.as_str() {
            "pl" => "Naciśnij skrajnie lewy i skrajnie prawy klawisz na klawiaturze...".into(),
            "de" => {
                "Spiele die ganz linke und die ganz rechts liegende Taste auf Deinem Keyboard..."
                    .into()
            }
            _ => "Play the far-left and far-right key on your keyboard...".into(),
        }
    }

    pub fn cancel() -> String {
        match LOCAL.as_str() {
            "pl" => "Anuluj".into(),
            "de" => "Abbrechen".into(),
            _ => "Cancel".into(),
        }
    }

    pub fn detection_in_progress(step: usize, total: usize) -> String {
        match LOCAL.as_str() {
            "pl" => format!("Wykrywanie w toku... Krok {step}/{total}"),
            "de" => format!("Erkennung läuft... Schritt {step}/{total}"),
            _ => format!("Detection in progress... Step {step}/{total}"),
        }
    }
}

pub mod playing {
    use super::LOCAL;

    pub fn speed(speed: f32) -> String {
        match LOCAL.as_str() {
            "pl" => format!("Prędkość: {speed}"),
            "de" => format!("Geschwindigkeit: {speed}"),
            _ => format!("Speed: {speed}"),
        }
    }

    pub fn animation_speed(speed: f32) -> String {
        match LOCAL.as_str() {
            "pl" => format!("Prędkość animacji: {speed}"),
            "de" => format!("Animationsgeschwindigkeit: {speed}"),
            _ => format!("Animation Speed: {speed}"),
        }
    }

    pub fn offset(offset: f32) -> String {
        match LOCAL.as_str() {
            "pl" => format!("Przesunięcie: {offset}"),
            "de" => format!("Offset: {offset}"),
            _ => format!("Offset: {offset}"),
        }
    }
}
