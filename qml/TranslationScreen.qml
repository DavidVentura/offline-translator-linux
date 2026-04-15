import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15

Item {
    id: root
    property var appBridge
    property var theme

    Loader {
        id: imagePickerLoader
        active: true
        source: appBridge.desktop_mode ? "DesktopImagePicker.qml" : "UbportsImagePicker.qml"

        onLoaded: {
            if (item) {
                item.appBridge = appBridge
            }
        }
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 16
        spacing: 12

        ScrollView {
            visible: !appBridge.image_mode
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true

            TextArea {
                text: appBridge.input_text
                color: theme.textPrimary
                placeholderText: "Enter text"
                wrapMode: TextEdit.Wrap
                selectByMouse: true
                background: Rectangle {
                    color: theme.backgroundColor
                    border.color: theme.borderColor
                    border.width: 1
                }
                onTextChanged: if (text !== appBridge.input_text) appBridge.process_text(text)
            }
        }

        Rectangle {
            visible: appBridge.image_mode
            Layout.fillWidth: true
            Layout.preferredHeight: Math.min(380, Math.max(220, root.height * 0.42))
            radius: 0
            color: theme.backgroundElevated
            border.color: theme.borderColor
            border.width: 1
            clip: true

            Image {
                id: selectedImage
                anchors.fill: parent
                anchors.margins: 12
                source: appBridge.selected_image_url
                fillMode: Image.PreserveAspectFit
                asynchronous: true
                cache: false
                smooth: true

                Item {
                    anchors.fill: parent
                    visible: appBridge.processed_image_width > 0 && appBridge.processed_image_height > 0

                    Item {
                        x: (parent.width - selectedImage.paintedWidth) / 2
                        y: (parent.height - selectedImage.paintedHeight) / 2
                        width: selectedImage.paintedWidth
                        height: selectedImage.paintedHeight

                        Repeater {
                            model: appBridge.image_overlay_model

                            Item {
                                x: block_x * parent.width / appBridge.processed_image_width
                                y: block_y * parent.height / appBridge.processed_image_height
                                width: block_width * parent.width / appBridge.processed_image_width
                                height: block_height * parent.height / appBridge.processed_image_height

                                Text {
                                    anchors.fill: parent
                                    anchors.margins: 2
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
                }
            }

            RoundButton {
                anchors.top: parent.top
                anchors.right: parent.right
                anchors.margins: 12
                width: 36
                height: 36
                text: "X"
                font.pixelSize: 14
                onClicked: appBridge.clear_selected_image()
                background: Rectangle {
                    radius: width / 2
                    color: parent.down ? Qt.darker(theme.surfaceColor, 1.1) : theme.surfaceColor
                    border.color: theme.borderColor
                    border.width: 1
                }
                contentItem: Label {
                    text: parent.text
                    color: theme.textPrimary
                    horizontalAlignment: Text.AlignHCenter
                    verticalAlignment: Text.AlignVCenter
                }
            }
        }

        Rectangle {
            visible: appBridge.show_missing_card
            Layout.fillWidth: true
            color: theme.surfaceColor
            radius: 12
            border.color: theme.borderColor
            implicitHeight: 88

            RowLayout {
                anchors.fill: parent
                anchors.margins: 16
                spacing: 12

                ColumnLayout {
                    Layout.fillWidth: true

                    Label {
                        text: appBridge.detected_language_installed ? "Translate from" : "Missing language"
                        color: theme.textSecondary
                    }

                    Label {
                        text: appBridge.detected_language_name
                        color: theme.textPrimary
                        font.pixelSize: 18
                    }
                }

                ProgressBar {
                    visible: appBridge.detected_language_progress > 0
                    value: appBridge.detected_language_progress
                    from: 0
                    to: 1
                    Layout.preferredWidth: 96
                }

                Button {
                    visible: appBridge.detected_language_progress <= 0
                    text: appBridge.detected_language_installed ? "Use" : "Download"
                    onClicked: appBridge.missing_language_action()
                }
            }
        }

        Rectangle {
            visible: !appBridge.show_missing_card
            Layout.fillWidth: true
            color: "transparent"
            implicitHeight: 8

            Rectangle {
                anchors.horizontalCenter: parent.horizontalCenter
                anchors.verticalCenter: parent.verticalCenter
                width: parent.width / 2
                height: 4
                radius: 2
                color: theme.borderColor
            }
        }

        ScrollView {
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true

            TextArea {
                text: appBridge.output_text
                readOnly: true
                wrapMode: TextEdit.Wrap
                color: theme.textPrimary
                background: Rectangle {
                    color: theme.backgroundColor
                    border.color: theme.borderColor
                    border.width: 1
                }
            }
        }
    }

    RoundButton {
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        anchors.margins: 24
        width: 64
        height: 64
        display: AbstractButton.IconOnly
        icon.source: appBridge.asset_url("camera.svg")
        icon.width: 28
        icon.height: 28
        text: "Camera"
        background: Rectangle {
            radius: width / 2
            color: parent.down ? Qt.darker(theme.accentColor, 1.15) : theme.accentColor
            border.width: 0
        }
        onClicked: if (imagePickerLoader.item) imagePickerLoader.item.open()
    }
}
