import QtQuick 2.15
import Lomiri.Content 1.1

Item {
    property var appBridge
    property var activeTransfer: null

    function open() {
        activeTransfer = sourcePeer.request()
    }

    ContentPeer {
        id: sourcePeer
        contentType: ContentType.Pictures
        handler: ContentHandler.Source
        selectionType: ContentTransfer.Single
    }

    Connections {
        target: activeTransfer
        ignoreUnknownSignals: true

        function onStateChanged() {
            if (!activeTransfer) {
                return
            }

            if (activeTransfer.state === ContentTransfer.Charged &&
                    activeTransfer.items &&
                    activeTransfer.items.length > 0) {
                appBridge.process_image_selection(activeTransfer.items[0].url.toString())
            }
        }
    }
}
