import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15

ScrollView {
    id: root
    property var appBridge
    property var theme
    property var installedModel
    property var availableModel
    property bool desktopMode: false
    clip: true
    UiScale { id: ui; desktopMode: root.desktopMode }

    ColumnLayout {
        width: parent.width
        spacing: ui.dp(12)

        Label {
            visible: installedModel.rowCount() > 0
            text: "Installed"
            color: theme.textPrimary
            font.pointSize: ui.pt(18)
        }

        Repeater {
            model: installedModel

            delegate: LanguageRow {
                required property string code
                required property string name
                required property string size
                required property bool built_in
                required property real download_progress

                appBridge: parent.parent.parent.parent.appBridge
                theme: parent.parent.parent.parent.theme
                desktopMode: parent.parent.parent.parent.desktopMode
                installed: true
            }
        }

        Label {
            visible: availableModel.rowCount() > 0
            text: "Available"
            color: theme.textPrimary
            font.pointSize: ui.pt(18)
        }

        Repeater {
            model: availableModel

            delegate: LanguageRow {
                required property string code
                required property string name
                required property string size
                required property real download_progress

                appBridge: parent.parent.parent.parent.appBridge
                theme: parent.parent.parent.parent.theme
                desktopMode: parent.parent.parent.parent.desktopMode
            }
        }
    }
}
