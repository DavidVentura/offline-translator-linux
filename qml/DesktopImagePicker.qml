import QtQuick 2.15
import QtQuick.Dialogs 1.3

Item {
    property var appBridge

    function open() {
        picker.open()
    }

    FileDialog {
        id: picker
        title: "Choose an image"
        nameFilters: ["Images (*.png *.jpg *.jpeg *.webp *.bmp *.gif *.tif *.tiff)"]
        selectExisting: true
        selectMultiple: false
        onAccepted: appBridge.process_image_selection(fileUrl.toString())
    }
}
