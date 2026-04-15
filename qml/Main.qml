import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Window 2.15

ApplicationWindow {
    id: root
    visible: true
    visibility: app.desktop_mode ? Window.Windowed : Window.FullScreen
    width: app.desktop_mode ? 600 : 720
    height: app.desktop_mode ? 1024 : 1280
    minimumWidth: app.desktop_mode ? 600 : 360
    minimumHeight: app.desktop_mode ? 1024 : 640
    title: "Offline Translator"

    AppTheme { id: theme }
    color: theme.backgroundColor

    header: TopBar {
        visible: app.current_screen === 1 && !app.image_viewer_open
        appBridge: app
        theme: theme
    }

    SetupScreen {
        anchors.fill: parent
        visible: app.current_screen === 0
        appBridge: app
        theme: theme
    }

    TranslationScreen {
        anchors.fill: parent
        visible: app.current_screen === 1
        appBridge: app
        theme: theme
    }

    SettingsScreen {
        anchors.fill: parent
        visible: app.current_screen === 2
        appBridge: app
        theme: theme
    }

    ManageLanguagesScreen {
        anchors.fill: parent
        visible: app.current_screen === 3
        appBridge: app
        theme: theme
    }
}
