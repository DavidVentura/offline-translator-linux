import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
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

    AppTheme {
        id: theme
    }

    color: theme.backgroundColor

    header: TopBar {
        visible: app.current_screen === 1
        appBridge: app
        theme: theme
    }

    StackLayout {
        anchors.fill: parent
        currentIndex: app.current_screen

        SetupScreen {
            appBridge: app
            theme: theme
            manageModel: manageLanguagesModel
        }

        TranslationScreen {
            appBridge: app
            theme: theme
        }

        SettingsScreen {
            appBridge: app
            theme: theme
        }

        ManageLanguagesScreen {
            appBridge: app
            theme: theme
            manageModel: manageLanguagesModel
        }
    }
}
