use cpp::cpp;
use qmetaobject::*;

cpp! {{
    #include <QtGui/QImage>
    #include <cstring>
}}

#[derive(QObject, Default)]
pub struct RenderedImageItem {
    base: qt_base_class!(trait QQuickPaintedItem),
    image: qt_property!(QImage; WRITE set_image NOTIFY image_changed),
    image_changed: qt_signal!(),
}

impl RenderedImageItem {
    fn set_image(&mut self, image: QImage) {
        if self.image != image {
            self.image = image;
            self.image_changed();
        }
        (self as &dyn QQuickItem).update();
    }
}

impl QQuickItem for RenderedImageItem {}

impl QQuickPaintedItem for RenderedImageItem {
    fn paint(&mut self, painter: &mut QPainter) {
        let size = self.image.size();
        if size.width <= 0 || size.height <= 0 {
            return;
        }

        painter.draw_image_fit_rect(
            (self as &dyn QQuickItem).bounding_rect(),
            self.image.clone(),
        );
    }
}

pub fn qimage_from_rgba_bytes(width: u32, height: u32, rgba_bytes: &[u8]) -> QImage {
    let expected_len = width.saturating_mul(height).saturating_mul(4) as usize;
    if width == 0 || height == 0 || rgba_bytes.len() != expected_len {
        return QImage::default();
    }

    let bytes_ptr = rgba_bytes.as_ptr();
    let bytes_len = rgba_bytes.len();
    cpp!(unsafe [
        width as "int",
        height as "int",
        bytes_ptr as "const unsigned char *",
        bytes_len as "size_t"
    ] -> QImage as "QImage" {
        QImage image(QSize(width, height), QImage::Format_RGBA8888);
        if (!image.isNull()) {
            std::memcpy(image.bits(), bytes_ptr, bytes_len);
        }
        return image;
    })
}
