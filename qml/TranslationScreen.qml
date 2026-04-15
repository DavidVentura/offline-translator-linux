import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15

Item {
    id: root
    property var appBridge
    property var theme
    property bool speechLongPressTriggered: false
    UiScale { id: ui; desktopMode: root.appBridge && root.appBridge.desktop_mode }
    readonly property real imageOverlayButtonSize: appBridge.desktop_mode ? ui.dp(36) : ui.dp(40)
    readonly property real imageOverlayIconSize: appBridge.desktop_mode ? ui.dp(18) : ui.dp(20)
    readonly property real fullscreenOverlayButtonSize: ui.dp(40)
    readonly property real fullscreenOverlayIconSize: ui.dp(20)

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
        anchors.leftMargin: ui.dp(16)
        anchors.rightMargin: ui.dp(16)
        anchors.topMargin: 0
        anchors.bottomMargin: ui.dp(16)
        spacing: ui.dp(12)

        ScrollView {
            visible: !appBridge.image_mode
            Layout.fillWidth: true
            Layout.fillHeight: true
            Layout.preferredHeight: Math.max(ui.dp(180), root.height * 0.38)
            Layout.minimumHeight: ui.dp(120)
            clip: true

            TextArea {
                text: appBridge.input_text
                color: theme.textPrimary
                placeholderText: "Enter text"
                placeholderTextColor: theme.textSecondary
                wrapMode: TextEdit.Wrap
                verticalAlignment: TextEdit.AlignTop
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
            Layout.preferredHeight: Math.min(ui.dp(380), Math.max(ui.dp(220), root.height * 0.42))
            radius: 0
            color: theme.backgroundElevated
            border.color: theme.borderColor
            border.width: 1
            clip: true

            TranslatedImageView {
                anchors.fill: parent
                appBridge: root.appBridge
                imageMargin: ui.dp(12)
                interactive: true
                onImageClicked: appBridge.open_image_viewer()
            }

            Row {
                anchors.top: parent.top
                anchors.right: parent.right
                anchors.margins: ui.dp(12)
                spacing: ui.dp(8)

                Rectangle {
                    width: root.imageOverlayButtonSize
                    height: root.imageOverlayButtonSize
                    radius: width / 2
                    color: shareOverlayMouse.pressed ? "#99000000" : "#80000000"

                    Image {
                        anchors.centerIn: parent
                        width: root.imageOverlayIconSize
                        height: root.imageOverlayIconSize
                        source: appBridge.asset_url("share.svg")
                        sourceSize.width: root.imageOverlayIconSize
                        sourceSize.height: root.imageOverlayIconSize
                    }

                    MouseArea {
                        id: shareOverlayMouse
                        anchors.fill: parent
                        onClicked: root.shareCurrentImage()
                    }
                }

                Rectangle {
                    width: root.imageOverlayButtonSize
                    height: root.imageOverlayButtonSize
                    radius: width / 2
                    color: closeOverlayMouse.pressed ? "#99000000" : "#80000000"

                    Image {
                        anchors.centerIn: parent
                        width: root.imageOverlayIconSize
                        height: root.imageOverlayIconSize
                        source: appBridge.asset_url("close.svg")
                        sourceSize.width: root.imageOverlayIconSize
                        sourceSize.height: root.imageOverlayIconSize
                    }

                    MouseArea {
                        id: closeOverlayMouse
                        anchors.fill: parent
                        onClicked: appBridge.clear_selected_image()
                    }
                }
            }
        }

        Rectangle {
            visible: appBridge.show_missing_card
            Layout.fillWidth: true
            Layout.topMargin: ui.dp(4)
            Layout.bottomMargin: ui.dp(4)
            color: theme.surfaceColor
            radius: ui.dp(8)
            implicitHeight: ui.dp(52)

            Column {
                anchors.left: parent.left
                anchors.leftMargin: ui.dp(16)
                anchors.verticalCenter: parent.verticalCenter
                spacing: ui.dp(2)

                Label {
                    text: "Translate from"
                    color: theme.textSecondary
                    font.pointSize: ui.pt(13)
                }

                Label {
                    text: appBridge.detected_language_name
                    color: theme.textPrimary
                    font.pointSize: ui.pt(16)
                    font.bold: true
                }
            }

            CircularProgress {
                visible: appBridge.detected_language_progress > 0 && appBridge.detected_language_progress < 1
                anchors.right: parent.right
                anchors.rightMargin: ui.dp(16)
                anchors.verticalCenter: parent.verticalCenter
                progress: appBridge.detected_language_progress
                progressColor: theme.accentColor
            }

            Item {
                visible: appBridge.detected_language_progress <= 0 || appBridge.detected_language_progress >= 1
                anchors.right: parent.right
                anchors.rightMargin: ui.dp(8)
                anchors.verticalCenter: parent.verticalCenter
                width: ui.dp(40); height: ui.dp(40)

                Image {
                    anchors.centerIn: parent
                    width: ui.dp(24); height: ui.dp(24)
                    source: appBridge.asset_url("forward.svg")
                    sourceSize.width: ui.dp(24); sourceSize.height: ui.dp(24)
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
            implicitHeight: ui.dp(8)

            Rectangle {
                anchors.horizontalCenter: parent.horizontalCenter
                anchors.verticalCenter: parent.verticalCenter
                width: parent.width / 2
                height: ui.dp(4)
                radius: ui.dp(2)
                color: theme.borderColor
            }
        }

        Item {
            Layout.fillWidth: true
            Layout.fillHeight: true
            Layout.preferredHeight: Math.max(ui.dp(180), root.height * 0.3)
            Layout.minimumHeight: ui.dp(140)

            ScrollView {
                anchors.fill: parent
                anchors.rightMargin: speechButton.visible ? ui.dp(32) : 0
                clip: true

                TextArea {
                    text: appBridge.output_text
                    readOnly: true
                    wrapMode: TextEdit.Wrap
                    verticalAlignment: TextEdit.AlignTop
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
                width: ui.dp(24)
                height: ui.dp(24)

                Image {
                    anchors.centerIn: parent
                    width: ui.dp(22)
                    height: ui.dp(22)
                    source: appBridge.asset_url((appBridge.tts_loading || appBridge.tts_playing) ? "close.svg" : "tts.svg")
                    sourceSize.width: ui.dp(22)
                    sourceSize.height: ui.dp(22)
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
                property bool voicePickerExpanded: false
                x: Math.max(0, speechButton.x - width + speechButton.width)
                y: speechButton.y + speechButton.height + ui.dp(8)
                width: Math.min(ui.dp(220), parent.width - ui.dp(24))
                height: popupContent.implicitHeight
                modal: false
                closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutside
                padding: 0
                onClosed: voicePickerExpanded = false

                background: Rectangle {
                    radius: ui.dp(8)
                    color: theme.surfaceColor
                    border.color: theme.borderColor
                    border.width: 1
                }

                contentItem: Item {
                    id: popupContent
                    implicitWidth: speechOptionsPopup.width
                    implicitHeight: popupColumn.implicitHeight + ui.dp(24)

                    Column {
                        id: popupColumn
                        x: ui.dp(12)
                        y: ui.dp(12)
                        width: Math.max(0, parent.width - ui.dp(24))
                        spacing: ui.dp(12)

                        Label {
                            text: "Playback speed"
                            color: theme.textPrimary
                            font.pointSize: ui.pt(16)
                            font.bold: true
                        }

                        Row {
                            width: parent.width
                            spacing: ui.dp(10)

                            Rectangle {
                                width: ui.dp(28)
                                height: ui.dp(28)
                                radius: ui.dp(8)
                                color: theme.backgroundElevated

                                Label {
                                    anchors.centerIn: parent
                                    text: "-"
                                    color: theme.textPrimary
                                    font.pointSize: ui.pt(18)
                                }

                                MouseArea {
                                    anchors.fill: parent
                                    onClicked: appBridge.set_tts_playback_speed_value(appBridge.tts_playback_speed - 0.1)
                                }
                            }

                            Label {
                                width: parent.width - ui.dp(76)
                                horizontalAlignment: Text.AlignHCenter
                                verticalAlignment: Text.AlignVCenter
                                text: appBridge.tts_playback_speed.toFixed(2) + "x"
                                color: theme.textPrimary
                                font.pointSize: ui.pt(16)
                            }

                            Rectangle {
                                width: ui.dp(28)
                                height: ui.dp(28)
                                radius: ui.dp(8)
                                color: theme.backgroundElevated

                                Label {
                                    anchors.centerIn: parent
                                    text: "+"
                                    color: theme.textPrimary
                                    font.pointSize: ui.pt(18)
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
                            font.pointSize: ui.pt(16)
                            font.bold: true
                        }

                        Column {
                            width: parent.width
                            spacing: ui.dp(6)

                            Rectangle {
                                width: parent.width
                                height: ui.dp(40)
                                radius: ui.dp(8)
                                color: theme.backgroundElevated
                                border.color: theme.borderColor
                                border.width: 1

                                Label {
                                    anchors.left: parent.left
                                    anchors.leftMargin: ui.dp(12)
                                    anchors.right: voiceExpandIndicator.left
                                    anchors.rightMargin: ui.dp(8)
                                    anchors.verticalCenter: parent.verticalCenter
                                    text: appBridge.tts_selected_voice_display_name
                                    color: theme.textPrimary
                                    verticalAlignment: Text.AlignVCenter
                                    elide: Text.ElideRight
                                    font.pointSize: ui.pt(15)
                                }

                                Image {
                                    id: voiceExpandIndicator
                                    anchors.right: parent.right
                                    anchors.rightMargin: ui.dp(10)
                                    anchors.verticalCenter: parent.verticalCenter
                                    visible: appBridge.tts_voice_option_count > 1
                                    source: appBridge.asset_url("expand_more.svg")
                                    width: ui.dp(18)
                                    height: ui.dp(18)
                                    rotation: speechOptionsPopup.voicePickerExpanded ? 180 : 0
                                    sourceSize.width: ui.dp(18)
                                    sourceSize.height: ui.dp(18)
                                }

                                MouseArea {
                                    anchors.fill: parent
                                    enabled: appBridge.tts_voice_option_count > 1
                                    onClicked: speechOptionsPopup.voicePickerExpanded = !speechOptionsPopup.voicePickerExpanded
                                }
                            }

                            Rectangle {
                                visible: speechOptionsPopup.voicePickerExpanded && appBridge.tts_voice_option_count > 1
                                width: parent.width
                                height: visible ? Math.min(ui.dp(222), voiceListView.contentHeight + ui.dp(2)) : 0
                                radius: ui.dp(8)
                                color: theme.surfaceColor
                                border.color: theme.borderColor
                                border.width: 1
                                clip: true

                                ListView {
                                    id: voiceListView
                                    anchors.fill: parent
                                    anchors.margins: ui.dp(1)
                                    clip: true
                                    model: speechOptionsPopup.voicePickerExpanded ? appBridge.tts_voice_options_model : null

                                    ScrollIndicator.vertical: ScrollIndicator { }

                                    delegate: ItemDelegate {
                                        required property string name
                                        required property string display_name

                                        width: voiceListView.width
                                        text: display_name
                                        highlighted: appBridge.tts_selected_voice_name === name
                                        onClicked: {
                                            appBridge.set_tts_voice_name(name)
                                            speechOptionsPopup.voicePickerExpanded = false
                                        }
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
        anchors.margins: ui.dp(24)
        width: ui.dp(64)
        height: ui.dp(64)
        display: AbstractButton.IconOnly
        icon.source: appBridge.asset_url("camera.svg")
        icon.width: ui.dp(28)
        icon.height: ui.dp(28)
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

        Rectangle {
            anchors.top: parent.top
            anchors.left: parent.left
            anchors.margins: ui.dp(16)
            width: root.fullscreenOverlayButtonSize
            height: root.fullscreenOverlayButtonSize
            radius: width / 2
            color: fullscreenBackMouse.pressed ? "#99000000" : "#80000000"

            Image {
                anchors.centerIn: parent
                width: root.fullscreenOverlayIconSize
                height: root.fullscreenOverlayIconSize
                source: appBridge.asset_url("back.svg")
                sourceSize.width: root.fullscreenOverlayIconSize
                sourceSize.height: root.fullscreenOverlayIconSize
            }

            MouseArea {
                id: fullscreenBackMouse
                anchors.fill: parent
                onClicked: appBridge.close_image_viewer()
            }
        }

        Rectangle {
            anchors.top: parent.top
            anchors.right: parent.right
            anchors.margins: ui.dp(16)
            width: root.fullscreenOverlayButtonSize
            height: root.fullscreenOverlayButtonSize
            radius: width / 2
            color: fullscreenShareMouse.pressed ? "#99000000" : "#80000000"

            Image {
                anchors.centerIn: parent
                width: root.fullscreenOverlayIconSize
                height: root.fullscreenOverlayIconSize
                source: appBridge.asset_url("share.svg")
                sourceSize.width: root.fullscreenOverlayIconSize
                sourceSize.height: root.fullscreenOverlayIconSize
            }

            MouseArea {
                id: fullscreenShareMouse
                anchors.fill: parent
                onClicked: root.shareCurrentImage()
            }
        }
    }
}
