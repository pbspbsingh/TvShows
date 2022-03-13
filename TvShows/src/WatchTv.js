import React, { useEffect, useState, useRef } from 'react';
import {
    NativeEventEmitter,
    NativeModules,
    StyleSheet,
    ToastAndroid,
    View,
} from 'react-native';
import { useNavigation } from '@react-navigation/native';
import Video from 'react-native-video';

import { STYLES } from './styles';
import { hostname } from './utils';


const SKIP_DURATION = 15;

export default function WatchTv({ route: { params: { episodeParts, index } } }) {
    const [title, videoUrl] = episodeParts[index];
    let currentTime = 0;
    let totalDuration = 0;

    const navigation = useNavigation();
    const playerRef = useRef(null);
    useEffect(() => {
        ToastAndroid.show(title, ToastAndroid.LONG);
        const eventEmitter = new NativeEventEmitter(NativeModules.KeyBoardModule);
        const eventListener = eventEmitter.addListener('keyEvent', (event) => {
            let { current: player } = playerRef;
            console.log('current', currentTime, totalDuration);
            switch (event.keyCode) {
                case 21: {
                    if (player != null) {
                        currentTime -= SKIP_DURATION;
                        if (currentTime < 0) {
                            currentTime = 0;
                        }
                        player.seek(currentTime);
                    }
                    break;
                }
                case 22: {
                    if (player != null) {
                        currentTime += SKIP_DURATION;
                        if (currentTime > totalDuration) {
                            currentTime = totalDuration;
                        }
                        player.seek(currentTime);
                    }
                    break;
                }    
                default:
                    ToastAndroid.show(`Unhandled key: ${event.keyCode}`, ToastAndroid.SHORT);
            }
        });
        return () => eventListener.remove();
    }, []);


    return (
        <View style={[STYLES.fullScreen, { padding: 0 }]}>
            <Video
                ref={playerRef}
                style={styles.video}
                controls={true}
                resizeMode='contain'
                focusable={true}
                source={{ uri: `http://${hostname()}${videoUrl}` }}
                onError={e => ToastAndroid.show('Playback Error, please go back and try again', ToastAndroid.LONG)}
                onProgress={(evt) => {
                    console.log(evt);
                    currentTime = evt.currentTime;
                    totalDuration = evt.seekableDuration;
                }}
                onEnd={() => {
                    if (index < episodeParts.length - 1) {
                        navigation.replace('WatchTv', { episodeParts, index: index + 1 });
                    }
                }}
            />
        </View>
    );
}

const styles = StyleSheet.create({
    video: {
        width: '100%',
        height: '100%',
    },
});