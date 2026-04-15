import QtQuick 2.15
import Lomiri.Content 1.1

Item {
    id: root
    property var appBridge
    property string pendingUrl: ""
    property var activeTransfer: null
    property var sharedItem: null

    function share(url) {
        if (!url) {
            return
        }

        pendingUrl = url
        picker.visible = true
    }

    function cleanupTransfer() {
        activeTransfer = null
        pendingUrl = ""
        if (sharedItem) {
            sharedItem.destroy()
            sharedItem = null
        }
    }

    ContentPeerPicker {
        id: picker
        anchors.fill: parent
        visible: false
        showTitle: false
        contentType: ContentType.Pictures
        handler: ContentHandler.Share
        onPeerSelected: {
            peer.selectionType = ContentTransfer.Single
            activeTransfer = peer.request()
            visible = false
            if (!activeTransfer) {
                cleanupTransfer()
            }
        }
        onCancelPressed: {
            visible = false
            cleanupTransfer()
        }
    }

    Component {
        id: shareItemComponent

        ContentItem {}
    }

    Connections {
        target: ContentHub
        ignoreUnknownSignals: true

        function onShareRequested(transfer) {
            if (!pendingUrl.length) {
                transfer.state = ContentTransfer.Aborted
                return
            }

            if (sharedItem) {
                sharedItem.destroy()
                sharedItem = null
            }

            sharedItem = shareItemComponent.createObject(root, { "url": pendingUrl })
            if (!sharedItem) {
                transfer.state = ContentTransfer.Aborted
                cleanupTransfer()
                return
            }

            activeTransfer = transfer
            transfer.items = [sharedItem]
            transfer.state = ContentTransfer.Charged
        }
    }

    Connections {
        target: activeTransfer
        ignoreUnknownSignals: true

        function onStateChanged() {
            if (!activeTransfer) {
                return
            }

            if (activeTransfer.state === ContentTransfer.Collected) {
                activeTransfer.finalize()
                cleanupTransfer()
                return
            }

            if (activeTransfer.state === ContentTransfer.Aborted ||
                    activeTransfer.state === ContentTransfer.Finalized) {
                cleanupTransfer()
            }
        }
    }
}
