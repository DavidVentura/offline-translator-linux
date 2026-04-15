import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15

Item {
    property var appBridge
    property var theme
    property var installedModel
    property var availableModel

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
                onClicked: appBridge.back_from_manage_languages()
            }

            Label {
                text: "Manage Languages"
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

        LanguageListView {
            Layout.fillWidth: true
            Layout.fillHeight: true
            appBridge: parent.parent.appBridge
            theme: parent.parent.theme
            installedModel: parent.parent.installedModel
            availableModel: parent.parent.availableModel
        }
    }
}
