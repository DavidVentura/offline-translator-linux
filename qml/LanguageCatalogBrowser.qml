import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15

Item {
    property var appBridge
    property var theme

    function actionIcon(installed) {
        return installed ? appBridge.asset_url("delete.svg") : appBridge.asset_url("download.svg")
    }

    function actionColor(installed) {
        return installed ? "#F28B82" : theme.textSecondary
    }

    function featureAction(code, feature, installed) {
        if (installed) {
            appBridge.delete_feature(code, feature)
        } else {
            appBridge.download_feature(code, feature)
        }
    }

    function statusColor(installed) {
        return installed ? theme.textPrimary : theme.textSecondary
    }

    function toggleLanguage(code) {
        appBridge.toggle_manage_language(code)
    }

    function isBusy(progress) {
        return progress > 0 && progress < 1
    }

    ColumnLayout {
        anchors.fill: parent
        spacing: 10

        TextField {
            Layout.fillWidth: true
            placeholderText: "Filter languages"
            text: appBridge.manage_filter_text
            color: theme.textPrimary
            placeholderTextColor: theme.textSecondary
            onTextChanged: appBridge.set_manage_filter(text)

            background: Rectangle {
                radius: 4
                color: "#181922"
                border.width: 1
                border.color: "#343646"
            }
        }

        ScrollView {
            id: browserScroll
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            contentWidth: availableWidth

            background: Rectangle {
                color: "#14151C"
            }

            ScrollBar.vertical.policy: ScrollBar.AsNeeded
            ScrollBar.horizontal.policy: ScrollBar.AlwaysOff

            Column {
                width: browserScroll.availableWidth
                spacing: 0

                Repeater {
                    model: appBridge.manage_languages_model

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

                        width: parent.width
                        height: contentColumn.implicitHeight + 12
                        color: index % 2 === 0 ? "#1A1B24" : "#16171F"
                        border.width: 0

                        Rectangle {
                            anchors.left: parent.left
                            anchors.right: parent.right
                            anchors.bottom: parent.bottom
                            height: 1
                            color: "#2A2D3A"
                        }

                        ColumnLayout {
                            id: contentColumn
                            anchors.left: parent.left
                            anchors.right: parent.right
                            anchors.top: parent.top
                            anchors.leftMargin: 8
                            anchors.rightMargin: 10
                            anchors.topMargin: 6
                            spacing: 8

                            Item {
                                Layout.fillWidth: true
                                implicitHeight: 46

                                MouseArea {
                                    anchors.fill: parent
                                    onClicked: toggleLanguage(code)
                                }

                                RowLayout {
                                    anchors.fill: parent
                                    spacing: 8

                                    ToolButton {
                                        Layout.alignment: Qt.AlignTop
                                        z: 1
                                        visible: core_available || dictionary_available || tts_available
                                        display: AbstractButton.IconOnly
                                        icon.source: expanded ? appBridge.asset_url("expand_less.svg") : appBridge.asset_url("expand_more.svg")
                                        icon.width: 18
                                        icon.height: 18
                                        icon.color: theme.textSecondary
                                        background: Item {}
                                        onClicked: toggleLanguage(code)
                                    }

                                    ColumnLayout {
                                        Layout.fillWidth: true
                                        spacing: 1

                                        Label {
                                            text: name
                                            color: theme.textPrimary
                                            font.pixelSize: 17
                                            font.bold: true
                                            elide: Text.ElideRight
                                        }

                                        Label {
                                            text: total_size
                                            color: theme.textSecondary
                                            font.pixelSize: 13
                                        }
                                    }

                                    RowLayout {
                                        Layout.alignment: Qt.AlignVCenter
                                        spacing: 6

                                        Label {
                                            visible: core_available
                                            text: "T"
                                            color: statusColor(core_installed)
                                            font.pixelSize: 12
                                            font.bold: true
                                            opacity: core_installed ? 1.0 : 0.7
                                        }

                                        Label {
                                            visible: dictionary_available
                                            text: "D"
                                            color: statusColor(dictionary_installed)
                                            font.pixelSize: 12
                                            font.bold: true
                                            opacity: dictionary_installed ? 1.0 : 0.7
                                        }

                                        Label {
                                            visible: tts_available
                                            text: "S"
                                            color: statusColor(tts_installed)
                                            font.pixelSize: 12
                                            font.bold: true
                                            opacity: tts_installed ? 1.0 : 0.7
                                        }
                                    }

                                    ToolButton {
                                        visible: core_available
                                        enabled: !isBusy(core_progress)
                                        z: 1
                                        display: AbstractButton.IconOnly
                                        icon.source: actionIcon(core_installed)
                                        icon.width: 18
                                        icon.height: 18
                                        icon.color: actionColor(core_installed)
                                        background: Item {}
                                        onClicked: featureAction(code, 0, core_installed)
                                    }
                                }
                            }

                            ColumnLayout {
                                visible: expanded
                                Layout.fillWidth: true
                                spacing: 2

                                Rectangle {
                                    Layout.fillWidth: true
                                    height: 1
                                    color: "#2A2D3A"
                                }

                                Item {
                                    visible: core_available
                                    Layout.fillWidth: true
                                    implicitHeight: coreProgress.visible ? 44 : 32

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
                                                font.pixelSize: 13
                                            }

                                            Item {
                                                Layout.fillWidth: true
                                            }

                                            ToolButton {
                                                enabled: !isBusy(core_progress)
                                                display: AbstractButton.IconOnly
                                                icon.source: actionIcon(core_installed)
                                                icon.width: 18
                                                icon.height: 18
                                                icon.color: actionColor(core_installed)
                                                background: Item {}
                                                onClicked: featureAction(code, 0, core_installed)
                                            }
                                        }

                                        ProgressBar {
                                            id: coreProgress
                                            visible: isBusy(core_progress)
                                            Layout.fillWidth: true
                                            from: 0
                                            to: 1
                                            value: core_progress

                                            background: Rectangle {
                                                implicitHeight: 4
                                                radius: 2
                                                color: "#303240"
                                            }

                                            contentItem: Item {
                                                Rectangle {
                                                    width: coreProgress.visualPosition * parent.width
                                                    height: parent.height
                                                    radius: 2
                                                    color: theme.accentColor
                                                }
                                            }
                                        }
                                    }
                                }

                                Item {
                                    visible: dictionary_available
                                    Layout.fillWidth: true
                                    implicitHeight: dictionaryProgress.visible ? 44 : 32

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
                                                font.pixelSize: 13
                                            }

                                            Item {
                                                Layout.fillWidth: true
                                            }

                                            ToolButton {
                                                enabled: !isBusy(dictionary_progress)
                                                display: AbstractButton.IconOnly
                                                icon.source: actionIcon(dictionary_installed)
                                                icon.width: 18
                                                icon.height: 18
                                                icon.color: actionColor(dictionary_installed)
                                                background: Item {}
                                                onClicked: featureAction(code, 1, dictionary_installed)
                                            }
                                        }

                                        ProgressBar {
                                            id: dictionaryProgress
                                            visible: isBusy(dictionary_progress)
                                            Layout.fillWidth: true
                                            from: 0
                                            to: 1
                                            value: dictionary_progress

                                            background: Rectangle {
                                                implicitHeight: 4
                                                radius: 2
                                                color: "#303240"
                                            }

                                            contentItem: Item {
                                                Rectangle {
                                                    width: dictionaryProgress.visualPosition * parent.width
                                                    height: parent.height
                                                    radius: 2
                                                    color: theme.accentColor
                                                }
                                            }
                                        }
                                    }
                                }

                                Item {
                                    visible: tts_available
                                    Layout.fillWidth: true
                                    implicitHeight: ttsProgress.visible ? 44 : 32

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
                                                font.pixelSize: 13
                                            }

                                            Item {
                                                Layout.fillWidth: true
                                            }

                                            ToolButton {
                                                enabled: !isBusy(tts_progress)
                                                display: AbstractButton.IconOnly
                                                icon.source: actionIcon(tts_installed)
                                                icon.width: 18
                                                icon.height: 18
                                                icon.color: actionColor(tts_installed)
                                                background: Item {}
                                                onClicked: featureAction(code, 2, tts_installed)
                                            }
                                        }

                                        ProgressBar {
                                            id: ttsProgress
                                            visible: isBusy(tts_progress)
                                            Layout.fillWidth: true
                                            from: 0
                                            to: 1
                                            value: tts_progress

                                            background: Rectangle {
                                                implicitHeight: 4
                                                radius: 2
                                                color: "#303240"
                                            }

                                            contentItem: Item {
                                                Rectangle {
                                                    width: ttsProgress.visualPosition * parent.width
                                                    height: parent.height
                                                    radius: 2
                                                    color: theme.accentColor
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
