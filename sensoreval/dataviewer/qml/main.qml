import QtQuick 2.9
import QtQuick.Window 2.2
import QtMultimedia 5.0
import Main 1.0

Window {
    id: window
    visible: true
    width: 640
    height: 480
    title: qsTr("Hello World")
    color: "black"
    visibility: "Maximized"

    Item {
        anchors.fill: parent
        focus: true

        Keys.onSpacePressed: {
            if (player.playbackState == MediaPlayer.PausedState)
                player.play();
            else if (player.playbackState == MediaPlayer.StoppedState) {
                player.seek(main_videoStartOffset);
                player.play();
            }
            else
                player.pause();
        }

        Keys.onPressed: {
            if (event.key == Qt.Key_F) {
                if (window.visibility == Window.FullScreen) {
                    window.visibility = Window.Maximized;
                }
                else {
                    window.visibility = Window.FullScreen;
                }
                event.accepted = true;
            }
        }

        MediaPlayer {
            id: player
            objectName: "player"
            source: main_videoPath
            autoPlay: false

            Component.onCompleted: {
                seek(main_videoStartOffset);
                play();
            }
        }

        Connections {
            target: player
            onPositionChanged: {
                if (player.playbackState == MediaPlayer.PlayingState && player.duration > 0 && player.position >= main_videoEndOffset) {
                    player.stop();
                    player.seek(main_videoEndOffset);
                }
            }
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

            quat: main_quaternion
        }

        VideoHUD {
            id: hud
            objectName: "hud"
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
            duration: main_videoEndOffset - main_videoStartOffset
            playPosition: player.position - main_videoStartOffset
            onSeekPositionChanged: player.seek(seekPosition + main_videoStartOffset);
        }
    }
}
