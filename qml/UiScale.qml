import QtQuick 2.15
import QtQuick.Window 2.15

Item {
    id: root
    visible: false
    width: 0
    height: 0
    property bool desktopMode: false

    readonly property real dpiScale: Math.max(1.0, Screen.pixelDensity / (160 / 25.4))
    readonly property real textScale: desktopMode ? 1.0 : 1.6

    function dp(value) {
        return value * dpiScale
    }

    function pt(value) {
        return value * textScale
    }
}
