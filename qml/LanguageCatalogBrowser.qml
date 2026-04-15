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

    function handleTtsFeatureAction(code, installed) {
        if (installed) {
            appBridge.delete_feature(code, 2)
        } else {
            appBridge.open_tts_download_picker(code)
        }
    }

    function toggleLanguage(code) {
        appBridge.toggle_manage_language(code)
    }

    function isBusy(progress) {
        return progress > 0 && progress < 1
    }

    Connections {
        target: appBridge

        function onManage_tts_picker_openChanged() {
            if (appBridge.manage_tts_picker_open) {
                ttsPickerPopup.open()
            } else {
                ttsPickerPopup.close()
            }
        }
    }

    Popup {
        id: ttsPickerPopup
        parent: Overlay.overlay
        modal: true
        focus: true
        dim: true
        closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutside
        x: Math.round((parent.width - width) / 2)
        y: Math.round((parent.height - height) / 2)
        width: Math.min(parent.width - 24, 420)
        height: Math.min(parent.height - 80, 520)
        padding: 0

        background: Rectangle {
            radius: 28
            color: "#2B2D36"
        }

        onClosed: {
            if (appBridge.manage_tts_picker_open) {
                appBridge.close_tts_download_picker()
            }
        }

        contentItem: ColumnLayout {
            spacing: 0

            Label {
                Layout.fillWidth: true
                Layout.topMargin: 18
                Layout.leftMargin: 20
                Layout.rightMargin: 20
                text: "Pick a voice"
                color: "white"
                font.pixelSize: 22
                font.bold: true
            }

            ListView {
                Layout.fillWidth: true
                Layout.fillHeight: true
                Layout.topMargin: 14
                Layout.leftMargin: 14
                Layout.rightMargin: 14
                clip: true
                spacing: 2
                model: appBridge.manage_tts_picker_model
                section.property: "region_display_name"
                section.criteria: ViewSection.FullString

                section.delegate: Label {
                    width: ListView.view.width
                    text: section
                    color: "#E4E6F2"
                    font.pixelSize: 14
                    font.bold: true
                    leftPadding: 4
                    topPadding: 8
                    bottomPadding: 4
                }

                delegate: Item {
                    required property string pack_id
                    required property string voice_display_name
                    required property string quality_text
                    required property string size_text
                    required property bool installed

                    width: ListView.view.width
                    height: 46

                    Column {
                        anchors.left: parent.left
                        anchors.leftMargin: 12
                        anchors.right: actionItem.left
                        anchors.rightMargin: 10
                        anchors.verticalCenter: parent.verticalCenter
                        spacing: 1

                        Label {
                            width: parent.width
                            text: voice_display_name
                            color: installed ? "#8A8E9F" : "#F1F3FB"
                            font.pixelSize: 16
                            elide: Text.ElideRight
                        }

                        Label {
                            width: parent.width
                            text: size_text
                            color: "#A9ADBC"
                            font.pixelSize: 13
                            elide: Text.ElideRight
                        }
                    }

                    Item {
                        id: actionItem
                        anchors.right: parent.right
                        anchors.rightMargin: 10
                        anchors.verticalCenter: parent.verticalCenter
                        width: installed ? 62 : 28
                        height: 28

                        Label {
                            anchors.centerIn: parent
                            visible: installed
                            text: "Installed"
                            color: "#8A8E9F"
                            font.pixelSize: 12
                        }

                        Image {
                            anchors.centerIn: parent
                            visible: !installed
                            width: 18
                            height: 18
                            source: appBridge.asset_url("download.svg")
                            sourceSize.width: 18
                            sourceSize.height: 18
                        }

                        MouseArea {
                            anchors.fill: parent
                            enabled: !installed
                            onClicked: appBridge.download_tts_pack(pack_id)
                        }
                    }
                }
            }

            Item {
                Layout.fillWidth: true
                Layout.preferredHeight: 64

                Button {
                    anchors.right: parent.right
                    anchors.rightMargin: 18
                    anchors.verticalCenter: parent.verticalCenter
                    text: "Cancel"
                    flat: true
                    onClicked: appBridge.close_tts_download_picker()

                    contentItem: Text {
                        text: parent.text
                        color: theme.accentColor
                        font.pixelSize: 16
                        horizontalAlignment: Text.AlignHCenter
                        verticalAlignment: Text.AlignVCenter
                    }

                    background: Item {}
                }
            }
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
                    (core_available && !built_in && core_installed ? 1 : 0) +
                    (dictionary_available && dictionary_installed ? 1 : 0) +
                    (tts_available && tts_installed ? 1 : 0)
                readonly property int availableCount:
                    (core_available && !built_in ? 1 : 0) +
                    (dictionary_available ? 1 : 0) +
                    (tts_available ? 1 : 0)
                readonly property bool allInstalled: availableCount > 0 && installedCount === availableCount
                readonly property bool noneInstalled: installedCount === 0
                readonly property bool someInstalled: !allInstalled && !noneInstalled
                readonly property bool anyBusy:
                    isBusy(core_progress) || isBusy(dictionary_progress) || isBusy(tts_progress)

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

                            // T D S icons: when expanded (always) or collapsed with partial install, and not busy
                            Row {
                                visible: (expanded || someInstalled) && !anyBusy
                                spacing: 2
                                anchors.verticalCenter: parent.verticalCenter

                                Image {
                                    width: 20; height: 20
                                    source: appBridge.asset_url("translate.svg")
                                    sourceSize.width: 20; sourceSize.height: 20
                                    opacity: (core_available && !built_in) ? (core_installed ? 1.0 : 0.3) : 0
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

                            // Download button: collapsed + nothing installed + not busy
                            Item {
                                visible: !expanded && noneInstalled && !anyBusy
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
                                    onClicked: appBridge.download_all_features(code)
                                }
                            }

                            // Delete button: collapsed + everything installed + not busy
                            Item {
                                visible: !expanded && allInstalled && !anyBusy
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
                                    onClicked: appBridge.delete_all_features(code)
                                }
                            }

                            // Indeterminate spinner: collapsed + any download active
                            CircularProgress {
                                visible: !expanded && anyBusy
                                indeterminate: true
                                progressColor: theme.accentColor
                                anchors.verticalCenter: parent.verticalCenter
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

                        // Translation feature (hidden for built-in languages like English)
                        Item {
                            visible: core_available && !built_in
                            Layout.fillWidth: true
                            implicitHeight: 28

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

                            // Circular progress when downloading
                            CircularProgress {
                                visible: isBusy(core_progress)
                                anchors.right: parent.right
                                anchors.verticalCenter: parent.verticalCenter
                                progress: core_progress
                                progressColor: theme.accentColor
                            }

                            // Action icon when not downloading
                            Item {
                                visible: !isBusy(core_progress)
                                anchors.right: parent.right
                                anchors.verticalCenter: parent.verticalCenter
                                width: 24; height: 24

                                Image {
                                    anchors.centerIn: parent
                                    width: 18; height: 18
                                    source: actionIcon(core_installed)
                                    sourceSize.width: 18; sourceSize.height: 18
                                }

                                MouseArea {
                                    anchors.fill: parent
                                    onClicked: featureAction(code, 0, core_installed)
                                }
                            }
                        }

                        // Dictionary feature
                        Item {
                            visible: dictionary_available
                            Layout.fillWidth: true
                            implicitHeight: 28

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

                            CircularProgress {
                                visible: isBusy(dictionary_progress)
                                anchors.right: parent.right
                                anchors.verticalCenter: parent.verticalCenter
                                progress: dictionary_progress
                                progressColor: theme.accentColor
                            }

                            Item {
                                visible: !isBusy(dictionary_progress)
                                anchors.right: parent.right
                                anchors.verticalCenter: parent.verticalCenter
                                width: 24; height: 24

                                Image {
                                    anchors.centerIn: parent
                                    width: 18; height: 18
                                    source: actionIcon(dictionary_installed)
                                    sourceSize.width: 18; sourceSize.height: 18
                                }

                                MouseArea {
                                    anchors.fill: parent
                                    onClicked: featureAction(code, 1, dictionary_installed)
                                }
                            }
                        }

                        // TTS feature
                        Item {
                            visible: tts_available
                            Layout.fillWidth: true
                            implicitHeight: 28

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

                            CircularProgress {
                                visible: isBusy(tts_progress)
                                anchors.right: parent.right
                                anchors.verticalCenter: parent.verticalCenter
                                progress: tts_progress
                                progressColor: theme.accentColor
                            }

                            Item {
                                visible: !isBusy(tts_progress)
                                anchors.right: parent.right
                                anchors.verticalCenter: parent.verticalCenter
                                width: 24; height: 24

                                Image {
                                    anchors.centerIn: parent
                                    width: 18; height: 18
                                    source: actionIcon(tts_installed)
                                    sourceSize.width: 18; sourceSize.height: 18
                                }

                                MouseArea {
                                    anchors.fill: parent
                                    onClicked: handleTtsFeatureAction(code, tts_installed)
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
