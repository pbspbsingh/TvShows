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
    const [durationState, durationDispath] = useState({ current: 0, total: 0 });
    const [pauseState, pauseDispatch] = useState(false);

    const navigation = useNavigation();
    const playerRef = useRef(null);

    useEffect(() => {
        ToastAndroid.show(title, ToastAndroid.LONG);
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
                    pauseDispatch(!pauseState);
                    break;
                }
                case 89: { // Play previous
                    if (index > 0) {
                        navigation.replace('WatchTv', { episodeParts, index: index - 1 });
                    } else {
                        ToastAndroid.show(`First part: ${title}`, ToastAndroid.SHORT);
                    }
                    break;
                }
                case 90: { // Play next
                    if (index < episodeParts.length - 1) {
                        navigation.replace('WatchTv', { episodeParts, index: index + 1 });
                    } else {
                        ToastAndroid.show(`Last part: ${title}`, ToastAndroid.SHORT);
                    }
                    break;
                }
                default:
                    ToastAndroid.show(`Unhandled key: ${event.keyCode}`, ToastAndroid.SHORT);
            }
        });
        return () => eventListener.remove();
    }, [pauseState]);

    return (
        <View style={[STYLES.fullScreen, { padding: 0 }]}>
            <Video
                ref={playerRef}
                style={styles.video}
                controls={false}
                paused={pauseState}
                resizeMode='contain'
                focusable={false}
                fullscreen={true}
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
            {pauseState && <Text style={styles.pauseIcon}>⏸</Text>}
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
    pauseIcon: {
        position: 'absolute',
        left: '50%',
        fontSize: 40,
        opacity: .75,
    },
});
