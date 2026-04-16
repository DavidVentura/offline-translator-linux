import QtQuick 2.15

Item {
    id: root
    property alias iconSource: icon.source
    property real iconSize: 22
    property real pressedOpacity: 0.55
    property real pressedScale: 0.92
    property real clickFlashOpacity: 0.35
    property real iconVerticalOffset: 0
    property bool flashOnClick: true
    signal clicked()

    width: 24
    height: 24

    Image {
        id: icon
        anchors.centerIn: parent
        anchors.verticalCenterOffset: root.iconVerticalOffset
        width: root.iconSize
        height: root.iconSize
        sourceSize.width: root.iconSize
        sourceSize.height: root.iconSize
        opacity: !root.enabled ? 0.35 : (mouseArea.pressed ? root.pressedOpacity : 1.0)
        scale: mouseArea.pressed && root.enabled ? root.pressedScale : 1.0

        Behavior on opacity {
            NumberAnimation { duration: 90 }
        }

        Behavior on scale {
            NumberAnimation { duration: 90 }
        }
    }

    MouseArea {
        id: mouseArea
        anchors.fill: parent
        enabled: root.enabled
        onClicked: {
            root.clicked()
            if (root.flashOnClick) {
                clickFlash.restart()
            }
        }
    }

    SequentialAnimation {
        id: clickFlash
        running: false

        PropertyAnimation {
            target: icon
            property: "opacity"
            to: root.clickFlashOpacity
            duration: 70
        }

        PropertyAnimation {
            target: icon
            property: "opacity"
            to: 1.0
            duration: 120
        }
    }
}
