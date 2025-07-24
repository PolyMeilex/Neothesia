use std::sync::{Arc, OnceLock, RwLock};

use glyphon::cosmic_text;

/// Returns the global [`FontSystem`].
pub fn font_system() -> &'static RwLock<FontSystem> {
    static FONT_SYSTEM: OnceLock<RwLock<FontSystem>> = OnceLock::new();

    FONT_SYSTEM.get_or_init(|| {
        RwLock::new(FontSystem {
            raw: cosmic_text::FontSystem::new_with_fonts([
                glyphon::fontdb::Source::Binary(Arc::new(include_bytes!(
                    "../../iced-graphics/fonts/Iced-Icons.ttf"
                ))),
                glyphon::fontdb::Source::Binary(Arc::new(include_bytes!(
                    "./render/text/Roboto-Regular.ttf"
                ))),
                glyphon::fontdb::Source::Binary(Arc::new(include_bytes!(
                    "../../neothesia/src/iced_utils/bootstrap-icons.ttf"
                ))),
            ]),
        })
    })
}

/// A set of system fonts.
pub struct FontSystem {
    raw: cosmic_text::FontSystem,
}

impl FontSystem {
    /// Returns the raw [`cosmic_text::FontSystem`].
    pub fn raw(&mut self) -> &mut cosmic_text::FontSystem {
        &mut self.raw
    }

    /// Returns the current [`Version`] of the [`FontSystem`].
    ///
    /// Loading a font will increase the version of a [`FontSystem`].
    pub fn version(&self) -> Version {
        // We don't support updating fonts
        Version(0)
    }
}

/// A version number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Version(u32);
