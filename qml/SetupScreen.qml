import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15

Item {
    id: root
    property var appBridge
    property var theme

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 16
        spacing: 12

        RowLayout {
            Layout.fillWidth: true

            Label {
                text: "Language Setup"
                color: theme.textPrimary
                font.pixelSize: 22
                Layout.fillWidth: true
            }

            ToolButton {
                display: AbstractButton.IconOnly
                icon.source: appBridge.asset_url("settings.svg")
                icon.width: 24
                icon.height: 24
                text: "Settings"
                onClicked: appBridge.show_settings()
            }
        }

        Label {
            Layout.fillWidth: true
            text: "Download language packs to start translating"
            color: theme.textSecondary
            wrapMode: Text.WordWrap
        }

        LanguageCatalogBrowser {
            Layout.fillWidth: true
            Layout.fillHeight: true
            appBridge: root.appBridge
            theme: root.theme
        }

        Button {
            Layout.fillWidth: true
            enabled: appBridge.has_languages
            text: "Done"
            onClicked: appBridge.finish_language_setup()
        }
    }
}
