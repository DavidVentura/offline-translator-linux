import QtQuick 2.15
import QtQuick.Controls 2.15

Slider {
    id: control
    property var theme

    background: Rectangle {
        x: control.leftPadding
        y: control.topPadding + control.availableHeight / 2 - height / 2
        width: control.availableWidth; height: 4
        radius: 2; color: "#303240"

        Rectangle {
            width: control.visualPosition * parent.width
            height: parent.height; radius: 2
            color: theme.accentColor
        }
    }

    handle: Rectangle {
        x: control.leftPadding + control.visualPosition * (control.availableWidth - width)
        y: control.topPadding + control.availableHeight / 2 - height / 2
        width: 18; height: 18; radius: 9
        color: control.pressed ? Qt.lighter(theme.accentColor, 1.2) : theme.accentColor
    }
}
