import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15

ToolBar {
    property var appBridge
    property var theme

    function syncComboBox(comboBox, value) {
        const index = comboBox.find(value)
        comboBox.currentIndex = index >= 0 ? index : 0
    }

    background: Rectangle {
        color: theme.backgroundColor
    }

    Component.onCompleted: {
        syncComboBox(fromCombo, appBridge.source_language_name)
        syncComboBox(toCombo, appBridge.target_language_name)
    }

    Connections {
        target: appBridge

        function onSource_language_name_changed() {
            syncComboBox(fromCombo, appBridge.source_language_name)
        }

        function onTarget_language_name_changed() {
            syncComboBox(toCombo, appBridge.target_language_name)
        }

        function onInstalled_from_language_names_changed() {
            syncComboBox(fromCombo, appBridge.source_language_name)
        }

        function onInstalled_to_language_names_changed() {
            syncComboBox(toCombo, appBridge.target_language_name)
        }
    }

    RowLayout {
        anchors.fill: parent
        anchors.margins: 8
        spacing: 8

        ComboBox {
            id: fromCombo
            Layout.fillWidth: true
            Layout.preferredWidth: 1
            model: appBridge.installed_from_language_names
            onActivated: appBridge.set_from(currentText)

            contentItem: Label {
                leftPadding: 10
                rightPadding: fromCombo.indicator.width + 10
                text: fromCombo.displayText
                color: theme.textPrimary
                verticalAlignment: Text.AlignVCenter
                elide: Text.ElideRight
            }

            background: Rectangle {
                radius: 4
                color: theme.backgroundElevated
                border.width: 1
                border.color: theme.borderColor
            }

            indicator: Image {
                source: appBridge.asset_url("expand_more.svg")
                width: 16; height: 16
                x: fromCombo.width - width - 10
                y: (fromCombo.height - height) / 2
            }

            delegate: ItemDelegate {
                width: fromCombo.width
                contentItem: Label {
                    text: modelData
                    color: theme.textPrimary
                    verticalAlignment: Text.AlignVCenter
                }
                background: Rectangle {
                    color: highlighted ? theme.surfaceAltColor : theme.surfaceColor
                }
                highlighted: fromCombo.highlightedIndex === index
            }

            popup: Popup {
                y: fromCombo.height
                width: fromCombo.width
                implicitHeight: contentItem.implicitHeight
                padding: 1

                contentItem: ListView {
                    clip: true
                    implicitHeight: contentHeight
                    model: fromCombo.popup.visible ? fromCombo.delegateModel : null
                    currentIndex: fromCombo.highlightedIndex
                }

                background: Rectangle {
                    color: theme.surfaceColor
                    border.color: theme.borderColor
                    radius: 4
                }
            }
        }

        ToolButton {
            display: AbstractButton.IconOnly
            icon.source: appBridge.asset_url("swap.svg")
            icon.width: 20
            icon.height: 20
            text: "Swap"
            enabled: appBridge.swap_enabled
            onClicked: appBridge.swap_languages()
        }

        ComboBox {
            id: toCombo
            Layout.fillWidth: true
            Layout.preferredWidth: 1
            model: appBridge.installed_to_language_names
            onActivated: appBridge.set_to(currentText)

            contentItem: Label {
                leftPadding: 10
                rightPadding: toCombo.indicator.width + 10
                text: toCombo.displayText
                color: theme.textPrimary
                verticalAlignment: Text.AlignVCenter
                elide: Text.ElideRight
            }

            background: Rectangle {
                radius: 4
                color: theme.backgroundElevated
                border.width: 1
                border.color: theme.borderColor
            }

            indicator: Image {
                source: appBridge.asset_url("expand_more.svg")
                width: 16; height: 16
                x: toCombo.width - width - 10
                y: (toCombo.height - height) / 2
            }

            delegate: ItemDelegate {
                width: toCombo.width
                contentItem: Label {
                    text: modelData
                    color: theme.textPrimary
                    verticalAlignment: Text.AlignVCenter
                }
                background: Rectangle {
                    color: highlighted ? theme.surfaceAltColor : theme.surfaceColor
                }
                highlighted: toCombo.highlightedIndex === index
            }

            popup: Popup {
                y: toCombo.height
                width: toCombo.width
                implicitHeight: contentItem.implicitHeight
                padding: 1

                contentItem: ListView {
                    clip: true
                    implicitHeight: contentHeight
                    model: toCombo.popup.visible ? toCombo.delegateModel : null
                    currentIndex: toCombo.highlightedIndex
                }

                background: Rectangle {
                    color: theme.surfaceColor
                    border.color: theme.borderColor
                    radius: 4
                }
            }
        }

        ToolButton {
            display: AbstractButton.IconOnly
            icon.source: appBridge.asset_url("settings.svg")
            icon.width: 20
            icon.height: 20
            text: "Settings"
            onClicked: appBridge.show_settings()
        }
    }
}
