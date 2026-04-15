import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15

Item {
    id: root
    property var appBridge
    property var theme

    property bool advancedExpanded: false

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

                    // Default 'from' language
                    ColumnLayout {
                        Layout.fillWidth: true; spacing: 6
                        Label { text: "Default 'from' language"; color: theme.textSecondary; font.pixelSize: 13 }
                        ComboBox {
                            id: fromSettingsCombo
                            Layout.fillWidth: true
                            Layout.preferredHeight: 40
                            model: appBridge.installed_from_language_names
                            Component.onCompleted: { var idx = find(appBridge.source_language_name); if (idx >= 0) currentIndex = idx }
                            onActivated: appBridge.set_from(currentText)

                            contentItem: Label {
                                leftPadding: 12
                                text: fromSettingsCombo.displayText
                                color: theme.textPrimary
                                verticalAlignment: Text.AlignVCenter
                                elide: Text.ElideRight
                            }
                            background: Rectangle { radius: 8; color: theme.backgroundElevated; border.width: 1; border.color: theme.borderColor }
                            indicator: Image { source: appBridge.asset_url("expand_more.svg"); width: 16; height: 16; x: fromSettingsCombo.width - width - 10; y: (fromSettingsCombo.height - height) / 2 }
                            delegate: ItemDelegate {
                                width: fromSettingsCombo.width
                                contentItem: Label { text: modelData; color: theme.textPrimary; verticalAlignment: Text.AlignVCenter }
                                background: Rectangle { color: highlighted ? theme.surfaceAltColor : theme.surfaceColor }
                                highlighted: fromSettingsCombo.highlightedIndex === index
                            }
                            popup: Popup {
                                y: fromSettingsCombo.height; width: fromSettingsCombo.width; implicitHeight: contentItem.implicitHeight; padding: 1
                                contentItem: ListView { clip: true; implicitHeight: contentHeight; model: parent.visible ? fromSettingsCombo.delegateModel : null }
                                background: Rectangle { color: theme.surfaceColor; border.color: theme.borderColor; radius: 4 }
                            }
                        }
                    }

                    // Default 'to' language
                    ColumnLayout {
                        Layout.fillWidth: true; spacing: 6
                        Label { text: "Default 'to' language"; color: theme.textSecondary; font.pixelSize: 13 }
                        ComboBox {
                            id: toSettingsCombo
                            Layout.fillWidth: true
                            Layout.preferredHeight: 40
                            model: appBridge.installed_to_language_names
                            Component.onCompleted: { var idx = find(appBridge.target_language_name); if (idx >= 0) currentIndex = idx }
                            onActivated: appBridge.set_to(currentText)

                            contentItem: Label {
                                leftPadding: 12
                                text: toSettingsCombo.displayText
                                color: theme.textPrimary
                                verticalAlignment: Text.AlignVCenter
                                elide: Text.ElideRight
                            }
                            background: Rectangle { radius: 8; color: theme.backgroundElevated; border.width: 1; border.color: theme.borderColor }
                            indicator: Image { source: appBridge.asset_url("expand_more.svg"); width: 16; height: 16; x: toSettingsCombo.width - width - 10; y: (toSettingsCombo.height - height) / 2 }
                            delegate: ItemDelegate {
                                width: toSettingsCombo.width
                                contentItem: Label { text: modelData; color: theme.textPrimary; verticalAlignment: Text.AlignVCenter }
                                background: Rectangle { color: highlighted ? theme.surfaceAltColor : theme.surfaceColor }
                                highlighted: toSettingsCombo.highlightedIndex === index
                            }
                            popup: Popup {
                                y: toSettingsCombo.height; width: toSettingsCombo.width; implicitHeight: contentItem.implicitHeight; padding: 1
                                contentItem: ListView { clip: true; implicitHeight: contentHeight; model: parent.visible ? toSettingsCombo.delegateModel : null }
                                background: Rectangle { color: theme.surfaceColor; border.color: theme.borderColor; radius: 4 }
                            }
                        }
                    }

                    // Font Size
                    ColumnLayout {
                        Layout.fillWidth: true; spacing: 6
                        Label { text: "Font Size"; color: theme.textSecondary; font.pixelSize: 13 }
                        Slider {
                            Layout.fillWidth: true
                            Layout.preferredHeight: 28
                            from: 12; to: 28; stepSize: 1
                            value: appBridge.font_size
                            onMoved: appBridge.set_font_size_value(value)

                            background: Rectangle {
                                x: parent.leftPadding; y: parent.topPadding + parent.availableHeight / 2 - height / 2
                                width: parent.availableWidth; height: 4; radius: 2; color: "#303240"
                                Rectangle { width: parent.parent.visualPosition * parent.width; height: parent.height; radius: 2; color: theme.accentColor }
                            }
                            handle: Rectangle {
                                x: parent.leftPadding + parent.visualPosition * (parent.availableWidth - width)
                                y: parent.topPadding + parent.availableHeight / 2 - height / 2
                                width: 18; height: 18; radius: 9
                                color: parent.pressed ? Qt.lighter(theme.accentColor, 1.2) : theme.accentColor
                            }
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

                    // Background Mode
                    ColumnLayout {
                        Layout.fillWidth: true; spacing: 6
                        Label { text: "Background Mode"; color: theme.textSecondary; font.pixelSize: 13 }
                        ComboBox {
                            id: ocrBgCombo
                            Layout.fillWidth: true
                            Layout.preferredHeight: 40
                            model: ["Auto-detect Colors", "Light Background", "Dark Background"]
                            Component.onCompleted: {
                                var idx = find(appBridge.ocr_background_mode)
                                if (idx >= 0) currentIndex = idx
                            }
                            onActivated: appBridge.set_ocr_background_mode_value(currentText)

                            contentItem: Label {
                                leftPadding: 12
                                text: ocrBgCombo.displayText
                                color: theme.textPrimary
                                verticalAlignment: Text.AlignVCenter
                                elide: Text.ElideRight
                            }
                            background: Rectangle { radius: 8; color: theme.backgroundElevated; border.width: 1; border.color: theme.borderColor }
                            indicator: Image { source: appBridge.asset_url("expand_more.svg"); width: 16; height: 16; x: ocrBgCombo.width - width - 10; y: (ocrBgCombo.height - height) / 2 }
                            delegate: ItemDelegate {
                                width: ocrBgCombo.width
                                contentItem: Label { text: modelData; color: theme.textPrimary; verticalAlignment: Text.AlignVCenter }
                                background: Rectangle { color: highlighted ? theme.surfaceAltColor : theme.surfaceColor }
                                highlighted: ocrBgCombo.highlightedIndex === index
                            }
                            popup: Popup {
                                y: ocrBgCombo.height; width: ocrBgCombo.width; implicitHeight: contentItem.implicitHeight; padding: 1
                                contentItem: ListView { clip: true; implicitHeight: contentHeight; model: parent.visible ? ocrBgCombo.delegateModel : null }
                                background: Rectangle { color: theme.surfaceColor; border.color: theme.borderColor; radius: 4 }
                            }
                        }
                    }

                    // Min Confidence
                    ColumnLayout {
                        Layout.fillWidth: true; spacing: 6
                        Label { text: "Min Confidence: " + appBridge.ocr_min_confidence + "%"; color: theme.textSecondary; font.pixelSize: 13 }
                        Slider {
                            Layout.fillWidth: true
                            Layout.preferredHeight: 28
                            from: 0; to: 100; stepSize: 5
                            value: appBridge.ocr_min_confidence
                            onMoved: appBridge.set_ocr_min_confidence_value(value)

                            background: Rectangle {
                                x: parent.leftPadding; y: parent.topPadding + parent.availableHeight / 2 - height / 2
                                width: parent.availableWidth; height: 4; radius: 2; color: "#303240"
                                Rectangle { width: parent.parent.visualPosition * parent.width; height: parent.height; radius: 2; color: theme.accentColor }
                            }
                            handle: Rectangle {
                                x: parent.leftPadding + parent.visualPosition * (parent.availableWidth - width)
                                y: parent.topPadding + parent.availableHeight / 2 - height / 2
                                width: 18; height: 18; radius: 9
                                color: parent.pressed ? Qt.lighter(theme.accentColor, 1.2) : theme.accentColor
                            }
                        }
                    }

                    // Max Image Size
                    ColumnLayout {
                        Layout.fillWidth: true; spacing: 6
                        Label { text: "Max Image Size: " + appBridge.ocr_max_image_size + "px"; color: theme.textSecondary; font.pixelSize: 13 }
                        Slider {
                            Layout.fillWidth: true
                            Layout.preferredHeight: 28
                            from: 1500; to: 4000; stepSize: 100
                            value: appBridge.ocr_max_image_size
                            onMoved: appBridge.set_ocr_max_image_size_value(value)

                            background: Rectangle {
                                x: parent.leftPadding; y: parent.topPadding + parent.availableHeight / 2 - height / 2
                                width: parent.availableWidth; height: 4; radius: 2; color: "#303240"
                                Rectangle { width: parent.parent.visualPosition * parent.width; height: parent.height; radius: 2; color: theme.accentColor }
                            }
                            handle: Rectangle {
                                x: parent.leftPadding + parent.visualPosition * (parent.availableWidth - width)
                                y: parent.topPadding + parent.availableHeight / 2 - height / 2
                                width: 18; height: 18; radius: 9
                                color: parent.pressed ? Qt.lighter(theme.accentColor, 1.2) : theme.accentColor
                            }
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
                            source: advancedExpanded ? appBridge.asset_url("expand_less.svg") : appBridge.asset_url("expand_more.svg")
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

                        Item {
                            Layout.fillWidth: true; implicitHeight: 36
                            Label { anchors.left: parent.left; anchors.right: disableOcrSwitch.left; anchors.rightMargin: 12; anchors.verticalCenter: parent.verticalCenter; text: "Disable OCR"; color: theme.textPrimary; font.pixelSize: 15 }
                            Switch {
                                id: disableOcrSwitch; anchors.right: parent.right; anchors.verticalCenter: parent.verticalCenter
                                checked: appBridge.disable_ocr; onToggled: appBridge.set_disable_ocr_value(checked)
                                indicator: Rectangle { implicitWidth: 48; implicitHeight: 26; x: disableOcrSwitch.leftPadding; y: disableOcrSwitch.height / 2 - height / 2; radius: 13; color: disableOcrSwitch.checked ? theme.accentColor : "#555"
                                    Rectangle { x: disableOcrSwitch.checked ? parent.width - width - 3 : 3; y: (parent.height - height) / 2; width: 20; height: 20; radius: 10; color: "white"; Behavior on x { NumberAnimation { duration: 150 } } }
                                }
                            }
                        }

                        Item {
                            Layout.fillWidth: true; implicitHeight: 36
                            Label { anchors.left: parent.left; anchors.right: disableAutoSwitch.left; anchors.rightMargin: 12; anchors.verticalCenter: parent.verticalCenter; text: "Disable automatic language detection"; color: theme.textPrimary; font.pixelSize: 15; wrapMode: Text.WordWrap }
                            Switch {
                                id: disableAutoSwitch; anchors.right: parent.right; anchors.verticalCenter: parent.verticalCenter
                                checked: appBridge.disable_auto_detect; onToggled: appBridge.set_disable_auto_detect_value(checked)
                                indicator: Rectangle { implicitWidth: 48; implicitHeight: 26; x: disableAutoSwitch.leftPadding; y: disableAutoSwitch.height / 2 - height / 2; radius: 13; color: disableAutoSwitch.checked ? theme.accentColor : "#555"
                                    Rectangle { x: disableAutoSwitch.checked ? parent.width - width - 3 : 3; y: (parent.height - height) / 2; width: 20; height: 20; radius: 10; color: "white"; Behavior on x { NumberAnimation { duration: 150 } } }
                                }
                            }
                        }

                        Item {
                            Layout.fillWidth: true; implicitHeight: 36
                            Label { anchors.left: parent.left; anchors.right: translitOutSwitch.left; anchors.rightMargin: 12; anchors.verticalCenter: parent.verticalCenter; text: "Show transliteration for output"; color: theme.textPrimary; font.pixelSize: 15 }
                            Switch {
                                id: translitOutSwitch; anchors.right: parent.right; anchors.verticalCenter: parent.verticalCenter
                                checked: appBridge.show_transliteration_output; onToggled: appBridge.set_show_transliteration_output_value(checked)
                                indicator: Rectangle { implicitWidth: 48; implicitHeight: 26; x: translitOutSwitch.leftPadding; y: translitOutSwitch.height / 2 - height / 2; radius: 13; color: translitOutSwitch.checked ? theme.accentColor : "#555"
                                    Rectangle { x: translitOutSwitch.checked ? parent.width - width - 3 : 3; y: (parent.height - height) / 2; width: 20; height: 20; radius: 10; color: "white"; Behavior on x { NumberAnimation { duration: 150 } } }
                                }
                            }
                        }

                        Item {
                            Layout.fillWidth: true; implicitHeight: 36
                            Label { anchors.left: parent.left; anchors.right: translitInSwitch.left; anchors.rightMargin: 12; anchors.verticalCenter: parent.verticalCenter; text: "Show transliteration for input"; color: theme.textPrimary; font.pixelSize: 15 }
                            Switch {
                                id: translitInSwitch; anchors.right: parent.right; anchors.verticalCenter: parent.verticalCenter
                                checked: appBridge.show_transliteration_input; onToggled: appBridge.set_show_transliteration_input_value(checked)
                                indicator: Rectangle { implicitWidth: 48; implicitHeight: 26; x: translitInSwitch.leftPadding; y: translitInSwitch.height / 2 - height / 2; radius: 13; color: translitInSwitch.checked ? theme.accentColor : "#555"
                                    Rectangle { x: translitInSwitch.checked ? parent.width - width - 3 : 3; y: (parent.height - height) / 2; width: 20; height: 20; radius: 10; color: "white"; Behavior on x { NumberAnimation { duration: 150 } } }
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
