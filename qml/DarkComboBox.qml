import QtQuick 2.15
import QtQuick.Controls 2.15

ComboBox {
    id: control
    property var theme
    property string iconSource

    contentItem: Label {
        leftPadding: 12
        rightPadding: control.indicator ? control.indicator.width + 10 : 30
        text: control.displayText
        color: theme.textPrimary
        verticalAlignment: Text.AlignVCenter
        elide: Text.ElideRight
    }

    background: Rectangle {
        radius: 8
        color: theme.backgroundElevated
        border.width: 1
        border.color: theme.borderColor
    }

    indicator: Image {
        source: iconSource
        width: 16; height: 16
        x: control.width - width - 10
        y: (control.height - height) / 2
    }

    popup: Popup {
        y: control.height
        width: control.width
        implicitHeight: contentItem.implicitHeight
        padding: 1

        contentItem: ListView {
            clip: true
            implicitHeight: contentHeight
            model: parent.visible ? control.model : null

            delegate: Rectangle {
                width: control.width
                height: 36
                color: delegateMouseArea.containsMouse ? theme.surfaceAltColor : theme.surfaceColor

                Label {
                    anchors.fill: parent
                    leftPadding: 12
                    text: modelData
                    color: theme.textPrimary
                    verticalAlignment: Text.AlignVCenter
                }

                MouseArea {
                    id: delegateMouseArea
                    anchors.fill: parent
                    hoverEnabled: true
                    onClicked: {
                        control.currentIndex = index
                        control.activated(index)
                        control.popup.close()
                    }
                }
            }
        }

        background: Rectangle {
            color: theme.surfaceColor
            border.color: theme.borderColor
            radius: 4
        }
    }
}
