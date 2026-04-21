import QtQuick 2.15

Canvas {
    id: root
    UiScale { id: ui }
    property real progress: 0.0   // 0.0 to 1.0, ignored if indeterminate
    property bool indeterminate: false
    property color progressColor: "#A8BCFF"
    property color trackColor: "#303240"
    property real lineWidth: ui.dp(2)

    width: ui.dp(18)
    height: ui.dp(18)

    property real _rotation: 0

    onProgressChanged: requestPaint()
    onIndeterminateChanged: requestPaint()
    on_RotationChanged: requestPaint()

    NumberAnimation on _rotation {
        running: root.indeterminate && root.visible
        from: 0
        to: 360
        duration: 1000
        loops: Animation.Infinite
    }

    onPaint: {
        var ctx = getContext("2d")
        var cx = width / 2
        var cy = height / 2
        var r = Math.min(cx, cy) - lineWidth
        ctx.reset()

        // Track
        ctx.beginPath()
        ctx.arc(cx, cy, r, 0, 2 * Math.PI)
        ctx.strokeStyle = trackColor
        ctx.lineWidth = lineWidth
        ctx.stroke()

        // Progress arc
        ctx.beginPath()
        var startAngle = -Math.PI / 2
        if (indeterminate) {
            startAngle += _rotation * Math.PI / 180
            ctx.arc(cx, cy, r, startAngle, startAngle + Math.PI * 0.75)
        } else {
            ctx.arc(cx, cy, r, startAngle, startAngle + progress * 2 * Math.PI)
        }
        ctx.strokeStyle = progressColor
        ctx.lineWidth = lineWidth
        ctx.lineCap = "round"
        ctx.stroke()
    }
}
