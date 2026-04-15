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
    readonly property real pageTitlePt: desktopMode ? 20 : pt(24)
    readonly property real listPrimaryPt: desktopMode ? 14 : pt(16)
    readonly property real listSecondaryPt: desktopMode ? 11 : pt(12)
    readonly property real sectionTitlePt: desktopMode ? 13 : pt(14)

    function dp(value) {
        return value * dpiScale
    }

    function pt(value) {
        return value * textScale
    }
}
