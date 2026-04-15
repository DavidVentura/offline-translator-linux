import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15

ToolBar {
    id: topBarRoot
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

        DarkComboBox {
            id: fromCombo
            Layout.fillWidth: true
            Layout.preferredWidth: 1
            theme: topBarRoot.theme
            iconSource: appBridge.asset_url("expand_more.svg")
            model: appBridge.installed_from_language_names
            onActivated: appBridge.set_from(currentText)
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

        DarkComboBox {
            id: toCombo
            Layout.fillWidth: true
            Layout.preferredWidth: 1
            theme: topBarRoot.theme
            iconSource: appBridge.asset_url("expand_more.svg")
            model: appBridge.installed_to_language_names
            onActivated: appBridge.set_to(currentText)
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
