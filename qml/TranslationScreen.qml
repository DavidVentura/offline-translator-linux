import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15

Item {
    id: root
    property var appBridge
    property var theme
    property bool speechLongPressTriggered: false

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
            Layout.preferredHeight: Math.max(180, root.height * 0.38)
            Layout.minimumHeight: 120
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
                    icon.color: theme.textPrimary
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
                    display: AbstractButton.IconOnly
                    icon.source: appBridge.asset_url("close.svg")
                    icon.color: theme.textPrimary
                    icon.width: 18
                    icon.height: 18
                    text: "Close"
                    onClicked: appBridge.clear_selected_image()
                    background: Rectangle {
                        radius: width / 2
                        color: parent.down ? Qt.darker(theme.surfaceColor, 1.1) : theme.surfaceColor
                        border.color: theme.borderColor
                        border.width: 1
                    }
                }
            }
        }

        Rectangle {
            visible: appBridge.show_missing_card
            Layout.fillWidth: true
            Layout.topMargin: 4
            Layout.bottomMargin: 4
            color: theme.surfaceColor
            radius: 8
            implicitHeight: 52

            Column {
                anchors.left: parent.left
                anchors.leftMargin: 16
                anchors.verticalCenter: parent.verticalCenter
                spacing: 2

                Label {
                    text: "Translate from"
                    color: theme.textSecondary
                    font.pixelSize: 13
                }

                Label {
                    text: appBridge.detected_language_name
                    color: theme.textPrimary
                    font.pixelSize: 16
                    font.bold: true
                }
            }

            CircularProgress {
                visible: appBridge.detected_language_progress > 0 && appBridge.detected_language_progress < 1
                anchors.right: parent.right
                anchors.rightMargin: 16
                anchors.verticalCenter: parent.verticalCenter
                progress: appBridge.detected_language_progress
                progressColor: theme.accentColor
            }

            Item {
                visible: appBridge.detected_language_progress <= 0 || appBridge.detected_language_progress >= 1
                anchors.right: parent.right
                anchors.rightMargin: 8
                anchors.verticalCenter: parent.verticalCenter
                width: 40; height: 40

                Image {
                    anchors.centerIn: parent
                    width: 24; height: 24
                    source: appBridge.asset_url("forward.svg")
                    sourceSize.width: 24; sourceSize.height: 24
                }

                MouseArea {
                    anchors.fill: parent
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

        Item {
            Layout.fillWidth: true
            Layout.fillHeight: true
            Layout.preferredHeight: Math.max(180, root.height * 0.3)
            Layout.minimumHeight: 140

            ScrollView {
                anchors.fill: parent
                anchors.rightMargin: (speechButton.visible ? 32 : 0)
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

            Item {
                id: speechButton
                visible: (appBridge.tts_available || appBridge.tts_loading || appBridge.tts_playing)
                         && appBridge.output_text.length > 0
                anchors.top: parent.top
                anchors.right: parent.right
                width: 24
                height: 24

                Image {
                    anchors.centerIn: parent
                    width: 22
                    height: 22
                    source: appBridge.asset_url((appBridge.tts_loading || appBridge.tts_playing) ? "close.svg" : "tts.svg")
                    sourceSize.width: 22
                    sourceSize.height: 22
                }

                MouseArea {
                    anchors.fill: parent
                    pressAndHoldInterval: 450
                    onPressed: root.speechLongPressTriggered = false
                    onPressAndHold: {
                        root.speechLongPressTriggered = true
                        speechOptionsPopup.open()
                    }
                    onClicked: {
                        if (root.speechLongPressTriggered) {
                            root.speechLongPressTriggered = false
                            return
                        }
                        appBridge.toggle_speak_output()
                    }
                }
            }

            Popup {
                id: speechOptionsPopup
                x: Math.max(0, speechButton.x - width + speechButton.width)
                y: speechButton.y + speechButton.height + 8
                width: Math.min(220, parent.width - 24)
                modal: false
                padding: 0

                background: Rectangle {
                    radius: 8
                    color: theme.surfaceColor
                    border.color: theme.borderColor
                    border.width: 1
                }

                contentItem: Item {
                    implicitWidth: speechOptionsPopup.width
                    implicitHeight: popupColumn.implicitHeight + 24

                    Column {
                        id: popupColumn
                        anchors.fill: parent
                        anchors.margins: 12
                        spacing: 12

                        Label {
                            text: "Playback speed"
                            color: theme.textPrimary
                            font.pixelSize: 16
                            font.bold: true
                        }

                        Row {
                            width: parent.width
                            spacing: 10

                            Rectangle {
                                width: 28
                                height: 28
                                radius: 8
                                color: theme.backgroundElevated

                                Label {
                                    anchors.centerIn: parent
                                    text: "-"
                                    color: theme.textPrimary
                                    font.pixelSize: 18
                                }

                                MouseArea {
                                    anchors.fill: parent
                                    onClicked: appBridge.set_tts_playback_speed_value(appBridge.tts_playback_speed - 0.1)
                                }
                            }

                            Label {
                                width: parent.width - 76
                                horizontalAlignment: Text.AlignHCenter
                                verticalAlignment: Text.AlignVCenter
                                text: appBridge.tts_playback_speed.toFixed(2) + "x"
                                color: theme.textPrimary
                                font.pixelSize: 16
                            }

                            Rectangle {
                                width: 28
                                height: 28
                                radius: 8
                                color: theme.backgroundElevated

                                Label {
                                    anchors.centerIn: parent
                                    text: "+"
                                    color: theme.textPrimary
                                    font.pixelSize: 18
                                }

                                MouseArea {
                                    anchors.fill: parent
                                    onClicked: appBridge.set_tts_playback_speed_value(appBridge.tts_playback_speed + 0.1)
                                }
                            }
                        }

                        Rectangle {
                            width: parent.width
                            height: 1
                            color: theme.borderColor
                            opacity: 0.7
                        }

                        Label {
                            text: "Voice"
                            color: theme.textPrimary
                            font.pixelSize: 16
                            font.bold: true
                        }

                        Column {
                            width: parent.width
                            spacing: 6

                            Rectangle {
                                visible: true
                                width: parent.width
                                height: 40
                                radius: 8
                                color: theme.backgroundElevated
                                border.color: theme.borderColor
                                border.width: 1

                                Label {
                                    anchors.left: parent.left
                                    anchors.leftMargin: 12
                                    anchors.right: voicePickerIndicator.left
                                    anchors.rightMargin: 8
                                    anchors.verticalCenter: parent.verticalCenter
                                    text: appBridge.tts_selected_voice_display_name
                                    color: theme.textPrimary
                                    verticalAlignment: Text.AlignVCenter
                                    elide: Text.ElideRight
                                }

                                Image {
                                    id: voicePickerIndicator
                                    anchors.right: parent.right
                                    anchors.rightMargin: 10
                                    anchors.verticalCenter: parent.verticalCenter
                                    visible: appBridge.tts_voice_options_model.count > 1
                                    source: appBridge.asset_url("expand_more.svg")
                                    width: 18
                                    height: 18
                                    sourceSize.width: 18
                                    sourceSize.height: 18
                                }

                                MouseArea {
                                    anchors.fill: parent
                                    enabled: appBridge.tts_voice_options_model.count > 1
                                    onClicked: voicePickerPopup.open()
                                }

                                Popup {
                                    id: voicePickerPopup
                                    y: parent.height + 4
                                    width: parent.width
                                    padding: 1

                                    contentItem: ListView {
                                        clip: true
                                        implicitHeight: Math.min(contentHeight, 220)
                                        model: voicePickerPopup.visible ? appBridge.tts_voice_options_model : null
                                        delegate: ItemDelegate {
                                            required property string name
                                            required property string display_name

                                            width: voicePickerPopup.width - 2
                                            text: display_name
                                            highlighted: appBridge.tts_selected_voice_name === name
                                            onClicked: {
                                                appBridge.set_tts_voice_name(name)
                                                voicePickerPopup.close()
                                            }
                                        }
                                    }

                                    background: Rectangle {
                                        radius: 8
                                        color: theme.surfaceColor
                                        border.color: theme.borderColor
                                        border.width: 1
                                    }
                                }
                            }
                        }
                    }
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
            appBridge: root.appBridge
            imageMargin: 0
            interactive: false
        }

        RoundButton {
            anchors.top: parent.top
            anchors.left: parent.left
            anchors.margins: 16
            width: 44
            height: 44
            display: AbstractButton.IconOnly
            icon.source: appBridge.asset_url("back.svg")
            icon.color: "#FFFFFF"
            icon.width: 22
            icon.height: 22
            text: "Back"
            onClicked: appBridge.close_image_viewer()
            background: Rectangle {
                radius: width / 2
                color: parent.down ? "#99000000" : "#80000000"
                border.width: 0
            }
        }

        RoundButton {
            anchors.top: parent.top
            anchors.right: parent.right
            anchors.margins: 16
            width: 44
            height: 44
            display: AbstractButton.IconOnly
            icon.source: appBridge.asset_url("share.svg")
            icon.color: "#FFFFFF"
            icon.width: 22
            icon.height: 22
            text: "Share"
            onClicked: root.shareCurrentImage()
            background: Rectangle {
                radius: width / 2
                color: parent.down ? "#99000000" : "#80000000"
                border.width: 0
            }
        }
    }
}
