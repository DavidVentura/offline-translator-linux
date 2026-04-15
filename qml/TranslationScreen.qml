import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15

Item {
    property var appBridge
    property var theme

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 16
        spacing: 12

        ScrollView {
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
        display: AbstractButton.IconOnly
        icon.source: appBridge.asset_url("camera.svg")
        icon.width: 24
        icon.height: 24
        text: "Camera"
        onClicked: appBridge.camera_clicked()
    }
}
