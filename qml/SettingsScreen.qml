import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15

Item {
    property var appBridge
    property var theme

    Flickable {
        anchors.fill: parent
        contentWidth: width
        contentHeight: content.implicitHeight
        boundsBehavior: Flickable.StopAtBounds

        ColumnLayout {
            id: content
            width: parent.width
            spacing: 0

            // Header
            Item {
                Layout.fillWidth: true
                Layout.preferredHeight: 56
                Layout.leftMargin: 16
                Layout.rightMargin: 16

                Item {
                    anchors.left: parent.left
                    anchors.verticalCenter: parent.verticalCenter
                    width: 32; height: 32

                    Image {
                        anchors.centerIn: parent
                        width: 24; height: 24
                        source: appBridge.asset_url("back.svg")
                        sourceSize.width: 24; sourceSize.height: 24
                    }

                    MouseArea {
                        anchors.fill: parent
                        onClicked: appBridge.back_from_settings()
                    }
                }

                Label {
                    anchors.left: parent.left
                    anchors.leftMargin: 40
                    anchors.verticalCenter: parent.verticalCenter
                    text: "Settings"
                    color: theme.textPrimary
                    font.pixelSize: 24
                    font.bold: true
                }
            }

            Item { Layout.preferredHeight: 12 }

            // Languages card
            Rectangle {
                Layout.fillWidth: true
                Layout.leftMargin: 16
                Layout.rightMargin: 16
                implicitHeight: langCol.implicitHeight + 32
                radius: 12
                color: theme.surfaceColor

                ColumnLayout {
                    id: langCol
                    anchors.left: parent.left
                    anchors.right: parent.right
                    anchors.top: parent.top
                    anchors.margins: 16
                    spacing: 12

                    Label {
                        text: "Languages"
                        color: theme.accentColor
                        font.pixelSize: 18
                        font.bold: true
                    }

                    Item {
                        Layout.fillWidth: true
                        implicitHeight: 28

                        Label {
                            anchors.left: parent.left
                            anchors.verticalCenter: parent.verticalCenter
                            text: "Language Packs"
                            color: theme.textPrimary
                            font.pixelSize: 15
                        }

                        Label {
                            anchors.right: parent.right
                            anchors.verticalCenter: parent.verticalCenter
                            text: "Manage"
                            color: theme.accentColor
                            font.pixelSize: 15

                            MouseArea {
                                anchors.fill: parent
                                cursorShape: Qt.PointingHandCursor
                                onClicked: appBridge.show_manage_languages()
                            }
                        }
                    }
                }
            }

            Item { Layout.preferredHeight: 16 }

            // General card
            Rectangle {
                Layout.fillWidth: true
                Layout.leftMargin: 16
                Layout.rightMargin: 16
                implicitHeight: generalCol.implicitHeight + 32
                radius: 12
                color: theme.surfaceColor

                ColumnLayout {
                    id: generalCol
                    anchors.left: parent.left
                    anchors.right: parent.right
                    anchors.top: parent.top
                    anchors.margins: 16
                    spacing: 16

                    Label {
                        text: "General"
                        color: theme.accentColor
                        font.pixelSize: 18
                        font.bold: true
                    }

                    Item {
                        Layout.fillWidth: true
                        implicitHeight: 32

                        Label {
                            anchors.left: parent.left
                            anchors.verticalCenter: parent.verticalCenter
                            text: "Disable automatic language detection"
                            color: theme.textPrimary
                            font.pixelSize: 15
                        }

                        Switch {
                            anchors.right: parent.right
                            anchors.verticalCenter: parent.verticalCenter
                            checked: appBridge.disable_auto_detect
                            onToggled: appBridge.set_disable_auto_detect_value(checked)

                            indicator: Rectangle {
                                implicitWidth: 48
                                implicitHeight: 26
                                x: parent.leftPadding
                                y: parent.height / 2 - height / 2
                                radius: 13
                                color: parent.checked ? theme.accentColor : "#555"

                                Rectangle {
                                    x: parent.parent.checked ? parent.width - width - 3 : 3
                                    y: (parent.height - height) / 2
                                    width: 20; height: 20
                                    radius: 10
                                    color: "white"

                                    Behavior on x { NumberAnimation { duration: 150 } }
                                }
                            }
                        }
                    }
                }
            }

            Item { Layout.preferredHeight: 32 }
        }
    }
}
