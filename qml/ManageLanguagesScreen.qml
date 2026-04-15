import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15

Item {
    property var appBridge
    property var theme
    property var manageModel

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 12
        spacing: 10

        RowLayout {
            Layout.fillWidth: true
            spacing: 12

            ToolButton {
                display: AbstractButton.IconOnly
                icon.source: appBridge.asset_url("back.svg")
                icon.width: 22
                icon.height: 22
                onClicked: appBridge.back_from_manage_languages()
            }

            Label {
                Layout.fillWidth: true
                text: "Manage languages"
                color: theme.textPrimary
                font.pixelSize: 24
                font.bold: true
            }
        }

        LanguageCatalogBrowser {
            Layout.fillWidth: true
            Layout.fillHeight: true
            appBridge: parent.parent.appBridge
            theme: parent.parent.theme
            manageModel: parent.parent.manageModel
        }
    }
}
