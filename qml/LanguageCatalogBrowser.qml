import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15

Item {
    property var appBridge
    property var theme

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

        ListView {
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            spacing: 0
            model: appBridge.manage_languages_model

            delegate: Item {
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

                readonly property int installedCount:
                    (core_available && core_installed ? 1 : 0) +
                    (dictionary_available && dictionary_installed ? 1 : 0) +
                    (tts_available && tts_installed ? 1 : 0)
                readonly property int availableCount:
                    (core_available ? 1 : 0) +
                    (dictionary_available ? 1 : 0) +
                    (tts_available ? 1 : 0)
                readonly property bool allInstalled: availableCount > 0 && installedCount === availableCount
                readonly property bool noneInstalled: installedCount === 0
                readonly property bool someInstalled: !allInstalled && !noneInstalled

                width: ListView.view.width
                height: delegateLayout.implicitHeight

                ColumnLayout {
                    id: delegateLayout
                    width: parent.width
                    spacing: 0

                    Item {
                        Layout.fillWidth: true
                        implicitHeight: 52

                        MouseArea {
                            anchors.fill: parent
                            onClicked: toggleLanguage(code)
                        }

                        ToolButton {
                            id: chevronBtn
                            anchors.left: parent.left
                            anchors.leftMargin: 4
                            anchors.verticalCenter: parent.verticalCenter
                            z: 1
                            display: AbstractButton.IconOnly
                            icon.source: expanded ? appBridge.asset_url("expand_less.svg") : appBridge.asset_url("expand_more.svg")
                            icon.width: 16; icon.height: 16
                            icon.color: theme.textSecondary
                            background: Item {}
                            onClicked: toggleLanguage(code)
                        }

                        Column {
                            anchors.left: chevronBtn.right
                            anchors.right: actionArea.left
                            anchors.verticalCenter: parent.verticalCenter
                            anchors.rightMargin: 8
                            spacing: 1

                            Label {
                                text: name
                                width: parent.width
                                color: theme.textPrimary
                                font.pixelSize: 16
                                font.bold: true
                                elide: Text.ElideRight
                            }

                            Label {
                                text: total_size
                                color: theme.textSecondary
                                font.pixelSize: 12
                            }
                        }

                        Row {
                            id: actionArea
                            anchors.right: parent.right
                            anchors.rightMargin: 12
                            anchors.verticalCenter: parent.verticalCenter
                            spacing: 4

                            Row {
                                visible: expanded || someInstalled
                                spacing: 2
                                anchors.verticalCenter: parent.verticalCenter

                                Image {
                                    width: 20; height: 20
                                    source: appBridge.asset_url("translate.svg")
                                    sourceSize.width: 20; sourceSize.height: 20
                                    opacity: core_available ? (core_installed ? 1.0 : 0.3) : 0
                                }

                                Image {
                                    width: 20; height: 20
                                    source: appBridge.asset_url("dictionary.svg")
                                    sourceSize.width: 20; sourceSize.height: 20
                                    opacity: dictionary_available ? (dictionary_installed ? 1.0 : 0.3) : 0
                                }

                                Image {
                                    width: 20; height: 20
                                    source: appBridge.asset_url("tts.svg")
                                    sourceSize.width: 20; sourceSize.height: 20
                                    opacity: tts_available ? (tts_installed ? 1.0 : 0.3) : 0
                                }
                            }

                            Item {
                                visible: !expanded && noneInstalled
                                width: 24; height: 24
                                anchors.verticalCenter: parent.verticalCenter

                                Image {
                                    anchors.centerIn: parent
                                    width: 20; height: 20
                                    source: appBridge.asset_url("download.svg")
                                    sourceSize.width: 20; sourceSize.height: 20
                                }

                                MouseArea {
                                    anchors.fill: parent
                                    z: 1
                                    onClicked: featureAction(code, 0, false)
                                }
                            }

                            Item {
                                visible: !expanded && allInstalled
                                width: 24; height: 24
                                anchors.verticalCenter: parent.verticalCenter

                                Image {
                                    anchors.centerIn: parent
                                    width: 20; height: 20
                                    source: appBridge.asset_url("delete.svg")
                                    sourceSize.width: 20; sourceSize.height: 20
                                }

                                MouseArea {
                                    anchors.fill: parent
                                    z: 1
                                    onClicked: featureAction(code, 0, true)
                                }
                            }
                        }
                    }

                    Rectangle {
                        Layout.fillWidth: true
                        height: 1
                        color: "#2A2D3A"
                    }

                    ColumnLayout {
                        visible: expanded
                        Layout.fillWidth: true
                        Layout.leftMargin: 40
                        Layout.rightMargin: 8
                        Layout.bottomMargin: 8
                        spacing: 2

                        Item {
                            visible: core_available
                            Layout.fillWidth: true
                            implicitHeight: coreProgress.visible ? 40 : 28

                            ColumnLayout {
                                anchors.fill: parent
                                spacing: 2

                                Item {
                                    Layout.fillWidth: true
                                    implicitHeight: 24

                                    Label {
                                        anchors.left: parent.left
                                        anchors.verticalCenter: parent.verticalCenter
                                        text: "Translation"
                                        color: theme.textPrimary
                                        font.pixelSize: 14
                                    }

                                    Label {
                                        anchors.left: parent.left
                                        anchors.leftMargin: 90
                                        anchors.verticalCenter: parent.verticalCenter
                                        text: core_size
                                        color: theme.textSecondary
                                        font.pixelSize: 12
                                    }

                                    Item {
                                        anchors.right: parent.right
                                        anchors.verticalCenter: parent.verticalCenter
                                        width: 24; height: 24
                                        opacity: isBusy(core_progress) ? 0.3 : 1.0

                                        Image {
                                            anchors.centerIn: parent
                                            width: 18; height: 18
                                            source: actionIcon(core_installed)
                                            sourceSize.width: 18; sourceSize.height: 18
                                        }

                                        MouseArea {
                                            anchors.fill: parent
                                            enabled: !isBusy(core_progress)
                                            onClicked: featureAction(code, 0, core_installed)
                                        }
                                    }
                                }

                                ProgressBar {
                                    id: coreProgress
                                    visible: isBusy(core_progress)
                                    Layout.fillWidth: true
                                    from: 0; to: 1; value: core_progress

                                    background: Rectangle { implicitHeight: 3; radius: 2; color: "#303240" }
                                    contentItem: Item {
                                        Rectangle { width: coreProgress.visualPosition * parent.width; height: parent.height; radius: 2; color: theme.accentColor }
                                    }
                                }
                            }
                        }

                        Item {
                            visible: dictionary_available
                            Layout.fillWidth: true
                            implicitHeight: dictionaryProgress.visible ? 40 : 28

                            ColumnLayout {
                                anchors.fill: parent
                                spacing: 2

                                Item {
                                    Layout.fillWidth: true
                                    implicitHeight: 24

                                    Label {
                                        anchors.left: parent.left
                                        anchors.verticalCenter: parent.verticalCenter
                                        text: "Dictionary"
                                        color: theme.textPrimary
                                        font.pixelSize: 14
                                    }

                                    Label {
                                        anchors.left: parent.left
                                        anchors.leftMargin: 90
                                        anchors.verticalCenter: parent.verticalCenter
                                        text: dictionary_size
                                        color: theme.textSecondary
                                        font.pixelSize: 12
                                    }

                                    Item {
                                        anchors.right: parent.right
                                        anchors.verticalCenter: parent.verticalCenter
                                        width: 24; height: 24
                                        opacity: isBusy(dictionary_progress) ? 0.3 : 1.0

                                        Image {
                                            anchors.centerIn: parent
                                            width: 18; height: 18
                                            source: actionIcon(dictionary_installed)
                                            sourceSize.width: 18; sourceSize.height: 18
                                        }

                                        MouseArea {
                                            anchors.fill: parent
                                            enabled: !isBusy(dictionary_progress)
                                            onClicked: featureAction(code, 1, dictionary_installed)
                                        }
                                    }
                                }

                                ProgressBar {
                                    id: dictionaryProgress
                                    visible: isBusy(dictionary_progress)
                                    Layout.fillWidth: true
                                    from: 0; to: 1; value: dictionary_progress

                                    background: Rectangle { implicitHeight: 3; radius: 2; color: "#303240" }
                                    contentItem: Item {
                                        Rectangle { width: dictionaryProgress.visualPosition * parent.width; height: parent.height; radius: 2; color: theme.accentColor }
                                    }
                                }
                            }
                        }

                        Item {
                            visible: tts_available
                            Layout.fillWidth: true
                            implicitHeight: ttsProgress.visible ? 40 : 28

                            ColumnLayout {
                                anchors.fill: parent
                                spacing: 2

                                Item {
                                    Layout.fillWidth: true
                                    implicitHeight: 24

                                    Label {
                                        anchors.left: parent.left
                                        anchors.verticalCenter: parent.verticalCenter
                                        text: "Text-to-speech"
                                        color: theme.textPrimary
                                        font.pixelSize: 14
                                    }

                                    Label {
                                        anchors.left: parent.left
                                        anchors.leftMargin: 115
                                        anchors.verticalCenter: parent.verticalCenter
                                        text: tts_size
                                        color: theme.textSecondary
                                        font.pixelSize: 12
                                    }

                                    Item {
                                        anchors.right: parent.right
                                        anchors.verticalCenter: parent.verticalCenter
                                        width: 24; height: 24
                                        opacity: isBusy(tts_progress) ? 0.3 : 1.0

                                        Image {
                                            anchors.centerIn: parent
                                            width: 18; height: 18
                                            source: actionIcon(tts_installed)
                                            sourceSize.width: 18; sourceSize.height: 18
                                        }

                                        MouseArea {
                                            anchors.fill: parent
                                            enabled: !isBusy(tts_progress)
                                            onClicked: featureAction(code, 2, tts_installed)
                                        }
                                    }
                                }

                                ProgressBar {
                                    id: ttsProgress
                                    visible: isBusy(tts_progress)
                                    Layout.fillWidth: true
                                    from: 0; to: 1; value: tts_progress

                                    background: Rectangle { implicitHeight: 3; radius: 2; color: "#303240" }
                                    contentItem: Item {
                                        Rectangle { width: ttsProgress.visualPosition * parent.width; height: parent.height; radius: 2; color: theme.accentColor }
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
