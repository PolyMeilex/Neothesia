use std::{
    cell::{OnceCell, RefCell},
    rc::Rc,
    sync::Arc,
};

pub use glyphon::FontSystem;

thread_local! {
     static FONT_SYSTEM: OnceCell<Rc<RefCell<FontSystem>>> = const { OnceCell::new() };
}

/// Returns the global [`FontSystem`].
pub fn font_system() -> Rc<RefCell<FontSystem>> {
    FONT_SYSTEM.with(|system| {
        system
            .get_or_init(|| {
                Rc::new(RefCell::new(FontSystem::new_with_fonts([
                    glyphon::fontdb::Source::Binary(Arc::new(include_bytes!(
                        "../../iced-graphics/fonts/Iced-Icons.ttf"
                    ))),
                    glyphon::fontdb::Source::Binary(Arc::new(include_bytes!(
                        "./render/text/Roboto-Regular.ttf"
                    ))),
                    glyphon::fontdb::Source::Binary(Arc::new(include_bytes!(
                        "../../neothesia/src/iced_utils/bootstrap-icons.ttf"
                    ))),
                ])))
            })
            .clone()
    })
}
