import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15

Item {
    id: root
    property var appBridge
    property var theme

    property bool advancedExpanded: false
    property string expandMoreIcon: appBridge.asset_url("expand_more.svg")

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
                Layout.leftMargin: 16; Layout.rightMargin: 16

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

            // ── Languages ──
            Rectangle {
                Layout.fillWidth: true
                Layout.leftMargin: 16; Layout.rightMargin: 16
                implicitHeight: langCol.implicitHeight + 32
                radius: 12; color: theme.surfaceColor

                ColumnLayout {
                    id: langCol
                    anchors { left: parent.left; right: parent.right; top: parent.top; margins: 16 }
                    spacing: 12

                    Label { text: "Languages"; color: theme.accentColor; font.pixelSize: 18; font.bold: true }

                    Item {
                        Layout.fillWidth: true; implicitHeight: 28

                        Label {
                            anchors.left: parent.left; anchors.verticalCenter: parent.verticalCenter
                            text: "Language Packs"; color: theme.textPrimary; font.pixelSize: 15
                        }
                        Label {
                            anchors.right: parent.right; anchors.verticalCenter: parent.verticalCenter
                            text: "Manage"; color: theme.accentColor; font.pixelSize: 15
                            MouseArea { anchors.fill: parent; cursorShape: Qt.PointingHandCursor; onClicked: appBridge.show_manage_languages() }
                        }
                    }
                }
            }

            Item { Layout.preferredHeight: 16 }

            // ── General ──
            Rectangle {
                Layout.fillWidth: true
                Layout.leftMargin: 16; Layout.rightMargin: 16
                implicitHeight: generalCol.implicitHeight + 32
                radius: 12; color: theme.surfaceColor

                ColumnLayout {
                    id: generalCol
                    anchors { left: parent.left; right: parent.right; top: parent.top; margins: 16 }
                    spacing: 16

                    Label { text: "General"; color: theme.accentColor; font.pixelSize: 18; font.bold: true }

                    ColumnLayout {
                        Layout.fillWidth: true; spacing: 6
                        Label { text: "Default 'from' language"; color: theme.textSecondary; font.pixelSize: 13 }
                        DarkComboBox {
                            Layout.fillWidth: true; Layout.preferredHeight: 40
                            theme: root.theme; iconSource: expandMoreIcon
                            model: appBridge.installed_from_language_names
                            Component.onCompleted: { var idx = find(appBridge.source_language_name); if (idx >= 0) currentIndex = idx }
                            onActivated: appBridge.set_from(currentText)
                        }
                    }

                    ColumnLayout {
                        Layout.fillWidth: true; spacing: 6
                        Label { text: "Default 'to' language"; color: theme.textSecondary; font.pixelSize: 13 }
                        DarkComboBox {
                            Layout.fillWidth: true; Layout.preferredHeight: 40
                            theme: root.theme; iconSource: expandMoreIcon
                            model: appBridge.installed_to_language_names
                            Component.onCompleted: { var idx = find(appBridge.target_language_name); if (idx >= 0) currentIndex = idx }
                            onActivated: appBridge.set_to(currentText)
                        }
                    }

                }
            }

            Item { Layout.preferredHeight: 16 }

            // ── OCR ──
            Rectangle {
                Layout.fillWidth: true
                Layout.leftMargin: 16; Layout.rightMargin: 16
                implicitHeight: ocrCol.implicitHeight + 32
                radius: 12; color: theme.surfaceColor

                ColumnLayout {
                    id: ocrCol
                    anchors { left: parent.left; right: parent.right; top: parent.top; margins: 16 }
                    spacing: 16

                    Label { text: "OCR"; color: theme.accentColor; font.pixelSize: 18; font.bold: true }

                    ColumnLayout {
                        Layout.fillWidth: true; spacing: 6
                        Label { text: "Background Mode"; color: theme.textSecondary; font.pixelSize: 13 }
                        DarkComboBox {
                            Layout.fillWidth: true; Layout.preferredHeight: 40
                            theme: root.theme; iconSource: expandMoreIcon
                            model: ["Auto-detect Colors", "Light Background", "Dark Background"]
                            Component.onCompleted: { var idx = find(appBridge.ocr_background_mode); if (idx >= 0) currentIndex = idx }
                            onActivated: appBridge.set_ocr_background_mode_value(currentText)
                        }
                    }

                    ColumnLayout {
                        Layout.fillWidth: true; spacing: 6
                        Label { text: "Min Confidence: " + appBridge.ocr_min_confidence + "%"; color: theme.textSecondary; font.pixelSize: 13 }
                        DarkSlider {
                            Layout.fillWidth: true; Layout.preferredHeight: 28
                            theme: root.theme
                            from: 0; to: 100; stepSize: 5
                            value: appBridge.ocr_min_confidence
                            onMoved: appBridge.set_ocr_min_confidence_value(value)
                        }
                    }

                    ColumnLayout {
                        Layout.fillWidth: true; spacing: 6
                        Label { text: "Max Image Size: " + appBridge.ocr_max_image_size + "px"; color: theme.textSecondary; font.pixelSize: 13 }
                        DarkSlider {
                            Layout.fillWidth: true; Layout.preferredHeight: 28
                            theme: root.theme
                            from: 1500; to: 4000; stepSize: 100
                            value: appBridge.ocr_max_image_size
                            onMoved: appBridge.set_ocr_max_image_size_value(value)
                        }
                    }
                }
            }

            Item { Layout.preferredHeight: 16 }

            // ── Advanced Settings ──
            Rectangle {
                Layout.fillWidth: true
                Layout.leftMargin: 16; Layout.rightMargin: 16
                implicitHeight: advCol.implicitHeight + 32
                radius: 12; color: theme.surfaceColor

                ColumnLayout {
                    id: advCol
                    anchors { left: parent.left; right: parent.right; top: parent.top; margins: 16 }
                    spacing: 16

                    Item {
                        Layout.fillWidth: true; implicitHeight: 28

                        Label {
                            anchors.left: parent.left; anchors.verticalCenter: parent.verticalCenter
                            text: "Advanced Settings"; color: theme.accentColor; font.pixelSize: 18; font.bold: true
                        }
                        Image {
                            anchors.right: parent.right; anchors.verticalCenter: parent.verticalCenter
                            width: 20; height: 20
                            source: advancedExpanded ? appBridge.asset_url("expand_less.svg") : expandMoreIcon
                            sourceSize.width: 20; sourceSize.height: 20
                        }
                        MouseArea { anchors.fill: parent; onClicked: advancedExpanded = !advancedExpanded }
                    }

                    ColumnLayout {
                        visible: advancedExpanded
                        Layout.fillWidth: true
                        spacing: 16

                        ColumnLayout {
                            Layout.fillWidth: true; spacing: 6
                            Label { text: "Catalog Index URL"; color: theme.textSecondary; font.pixelSize: 13 }
                            TextField {
                                Layout.fillWidth: true
                                text: appBridge.catalog_index_url
                                color: theme.textPrimary
                                placeholderTextColor: theme.textSecondary
                                onEditingFinished: appBridge.set_catalog_index_url_value(text)
                                background: Rectangle { radius: 8; color: theme.backgroundElevated; border.width: 1; border.color: theme.borderColor }
                            }
                        }

                        DarkSwitch {
                            Layout.fillWidth: true; theme: root.theme
                            label: "Disable OCR"
                            checked: appBridge.disable_ocr
                            onToggled: appBridge.set_disable_ocr_value(checked)
                        }

                        DarkSwitch {
                            Layout.fillWidth: true; theme: root.theme
                            label: "Disable automatic language detection"
                            checked: appBridge.disable_auto_detect
                            onToggled: appBridge.set_disable_auto_detect_value(checked)
                        }

                        DarkSwitch {
                            Layout.fillWidth: true; theme: root.theme
                            label: "Show transliteration for output"
                            checked: appBridge.show_transliteration_output
                            onToggled: appBridge.set_show_transliteration_output_value(checked)
                        }

                        DarkSwitch {
                            Layout.fillWidth: true; theme: root.theme
                            label: "Show transliteration for input"
                            checked: appBridge.show_transliteration_input
                            onToggled: appBridge.set_show_transliteration_input_value(checked)
                        }
                    }
                }
            }

            Item { Layout.preferredHeight: 32 }
        }
    }
}
