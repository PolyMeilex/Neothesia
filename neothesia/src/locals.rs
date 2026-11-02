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

    pub fn hand_auto() -> String {
        match LOCAL.as_str() {
            "pl" => "Auto".into(),
            "de" => "Auto".into(),
            _ => "Auto".into(),
        }
    }

    pub fn hand_left() -> String {
        match LOCAL.as_str() {
            "pl" => "Lewa".into(),
            "de" => "Links".into(),
            _ => "Left".into(),
        }
    }

    pub fn hand_right() -> String {
        match LOCAL.as_str() {
            "pl" => "Prawa".into(),
            "de" => "Rechts".into(),
            _ => "Right".into(),
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

    pub fn keyboard_size() -> String {
        match LOCAL.as_str() {
            "pl" => "Rozmiar klawiatury".into(),
            "de" => "Tastaturgröße".into(),
            _ => "Keyboard Size".into(),
        }
    }

    pub fn keys(count: u8) -> String {
        match LOCAL.as_str() {
            "pl" => format!("{count} klawiszy"),
            "de" => format!("{count} Tasten"),
            _ => format!("{count} Keys"),
        }
    }

    pub fn custom() -> String {
        match LOCAL.as_str() {
            "pl" => "Własny".into(),
            "de" => "Eigene".into(),
            _ => "Custom".into(),
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

    pub fn waterfall() -> String {
        match LOCAL.as_str() {
            "pl" => "Wodospad".into(),
            "de" => "Wasserfall".into(),
            _ => "Waterfall".into(),
        }
    }

    pub fn keyboard() -> String {
        match LOCAL.as_str() {
            "pl" => "Klawiatura".into(),
            "de" => "Tastatur".into(),
            _ => "Keyboard".into(),
        }
    }

    pub fn octave_lines() -> String {
        match LOCAL.as_str() {
            "pl" => "Linie oktaw".into(),
            "de" => "Oktavlinien".into(),
            _ => "Octave Lines".into(),
        }
    }

    pub fn octave_lines_subtitle() -> String {
        match LOCAL.as_str() {
            "pl" => "Pionowe linie przy każdym C i F, ułatwiające odnalezienie klawiszy".into(),
            "de" => "Vertikale Linien bei jedem C und F zur Orientierung".into(),
            _ => "Vertical lines at every C and F to help you find keys".into(),
        }
    }

    pub fn bar_lines() -> String {
        match LOCAL.as_str() {
            "pl" => "Linie taktów".into(),
            "de" => "Taktstriche".into(),
            _ => "Bar Lines".into(),
        }
    }

    pub fn bar_lines_subtitle() -> String {
        match LOCAL.as_str() {
            "pl" => "Pozioma linia na początku każdego taktu".into(),
            "de" => "Horizontale Linie am Anfang jedes Takts".into(),
            _ => "Horizontal line at the start of each measure".into(),
        }
    }

    pub fn note_names() -> String {
        match LOCAL.as_str() {
            "pl" => "Nazwy nut".into(),
            "de" => "Notennamen".into(),
            _ => "Note Names".into(),
        }
    }

    pub fn note_names_subtitle() -> String {
        match LOCAL.as_str() {
            "pl" => "Wyświetlaj nazwę nuty na każdej opadającej nucie".into(),
            "de" => "Notennamen auf jeder fallenden Note anzeigen".into(),
            _ => "Show the note name on each falling note".into(),
        }
    }

    pub fn fingering() -> String {
        match LOCAL.as_str() {
            "pl" => "Palcowanie".into(),
            "de" => "Fingersatz".into(),
            _ => "Fingering".into(),
        }
    }

    pub fn fingering_subtitle() -> String {
        match LOCAL.as_str() {
            "pl" => "Numer palca na każdej opadającej nucie, edycja: pauza i kliknięcie".into(),
            "de" => {
                "Fingernummer auf jeder fallenden Note, bearbeiten: pausieren und anklicken".into()
            }
            _ => "Finger number on each falling note, edit: pause and click a note".into(),
        }
    }

    pub fn fingering_on_keys() -> String {
        match LOCAL.as_str() {
            "pl" => "Palcowanie na klawiszach".into(),
            "de" => "Fingersatz auf Tasten".into(),
            _ => "Fingering On Keys".into(),
        }
    }

    pub fn fingering_on_keys_subtitle() -> String {
        match LOCAL.as_str() {
            "pl" => "Numer palca na aktualnie granym klawiszu".into(),
            "de" => "Fingernummer auf der gerade gespielten Taste".into(),
            _ => "Finger number on the currently played key".into(),
        }
    }

    pub fn naming_style() -> String {
        match LOCAL.as_str() {
            "pl" => "Styl nazw".into(),
            "de" => "Benennungsstil".into(),
            _ => "Naming Style".into(),
        }
    }

    pub fn naming_style_keyboard_subtitle() -> String {
        match LOCAL.as_str() {
            "pl" => "Wybierz sposób nazywania klawiszy".into(),
            "de" => "Wähle, wie die Tasten beschriftet werden".into(),
            _ => "Choose how the keys are labeled".into(),
        }
    }

    pub fn naming_style_waterfall_subtitle() -> String {
        match LOCAL.as_str() {
            "pl" => "Wybierz sposób nazywania opadających nut".into(),
            "de" => "Wähle, wie fallende Noten beschriftet werden".into(),
            _ => "Choose how falling notes are labeled".into(),
        }
    }

    pub fn note_naming_octaves() -> String {
        match LOCAL.as_str() {
            "pl" => "Oktawy".into(),
            "de" => "Oktaven".into(),
            _ => "Octaves".into(),
        }
    }

    pub fn note_naming_letters() -> String {
        match LOCAL.as_str() {
            "pl" => "Angielskie nazwy nut (C, C#, D)".into(),
            "de" => "Englische Notennamen (C, C#, D)".into(),
            _ => "English Note Names (C, C#, D)".into(),
        }
    }

    pub fn note_naming_solfege() -> String {
        match LOCAL.as_str() {
            "pl" => "Solfeż (Do, Di, Mi)".into(),
            "de" => "Solfège (Do, Di, Mi)".into(),
            _ => "Solfege (Do, Di, Mi)".into(),
        }
    }

    pub fn note_naming_latin() -> String {
        match LOCAL.as_str() {
            "pl" => "Łacińskie (Do, Re, Mi)".into(),
            "de" => "Lateinisch (Do, Re, Mi)".into(),
            _ => "Latin (Do, Re, Mi)".into(),
        }
    }

    pub fn key_glow() -> String {
        match LOCAL.as_str() {
            "pl" => "Poświata klawiszy".into(),
            "de" => "Tastenleuchten".into(),
            _ => "Key Glow".into(),
        }
    }

    pub fn key_glow_subtitle() -> String {
        match LOCAL.as_str() {
            "pl" => "Podświetlaj klawisze podczas gry".into(),
            "de" => "Tasten beim Spielen aufleuchten lassen".into(),
            _ => "Light up keys as they are played".into(),
        }
    }

    pub fn chord_name() -> String {
        match LOCAL.as_str() {
            "pl" => "Nazwa akordu".into(),
            "de" => "Akkordname".into(),
            _ => "Chord Name".into(),
        }
    }

    pub fn chord_name_subtitle() -> String {
        match LOCAL.as_str() {
            "pl" => "Wyświetlaj nazwę granego akordu nad klawiszami".into(),
            "de" => "Gespielten Akkord über den Tasten anzeigen".into(),
            _ => "Name the chord you are playing above the keys".into(),
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

    pub fn fingering_hint() -> String {
        match LOCAL.as_str() {
            "pl" => "Palec: 1-5, automatycznie: 0".into(),
            "de" => "Finger: 1-5, automatisch: 0".into(),
            _ => "Finger: 1-5, automatic: 0".into(),
        }
    }

    pub fn finger_set(finger: u8) -> String {
        match LOCAL.as_str() {
            "pl" => format!("Palec {finger}"),
            "de" => format!("Finger {finger}"),
            _ => format!("Finger {finger}"),
        }
    }

    pub fn finger_auto() -> String {
        match LOCAL.as_str() {
            "pl" => "Palec: automatycznie".into(),
            "de" => "Finger: automatisch".into(),
            _ => "Finger: automatic".into(),
        }
    }
}
