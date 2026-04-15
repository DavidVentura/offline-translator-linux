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

        TabBar {
            Layout.fillWidth: true
            currentIndex: appBridge.active_tab

            TabButton {
                text: "Languages"
                onClicked: appBridge.set_active_tab(0)
            }

            TabButton {
                text: "Dictionaries"
                onClicked: appBridge.set_active_tab(1)
            }
        }

        StackLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            currentIndex: appBridge.active_tab

            LanguageListView {
                appBridge: parent.parent.parent.appBridge
                theme: parent.parent.parent.theme
                installedModel: parent.parent.parent.installedModel
                availableModel: parent.parent.parent.availableModel
            }

            Item {
                Label {
                    anchors.centerIn: parent
                    text: "Dictionaries coming soon..."
                    color: parent.parent.parent.theme.textSecondary
                }
            }
        }

        Button {
            Layout.fillWidth: true
            enabled: appBridge.has_languages
            text: "Done"
            onClicked: appBridge.finish_language_setup()
        }
    }
}
