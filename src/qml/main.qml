import QtQuick 2.9
import QtQuick.Window 2.2
import QtMultimedia 5.0
import Main 1.0

Window {
    visible: true
    width: 640
    height: 480
    title: qsTr("Hello World")
    color: "black"
    visibility: "Maximized"

    MediaPlayer {
        id: player
        source: videoPath
        autoPlay: true
    }

    VideoOutput {
        id: videoOutput
        source: player
        anchors.fill: parent
    }

    Orientation {
        y: 0
        x: 0
        width: parent.width
        height: parent.height

        quat: quaternion
    }

    VideoHUD {
        y: videoOutput.contentRect.y
        x: videoOutput.contentRect.x
        width: videoOutput.contentRect.width
        height: videoOutput.contentRect.height
    }

    SeekControl {
        anchors {
            left: parent.left
            right: parent.right
            margins: 10
            bottom: parent.bottom
        }
        duration: player.duration
        playPosition: player.position
        onSeekPositionChanged: player.seek(seekPosition);
    }
}
