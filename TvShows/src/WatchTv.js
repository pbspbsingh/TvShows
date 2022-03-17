import React, { useEffect, useState, useRef } from 'react';
import {
    NativeEventEmitter,
    NativeModules,
    Slider,
    StyleSheet,
    Text,
    ToastAndroid,
    View,
} from 'react-native';
import { useNavigation } from '@react-navigation/native';
import Video from 'react-native-video';

import { COLORS, STYLES } from './styles';
import { hostname } from './utils';


const SKIP_DURATION = 15;

export default function WatchTv({ route: { params: { episodeParts, index } } }) {
    const eventEmitter = new NativeEventEmitter(NativeModules.KeyBoardModule);

    const [title, videoUrl] = episodeParts[index];
    const [fullScreen, fullScreenDispatch] = useState(false);
    const [durationState, durationDispath] = useState({ current: 0, total: 0 });
    const [[pauseState, playSpeed], pauseDispatch] = useState([false, 1]);

    const navigation = useNavigation();
    const playerRef = useRef(null);

    useEffect(() => {
        const eventListener = eventEmitter.addListener('keyEvent', (event) => {
            let { current: player } = playerRef;
            switch (event.keyCode) {
                case 21: {
                    if (player != null) {
                        //     currentTime -= SKIP_DURATION;
                        //     if (currentTime < 0) {
                        //         currentTime = 0;
                        //     }
                        //     player.seek(currentTime);
                    }
                    break;
                }
                case 22: {
                    if (player != null) {
                        //     currentTime += SKIP_DURATION;
                        //     if (currentTime > totalDuration) {
                        //         currentTime = totalDuration;
                        //     }
                        //     player.seek(currentTime);
                    }
                    break;
                }
                case 85: { // play|pause
                    if (playSpeed == 1) {
                        pauseDispatch([!pauseState, 1]);
                    } else {
                        pauseDispatch([false, 1]);
                    }
                    break;
                }
                case 89: { // <<-
                    if (pauseState) {
                        pauseDispatch([false, 1])
                    } else {
                        let newPlaySpeed = playSpeed;
                        if (newPlaySpeed >= 0.5 && newPlaySpeed <= 1) {
                            newPlaySpeed /= 2;
                        } else {
                            newPlaySpeed = 1;
                        }
                        pauseDispatch([false, newPlaySpeed]);
                    }
                    break;
                }
                case 90: { // ->>
                    if (pauseState) {
                        pauseDispatch([false, 1])
                    } else {
                        let newPlaySpeed = playSpeed;
                        if (newPlaySpeed >= 1 && newPlaySpeed <= 3) {
                            newPlaySpeed += 1;
                        } else {
                            newPlaySpeed = 1;
                        }
                        pauseDispatch([false, newPlaySpeed]);
                    }
                    break;
                }
                default:
                    ToastAndroid.show(`Unhandled key: ${event.keyCode}`, ToastAndroid.SHORT);
            }
        });
        return () => eventListener.remove();
    }, [pauseState, playSpeed]);

    useEffect(() => {
        ToastAndroid.show(title, ToastAndroid.LONG);
        setTimeout(() => fullScreenDispatch(true), 2500);
    }, []);

    return (
        <View style={[STYLES.fullScreen, { padding: 0 }]}>
            <Video
                ref={playerRef}
                style={styles.video}
                controls={false}
                paused={pauseState}
                rate={playSpeed}
                resizeMode='contain'
                focusable={false}
                fullscreen={fullScreen}
                source={{ uri: `http://${hostname()}${videoUrl}` }}
                onError={e => ToastAndroid.show('Playback Error, please go back and try again', ToastAndroid.LONG)}
                onProgress={(evt) => durationDispath({
                    current: evt.currentTime,
                    total: evt.seekableDuration,
                })}
                onEnd={() => {
                    if (index < episodeParts.length - 1) {
                        navigation.replace('WatchTv', { episodeParts, index: index + 1 });
                    } else {
                        navigation.goBack();
                    }
                }}
            />
            {durationState.total > 0 && <View style={styles.seekbar}>
                <Text style={styles.textCurrent}>{humanReadable(durationState.current)}</Text>
                <Slider style={{ flex: 1 }}
                    focusable={true}
                    step={SKIP_DURATION}
                    minimumValue={0}
                    value={durationState.current}
                    maximumValue={durationState.total}
                    onValueChange={(seekPosition) => {
                        let { current: player } = playerRef;
                        if (player != null) {
                            player.seek(seekPosition);
                        }
                    }} />
                <Text style={styles.textLeft}>{humanReadable(durationState.total - durationState.current)}</Text>
            </View>}
            {pauseState && <Text style={styles.overlayMsg}>⏯</Text>}
            {(!pauseState && playSpeed != 1) && <Text style={styles.overlayMsg}>{playSpeed}x</Text>}
        </View>
    );
}

function humanReadable(timeInSecs) {
    let time = parseInt(timeInSecs);
    const hours = parseInt(time / 60 / 60);
    time -= hours * 60 * 60;
    const minutes = parseInt(time / 60);
    time -= minutes * 60;
    const arr = [];
    if (hours > 0) {
        arr.push(hours);
    }
    arr.push(minutes);
    arr.push(time);
    return arr.map(String)
        .map(num => num.length == 1 ? '0' + num : num)
        .join(':');
}

const styles = StyleSheet.create({
    video: {
        width: '100%',
        height: '100%',
    },
    seekbar: {
        position: 'absolute',
        bottom: 0,
        width: '100%',
        flexDirection: 'row',
        alignItems: 'flex-end',
    },
    textCurrent: {
        color: COLORS.primaryLightest,
        marginLeft: 10,
    },
    textLeft: {
        color: COLORS.primaryLightest,
        marginRight: 10,
    },
    overlayMsg: {
        position: 'absolute',
        left: '46%',
        fontSize: 40,
        opacity: .75,
        color: COLORS.border,
    },
});
