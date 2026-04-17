import QtQuick 2.15
import TranslatorUi 1.0

Item {
    id: root
    property var appBridge
    property int imageMargin: 0
    property bool interactive: false
    signal imageClicked()
    UiScale { id: ui }

    Item {
        id: paintedBounds
        property real sourceWidth: root.appBridge && root.appBridge.processed_image_width > 0
                                   ? root.appBridge.processed_image_width
                                   : Math.max(selectedImage.sourceSize.width, selectedImage.implicitWidth)
        property real sourceHeight: root.appBridge && root.appBridge.processed_image_height > 0
                                    ? root.appBridge.processed_image_height
                                    : Math.max(selectedImage.sourceSize.height, selectedImage.implicitHeight)
        property real availableWidth: Math.max(0, root.width - root.imageMargin * 2)
        property real availableHeight: Math.max(0, root.height - root.imageMargin * 2)
        property real scaleFactor: sourceWidth > 0 && sourceHeight > 0
                                   ? Math.min(availableWidth / sourceWidth, availableHeight / sourceHeight)
                                   : 0
        x: root.imageMargin + (availableWidth - width) / 2
        y: root.imageMargin + (availableHeight - height) / 2
        width: sourceWidth > 0 && sourceHeight > 0 ? sourceWidth * scaleFactor : 0
        height: sourceWidth > 0 && sourceHeight > 0 ? sourceHeight * scaleFactor : 0
    }

    Image {
        id: selectedImage
        x: paintedBounds.x
        y: paintedBounds.y
        width: paintedBounds.width
        height: paintedBounds.height
        source: root.appBridge ? root.appBridge.selected_image_url : ""
        fillMode: Image.PreserveAspectFit
        asynchronous: true
        cache: false
        smooth: true
        opacity: root.appBridge && root.appBridge.processed_image_width > 0 && root.appBridge.processed_image_height > 0 ? 0 : 1
    }

    RenderedImageItem {
        id: processedImage
        x: paintedBounds.x
        y: paintedBounds.y
        width: paintedBounds.width
        height: paintedBounds.height
        visible: root.appBridge
                 && root.appBridge.processed_image_width > 0
                 && root.appBridge.processed_image_height > 0
        image: root.appBridge ? root.appBridge.processed_image : undefined
    }

    Item {
        anchors.fill: parent
        visible: root.appBridge
                 && root.appBridge.processed_image_width > 0
                 && root.appBridge.processed_image_height > 0

            Repeater {
                model: root.appBridge ? root.appBridge.image_overlay_model : null

                Item {
                    id: blockItem
                    width: paintedBounds.width
                    height: paintedBounds.height
                    property var lineRects: []
                    property var fittedLines: []
                    property real fittedPixelSize: Math.max(8, Math.floor(avg_line_height))
                    property real minLineHeight: 0

                    function skipSeparators(text, startIndex) {
                        var index = startIndex
                        while (index < text.length) {
                            var ch = text.charAt(index)
                            if (ch === " " || ch === "\t" || ch === "\r" || ch === "\n") {
                                index += 1
                            } else {
                                break
                            }
                        }
                        return index
                    }

                    function trimLineText(text) {
                        return text.replace(/[ \t\r\n]+$/g, "")
                    }

                    function findBreakIndex(text, startIndex, rawEndIndex) {
                        for (var index = rawEndIndex - 1; index >= startIndex; index -= 1) {
                            var ch = text.charAt(index)
                            if (ch === " " || ch === "\t") {
                                return index + 1
                            }
                            if (ch === "\n") {
                                return index
                            }
                        }
                        return rawEndIndex
                    }

                    function countFittingChars(text, startIndex, endIndex, maxWidth) {
                        var available = text.slice(startIndex, endIndex)
                        if (!available.length) {
                            return 0
                        }

                        var low = 0
                        var high = available.length
                        var best = 0

                        while (low <= high) {
                            var mid = Math.floor((low + high) / 2)
                            metrics.text = available.slice(0, mid)
                            var measuredWidth = metrics.advanceWidth || metrics.width

                            if (measuredWidth <= maxWidth + 0.5) {
                                best = mid
                                low = mid + 1
                            } else {
                                high = mid - 1
                            }
                        }

                        return best
                    }

                    function fitTextToLines(text, rects, pixelSize) {
                        metrics.font.pixelSize = pixelSize
                        var startIndex = 0
                        var lines = []

                        for (var lineIndex = 0; lineIndex < rects.length; lineIndex += 1) {
                            startIndex = skipSeparators(text, startIndex)
                            if (startIndex >= text.length) {
                                break
                            }

                            var newlineIndex = text.indexOf("\n", startIndex)
                            if (newlineIndex === -1) {
                                newlineIndex = text.length
                            }

                            var countedChars = countFittingChars(
                                        text,
                                        startIndex,
                                        newlineIndex,
                                        rects[lineIndex].width)
                            if (countedChars <= 0) {
                                return { fits: false, lines: lines }
                            }

                            var rawEndIndex = startIndex + countedChars
                            var endIndex = rawEndIndex >= newlineIndex
                                    ? newlineIndex
                                    : findBreakIndex(text, startIndex, rawEndIndex)
                            if (endIndex <= startIndex) {
                                endIndex = rawEndIndex
                            }

                            var lineText = trimLineText(text.slice(startIndex, endIndex))
                            metrics.text = lineText || "Ag"
                            var measuredHeight = metrics.boundingRect ? metrics.boundingRect.height : 0
                            if (measuredHeight > rects[lineIndex].height) {
                                return { fits: false, lines: lines }
                            }

                            lines.push(lineText)
                            startIndex = endIndex
                        }

                        startIndex = skipSeparators(text, startIndex)
                        return { fits: startIndex >= text.length, lines: lines }
                    }

                    function rebuildLayout() {
                        var parsedRects = []
                        if (line_rects) {
                            try {
                                var parsed = JSON.parse(line_rects)
                                if (Array.isArray(parsed)) {
                                    parsedRects = parsed
                                }
                            } catch (error) {
                                parsedRects = []
                            }
                        }

                        lineRects = parsedRects
                        minLineHeight = parsedRects.length
                                ? parsedRects.reduce(function(minValue, rect) {
                                      return Math.min(minValue, rect.height)
                                  }, parsedRects[0].height)
                                : 0
                        if (!translated_text || !parsedRects.length) {
                            fittedPixelSize = Math.max(8, Math.floor(avg_line_height))
                            fittedLines = []
                            return
                        }

                        var minPixelSize = 8
                        var startPixelSize = Math.max(
                                    minPixelSize,
                                    Math.ceil(avg_line_height) + 4)
                        var lastAttempt = { fits: false, lines: [] }

                        for (var pixelSize = startPixelSize; pixelSize >= minPixelSize; pixelSize -= 1) {
                            lastAttempt = fitTextToLines(translated_text, parsedRects, pixelSize)
                            if (lastAttempt.fits) {
                                fittedPixelSize = pixelSize
                                fittedLines = lastAttempt.lines
                                return
                            }
                        }

                        fittedPixelSize = minPixelSize
                        fittedLines = lastAttempt.lines
                    }

                    Component.onCompleted: rebuildLayout()

                    TextMetrics {
                        id: metrics
                    }

                    Repeater {
                        model: blockItem.fittedLines.length

                        Text {
                            property var lineRect: blockItem.lineRects[index] || null
                            x: lineRect ? lineRect.x * paintedBounds.width / root.appBridge.processed_image_width : 0
                            y: lineRect ? lineRect.y * paintedBounds.height / root.appBridge.processed_image_height : 0
                            width: lineRect ? lineRect.width * paintedBounds.width / root.appBridge.processed_image_width : 0
                            height: lineRect ? lineRect.height * paintedBounds.height / root.appBridge.processed_image_height : 0
                            text: blockItem.fittedLines[index]
                            color: lineRect && lineRect.foreground_color ? lineRect.foreground_color : foreground_color
                            wrapMode: Text.NoWrap
                            clip: true
                            font.pixelSize: Math.max(
                                8,
                                blockItem.fittedPixelSize * paintedBounds.height / root.appBridge.processed_image_height
                            )
                            verticalAlignment: Text.AlignTop
                            renderType: Text.NativeRendering
                        }
                    }
                }
            }
        }

        MouseArea {
            visible: root.interactive && paintedBounds.width > 0 && paintedBounds.height > 0
            x: paintedBounds.x
            y: paintedBounds.y
            width: paintedBounds.width
            height: paintedBounds.height
            onClicked: root.imageClicked()
        }
    }
}
