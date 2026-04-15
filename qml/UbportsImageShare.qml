import QtQuick 2.15
import Lomiri.Content 1.1

Item {
    id: root
    property var appBridge
    property var activeTransfer: null
    property var sharedItem: null

    function share(url) {
        if (!url) {
            return
        }

        if (sharedItem) {
            sharedItem.destroy()
            sharedItem = null
        }

        sharedItem = shareItemComponent.createObject(root, { "url": url })
        activeTransfer = sharePeer.request()
        if (!activeTransfer || !sharedItem) {
            return
        }

        activeTransfer.items = [sharedItem]
        activeTransfer.start()
    }

    function cleanupTransfer() {
        activeTransfer = null
        if (sharedItem) {
            sharedItem.destroy()
            sharedItem = null
        }
    }

    ContentPeer {
        id: sharePeer
        contentType: ContentType.Pictures
        handler: ContentHandler.Share
        selectionType: ContentTransfer.Single
    }

    Component {
        id: shareItemComponent

        ContentItem {}
    }

    Connections {
        target: activeTransfer
        ignoreUnknownSignals: true

        function onStateChanged() {
            if (!activeTransfer) {
                return
            }

            if (activeTransfer.state === ContentTransfer.Aborted ||
                    activeTransfer.state === ContentTransfer.Finalized ||
                    activeTransfer.state === ContentTransfer.Collected) {
                cleanupTransfer()
            }
        }
    }
}
