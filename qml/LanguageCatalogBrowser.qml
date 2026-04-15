import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15

Item {
    property var appBridge
    property var theme
    property var manageModel

    function actionIcon(installed) {
        return installed ? appBridge.asset_url("delete.svg") : appBridge.asset_url("download.svg")
    }

    function featureAction(code, feature, installed) {
        if (installed) {
            appBridge.delete_feature(code, feature)
        } else {
            appBridge.download_feature(code, feature)
        }
    }

    ColumnLayout {
        anchors.fill: parent
        spacing: 10

        TextField {
            Layout.fillWidth: true
            placeholderText: "Filter languages"
            text: appBridge.manage_filter_text
            color: theme.textPrimary
            onTextChanged: appBridge.set_manage_filter(text)
            background: Rectangle {
                radius: 4
                color: theme.backgroundElevated
                border.width: 1
                border.color: theme.borderColor
            }
        }

        ListView {
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            spacing: 2
            model: manageModel

            delegate: Rectangle {
                required property string code
                required property string name
                required property string total_size
                required property bool built_in
                required property bool expanded
                required property bool core_available
                required property bool core_installed
                required property string core_size
                required property real core_progress
                required property bool dictionary_available
                required property bool dictionary_installed
                required property string dictionary_size
                required property real dictionary_progress
                required property bool tts_available
                required property bool tts_installed
                required property string tts_size
                required property real tts_progress

                width: ListView.view.width
                height: rowColumn.implicitHeight + 16
                color: index % 2 === 0 ? theme.backgroundColor : theme.backgroundElevated

                ColumnLayout {
                    id: rowColumn
                    anchors.left: parent.left
                    anchors.right: parent.right
                    anchors.top: parent.top
                    anchors.leftMargin: 10
                    anchors.rightMargin: 10
                    anchors.topMargin: 8
                    spacing: 8

                    RowLayout {
                        Layout.fillWidth: true
                        spacing: 8

                        ToolButton {
                            display: AbstractButton.IconOnly
                            icon.source: expanded ? appBridge.asset_url("expand_less.svg") : appBridge.asset_url("expand_more.svg")
                            icon.width: 18
                            icon.height: 18
                            background: Item {}
                            onClicked: appBridge.toggle_manage_language(code)
                        }

                        ColumnLayout {
                            Layout.fillWidth: true
                            spacing: 0

                            Label {
                                text: name
                                color: theme.textPrimary
                                font.pixelSize: 18
                                font.bold: true
                            }

                            Label {
                                text: total_size
                                color: theme.textSecondary
                                font.pixelSize: 14
                            }
                        }

                        RowLayout {
                            spacing: 6

                            Label {
                                visible: core_available
                                text: "T"
                                color: core_installed ? theme.textPrimary : theme.textSecondary
                                font.bold: true
                                font.pixelSize: 13
                            }

                            Label {
                                visible: dictionary_available
                                text: "D"
                                color: dictionary_installed ? theme.textPrimary : theme.textSecondary
                                font.bold: true
                                font.pixelSize: 13
                            }

                            Label {
                                visible: tts_available
                                text: "S"
                                color: tts_installed ? theme.textPrimary : theme.textSecondary
                                font.bold: true
                                font.pixelSize: 13
                            }
                        }

                        ToolButton {
                            visible: core_available
                            display: AbstractButton.IconOnly
                            icon.source: actionIcon(core_installed)
                            icon.width: 18
                            icon.height: 18
                            onClicked: featureAction(code, 0, core_installed)
                        }
                    }

                    ColumnLayout {
                        visible: expanded
                        Layout.fillWidth: true
                        spacing: 6

                        Item {
                            visible: core_available
                            Layout.fillWidth: true
                            implicitHeight: core_progress > 0 ? 44 : 28

                            ColumnLayout {
                                anchors.fill: parent
                                spacing: 4

                                RowLayout {
                                    Layout.fillWidth: true

                                    Label {
                                        text: "Translation"
                                        color: theme.textPrimary
                                        font.pixelSize: 15
                                    }

                                    Label {
                                        text: core_size
                                        color: theme.textSecondary
                                        font.pixelSize: 14
                                    }

                                    Item {
                                        Layout.fillWidth: true
                                    }

                                    ToolButton {
                                        visible: core_progress <= 0
                                        display: AbstractButton.IconOnly
                                        icon.source: actionIcon(core_installed)
                                        icon.width: 18
                                        icon.height: 18
                                        onClicked: featureAction(code, 0, core_installed)
                                    }
                                }

                                ProgressBar {
                                    visible: core_progress > 0
                                    Layout.fillWidth: true
                                    value: core_progress
                                    from: 0
                                    to: 1
                                }
                            }
                        }

                        Item {
                            visible: dictionary_available
                            Layout.fillWidth: true
                            implicitHeight: dictionary_progress > 0 ? 44 : 28

                            ColumnLayout {
                                anchors.fill: parent
                                spacing: 4

                                RowLayout {
                                    Layout.fillWidth: true

                                    Label {
                                        text: "Dictionary"
                                        color: theme.textPrimary
                                        font.pixelSize: 15
                                    }

                                    Label {
                                        text: dictionary_size
                                        color: theme.textSecondary
                                        font.pixelSize: 14
                                    }

                                    Item {
                                        Layout.fillWidth: true
                                    }

                                    ToolButton {
                                        visible: dictionary_progress <= 0
                                        display: AbstractButton.IconOnly
                                        icon.source: actionIcon(dictionary_installed)
                                        icon.width: 18
                                        icon.height: 18
                                        onClicked: featureAction(code, 1, dictionary_installed)
                                    }
                                }

                                ProgressBar {
                                    visible: dictionary_progress > 0
                                    Layout.fillWidth: true
                                    value: dictionary_progress
                                    from: 0
                                    to: 1
                                }
                            }
                        }

                        Item {
                            visible: tts_available
                            Layout.fillWidth: true
                            implicitHeight: tts_progress > 0 ? 44 : 28

                            ColumnLayout {
                                anchors.fill: parent
                                spacing: 4

                                RowLayout {
                                    Layout.fillWidth: true

                                    Label {
                                        text: "Text-to-speech"
                                        color: theme.textPrimary
                                        font.pixelSize: 15
                                    }

                                    Label {
                                        text: tts_size
                                        color: theme.textSecondary
                                        font.pixelSize: 14
                                    }

                                    Item {
                                        Layout.fillWidth: true
                                    }

                                    ToolButton {
                                        visible: tts_progress <= 0
                                        display: AbstractButton.IconOnly
                                        icon.source: actionIcon(tts_installed)
                                        icon.width: 18
                                        icon.height: 18
                                        onClicked: featureAction(code, 2, tts_installed)
                                    }
                                }

                                ProgressBar {
                                    visible: tts_progress > 0
                                    Layout.fillWidth: true
                                    value: tts_progress
                                    from: 0
                                    to: 1
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
