import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15

Item {
    id: root
    property var appBridge
    property var theme
    UiScale { id: ui; desktopMode: root.appBridge && root.appBridge.desktop_mode }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: ui.dp(12)
        spacing: ui.dp(10)

        RowLayout {
            Layout.fillWidth: true
            spacing: ui.dp(12)

            ToolButton {
                display: AbstractButton.IconOnly
                icon.source: appBridge.asset_url("back.svg")
                icon.color: theme.textPrimary
                icon.width: ui.dp(22)
                icon.height: ui.dp(22)
                onClicked: appBridge.back_from_manage_languages()
            }

            Label {
                Layout.fillWidth: true
                text: "Manage languages"
                color: theme.textPrimary
                font.pointSize: ui.pt(24)
                font.bold: true
            }
        }

        LanguageCatalogBrowser {
            Layout.fillWidth: true
            Layout.fillHeight: true
            appBridge: root.appBridge
            desktopMode: root.appBridge.desktop_mode
            theme: root.theme
        }
    }
}
