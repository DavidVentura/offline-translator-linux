import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15

Item {
    id: root
    property var appBridge
    property var theme

    function shareCurrentImage() {
        if (imageShareLoader.item) {
            imageShareLoader.item.share(appBridge.selected_image_url)
        }
    }

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

    Loader {
        id: imageShareLoader
        active: true
        source: appBridge.desktop_mode ? "DesktopImageShare.qml" : "UbportsImageShare.qml"

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

            TranslatedImageView {
                anchors.fill: parent
                appBridge: root.appBridge
                imageMargin: 12
                interactive: true
                onImageClicked: appBridge.open_image_viewer()
            }

            Row {
                anchors.top: parent.top
                anchors.right: parent.right
                anchors.margins: 12
                spacing: 8

                RoundButton {
                    width: 36
                    height: 36
                    display: AbstractButton.IconOnly
                    icon.source: appBridge.asset_url("share.svg")
                    icon.width: 18
                    icon.height: 18
                    text: "Share"
                    onClicked: root.shareCurrentImage()
                    background: Rectangle {
                        radius: width / 2
                        color: parent.down ? Qt.darker(theme.surfaceColor, 1.1) : theme.surfaceColor
                        border.color: theme.borderColor
                        border.width: 1
                    }
                }

                RoundButton {
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

    Rectangle {
        anchors.fill: parent
        visible: appBridge.image_viewer_open
        color: "#000000"
        z: 20

        TranslatedImageView {
            anchors.fill: parent
            anchors.topMargin: 56
            appBridge: root.appBridge
            imageMargin: 0
            interactive: false
        }

        Item {
            anchors.top: parent.top
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.margins: 16
            height: 40

            RoundButton {
                anchors.left: parent.left
                anchors.verticalCenter: parent.verticalCenter
                width: 40
                height: 40
                display: AbstractButton.IconOnly
                icon.source: appBridge.asset_url("back.svg")
                icon.width: 20
                icon.height: 20
                text: "Back"
                onClicked: appBridge.close_image_viewer()
                background: Rectangle {
                    radius: width / 2
                    color: "transparent"
                }
            }

            RoundButton {
                anchors.right: parent.right
                anchors.verticalCenter: parent.verticalCenter
                width: 40
                height: 40
                display: AbstractButton.IconOnly
                icon.source: appBridge.asset_url("share.svg")
                icon.width: 20
                icon.height: 20
                text: "Share"
                onClicked: root.shareCurrentImage()
                background: Rectangle {
                    radius: width / 2
                    color: "transparent"
                }
            }
        }
    }
}
