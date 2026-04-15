import QtQuick 2.15

Item {
    property var appBridge

    function share(url) {
        if (!url) {
            return
        }
        Qt.openUrlExternally(url)
    }
}
