import QtQuick 2.15
import Lomiri.Content 1.1

Item {
    id: root
    property var appBridge
    property var activeTransfer: null
    property var sharedItem: null
    property bool transferCharged: false

    function share(url) {
        if (!url) {
            return
        }

        if (sharedItem) {
            sharedItem.destroy()
            sharedItem = null
        }

        sharedItem = shareItemComponent.createObject(root, { "url": url })
        transferCharged = false
        activeTransfer = sharePeer.request()
        if (!activeTransfer || !sharedItem) {
            cleanupTransfer()
            return
        }
    }

    function cleanupTransfer() {
        activeTransfer = null
        transferCharged = false
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

            if (!transferCharged && activeTransfer.state === ContentTransfer.InProgress && sharedItem) {
                activeTransfer.items = [sharedItem]
                activeTransfer.state = ContentTransfer.Charged
                transferCharged = true
                return
            }

            if (activeTransfer.state === ContentTransfer.Aborted ||
                    activeTransfer.state === ContentTransfer.Charged ||
                    activeTransfer.state === ContentTransfer.Finalized ||
                    activeTransfer.state === ContentTransfer.Collected) {
                cleanupTransfer()
            }
        }
    }
}
