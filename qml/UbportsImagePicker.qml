import QtQuick 2.15
import Lomiri.Content 1.1

Item {
    id: root
    property var appBridge
    property var activeTransfer: null

    function open() {
        picker.visible = true
    }

    ContentPeerPicker {
        id: picker
        anchors.fill: parent
        visible: false
        showTitle: true
        headerText: "Choose from"
        contentType: ContentType.Pictures
        handler: ContentHandler.Source

        onCancelPressed: {
            visible = false
            root.activeTransfer = null
        }

        onPeerSelected: {
            visible = false
            if (peer) {
                peer.selectionType = ContentTransfer.Single
                root.activeTransfer = peer.request()
            }
        }
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
                root.activeTransfer = null
            } else if (activeTransfer.state === ContentTransfer.Aborted ||
                       activeTransfer.state === ContentTransfer.Finalized) {
                root.activeTransfer = null
            }
        }
    }
}
