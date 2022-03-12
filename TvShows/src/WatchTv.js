import React, { useEffect, useState } from 'react';
import {
    StyleSheet,
    ToastAndroid,
    View
} from 'react-native';
import Video from 'react-native-video';

import { STYLES } from './styles';
import { hostname } from './utils';



export default function WatchTv({ route: { params: { title, videoUrl } } }) {
    useEffect(() => {
        ToastAndroid.show(title, ToastAndroid.LONG);
    });
    console.log(`http://${hostname()}${videoUrl}`);
    return (
        <View style={STYLES.fullScreen}>
            <Video style={styles.video}
                controls={true}
                resizeMode='contain'
                source={{ uri: `http://${hostname()}${videoUrl}` }}
                onError={(e) => console.log(e)} />
        </View>
    );
}

const styles = StyleSheet.create({
    video: {
        width: '100%',
        height: '100%',
    },
});