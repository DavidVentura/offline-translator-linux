use cpp::cpp;
use qmetaobject::*;

cpp! {{
    #include <QtGui/QImage>
    #include <QtGui/QGuiApplication>
    #include <QtQuick/QQuickWindow>
    #include <QtGui/QScreen>
    #include <QtGui/QPixmap>
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
        if size.width == 0 || size.height == 0 {
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

pub fn save_window_screenshot(path: &str) -> bool {
    let path_ptr = path.as_ptr();
    let path_len = path.len();
    cpp!(unsafe [path_ptr as "const char *", path_len as "size_t"] -> bool as "bool" {
        const QString path = QString::fromUtf8(path_ptr, static_cast<int>(path_len));
        QPixmap shot;
        const auto windows = QGuiApplication::topLevelWindows();
        for (QWindow* window : windows) {
            auto quickWindow = qobject_cast<QQuickWindow*>(window);
            if (!quickWindow || !quickWindow->isVisible()) {
                continue;
            }
            QScreen* screen = quickWindow->screen();
            if (!screen) {
                screen = QGuiApplication::primaryScreen();
            }
            if (!screen) {
                continue;
            }
            shot = screen->grabWindow(quickWindow->winId());
            if (!shot.isNull()) {
                break;
            }
        }
        if (shot.isNull()) {
            return false;
        }
        return shot.save(path);
    })
}
