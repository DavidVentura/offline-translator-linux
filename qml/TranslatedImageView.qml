import QtQuick 2.15

Item {
    id: root
    property var appBridge
    property int imageMargin: 0
    property bool interactive: false
    signal imageClicked()
    UiScale { id: ui }

    Image {
        id: selectedImage
        anchors.fill: parent
        anchors.margins: root.imageMargin
        source: root.appBridge ? root.appBridge.selected_image_url : ""
        fillMode: Image.PreserveAspectFit
        asynchronous: true
        cache: false
        smooth: true
    }

    Item {
        anchors.fill: parent
        visible: root.appBridge
                 && root.appBridge.processed_image_width > 0
                 && root.appBridge.processed_image_height > 0

        Item {
            id: paintedBounds
            x: (parent.width - selectedImage.paintedWidth) / 2
            y: (parent.height - selectedImage.paintedHeight) / 2
            width: selectedImage.paintedWidth
            height: selectedImage.paintedHeight

            Repeater {
                model: root.appBridge ? root.appBridge.image_overlay_model : null

                Item {
                    x: block_x * parent.width / root.appBridge.processed_image_width
                    y: block_y * parent.height / root.appBridge.processed_image_height
                    width: block_width * parent.width / root.appBridge.processed_image_width
                    height: block_height * parent.height / root.appBridge.processed_image_height

                    Text {
                        anchors.fill: parent
                        anchors.margins: ui.dp(2)
                        text: translated_text
                        color: foreground_color
                        wrapMode: Text.Wrap
                        font.pixelSize: Math.max(10, parent.height * 0.55)
                        elide: Text.ElideRight
                        clip: true
                    }
                }
            }
        }

        MouseArea {
            visible: root.interactive && paintedBounds.width > 0 && paintedBounds.height > 0
            x: paintedBounds.x
            y: paintedBounds.y
            width: paintedBounds.width
            height: paintedBounds.height
            onClicked: root.imageClicked()
        }
    }
}
