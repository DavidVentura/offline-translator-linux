import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15

Item {
    id: root
    required property var appBridge
    required property var theme
    required property string title
    signal backRequested()

    UiScale { id: ui; desktopMode: root.appBridge && root.appBridge.desktop_mode }

    implicitHeight: ui.dp(56)

    RowLayout {
        anchors.fill: parent
        anchors.leftMargin: ui.dp(12)
        anchors.rightMargin: ui.dp(12)
        anchors.topMargin: ui.dp(4)
        anchors.bottomMargin: ui.dp(4)
        spacing: ui.dp(8)

        ToolButton {
            Layout.preferredWidth: ui.dp(36)
            Layout.fillHeight: true
            display: AbstractButton.IconOnly
            icon.source: appBridge.asset_url("back.svg")
            icon.color: theme.textPrimary
            icon.width: ui.dp(20)
            icon.height: ui.dp(20)
            onClicked: root.backRequested()
        }

        Label {
            Layout.fillWidth: true
            text: root.title
            color: theme.textPrimary
            font.pointSize: ui.pageTitlePt
            font.bold: true
            verticalAlignment: Text.AlignVCenter
            elide: Text.ElideRight
        }
    }
}
