use crate::{Element, ElementsMap, Point, Size};

#[derive(Default, Debug, Clone)]
pub struct ElementDebug {
    pub border: [(Point, Size); 4],
    pub bounding_box: (Point, Size),
}

/// Build debug elements, for elements borders & bounding boxes
pub fn iter_elements<Msg>(
    elements: &mut ElementsMap<Msg>,
) -> impl Iterator<Item = (&Element<Msg>, ElementDebug)> {
    elements.iter().map(|(_id, element)| {
        let pos = element.rect().origin;
        let size = element.rect().size;

        let v = 1.0;

        let left = (pos, Size::new(v, size.height));
        let right = (
            Point::new(pos.x + size.width - v, pos.y),
            Size::new(v, size.height),
        );

        let top = (pos, Size::new(size.width, v));
        let bottom = (
            Point::new(pos.x, pos.y + size.height - v),
            Size::new(size.width, v),
        );

        (
            element,
            ElementDebug {
                border: [left, right, top, bottom],
                bounding_box: (pos, size),
            },
        )
    })
}
