import QtQuick 2.15
import QtQuick.Controls 2.15

Item {
    id: control
    property string label
    property bool checked
    property var theme
    signal toggled(bool checked)

    implicitHeight: 36

    Label {
        anchors.left: parent.left
        anchors.right: sw.left
        anchors.rightMargin: 12
        anchors.verticalCenter: parent.verticalCenter
        text: control.label
        color: theme.textPrimary
        font.pixelSize: 15
        wrapMode: Text.WordWrap
    }

    Switch {
        id: sw
        anchors.right: parent.right
        anchors.verticalCenter: parent.verticalCenter
        checked: control.checked
        onToggled: control.toggled(checked)

        indicator: Rectangle {
            implicitWidth: 48; implicitHeight: 26
            x: sw.leftPadding
            y: sw.height / 2 - height / 2
            radius: 13
            color: sw.checked ? theme.accentColor : "#555"

            Rectangle {
                x: sw.checked ? parent.width - width - 3 : 3
                y: (parent.height - height) / 2
                width: 20; height: 20; radius: 10
                color: "white"
                Behavior on x { NumberAnimation { duration: 150 } }
            }
        }
    }
}
