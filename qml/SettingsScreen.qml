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

        RowLayout {
            Layout.fillWidth: true

            ToolButton {
                display: AbstractButton.IconOnly
                icon.source: appBridge.asset_url("back.svg")
                icon.width: 24
                icon.height: 24
                text: "Back"
                onClicked: appBridge.back_from_settings()
            }

            Label {
                text: "Settings"
                color: theme.textPrimary
                font.pixelSize: 22
                Layout.fillWidth: true
            }
        }

        Rectangle {
            Layout.fillWidth: true
            height: 1
            color: theme.borderColor
        }

        Button {
            text: "Manage languages"
            onClicked: appBridge.show_manage_languages()
        }

        CheckBox {
            checked: appBridge.disable_auto_detect
            text: "Disable automatic language detection"
            onToggled: appBridge.set_disable_auto_detect_value(checked)
        }
    }
}
