import QtQuick 2.15

Text {
    id: root
    required property var target
    property string placeholderText: ""
    property color placeholderColor: "#808080"

    visible: !!target && !target.activeFocus && (!target.text || target.text.length === 0)
    x: target ? target.x + target.leftPadding : 0
    y: target ? target.y + target.topPadding : 0
    width: target ? Math.max(0, target.width - target.leftPadding - target.rightPadding) : 0
    text: placeholderText
    color: placeholderColor
    font: target ? target.font : font
    wrapMode: Text.Wrap
    z: target ? target.z + 1 : 1
}
