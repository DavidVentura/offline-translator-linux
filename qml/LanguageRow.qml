import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15

Rectangle {
    required property string code
    required property string name
    required property string size
    required property real download_progress
    property bool built_in: false
    property bool installed: false
    property var appBridge
    property var theme

    Layout.fillWidth: true
    color: theme.surfaceColor
    radius: 12
    border.color: theme.borderColor
    implicitHeight: 72

    RowLayout {
        anchors.fill: parent
        anchors.margins: 16
        spacing: 12

        ColumnLayout {
            Layout.fillWidth: true

            Label {
                text: parent.parent.name
                color: parent.parent.parent.theme.textPrimary
                font.pixelSize: 16
            }

            Label {
                text: parent.parent.parent.built_in ? "Built in" : parent.parent.parent.size
                color: parent.parent.parent.theme.textSecondary
                font.pixelSize: 13
            }
        }

        ProgressBar {
            visible: parent.parent.download_progress > 0
            value: parent.parent.download_progress
            from: 0
            to: 1
            Layout.preferredWidth: 96
        }

        Button {
            visible: parent.parent.download_progress <= 0
            enabled: !parent.parent.built_in || !parent.parent.installed
            text: parent.parent.installed ? "Delete" : "Download"
            onClicked: {
                if (parent.parent.installed) {
                    parent.parent.appBridge.delete_language(parent.parent.code)
                } else {
                    parent.parent.appBridge.download_language(parent.parent.code)
                }
            }
        }
    }
}
