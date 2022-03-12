import React, { useEffect, useState } from 'react';
import {
    ActivityIndicator,
    FlatList,
    StyleSheet,
    Text,
    TouchableOpacity,
    View
} from 'react-native';
import { useNavigation } from '@react-navigation/native';

import { ErrorScreen, Loader } from './CommonScreens';
import { COLORS, STYLES } from './styles';
import { abortGet, get } from './utils';


export default function TvEpisode({ route: { params: { tvChannel, tvShow, episode } } }) {
    const [state, dispatch] = useState({ status: 'loading', episodeParts: [] });
    useEffect(() => {
        get(`/episode/${encodeURIComponent(tvChannel)}/${encodeURIComponent(tvShow)}/${encodeURIComponent(episode)}`)
            .then(res => dispatch({ status: 'loaded', episodeParts: res }))
            .catch(e => dispatch({
                status: 'error',
                error: e,
            }));
        return abortGet;
    }, []);

    return (
        <View style={STYLES.fullScreen}>
            {state.status === 'loading' && <Loader />}
            {state.status === 'error' && <ErrorScreen subTitle={state.error.message} />}
            {state.status === 'loaded' && <FlatList
                contentContainerStyle={styles.list}
                data={state.episodeParts}
                focusable={true}
                keyExtractor={(item, index) => `${index}:${item[0]}`}
                renderItem={({ item, index }) => <Episode index={index} title={item[0]} videoUrl={item[1]} />}
            />}
        </View>
    );
}

function Episode({ index, title, videoUrl }) {
    const navigation = useNavigation();
    const [isFocused, focusDispatch] = useState(false);
    return (<TouchableOpacity
        style={[styles.eps, isFocused && STYLES.focused]}
        onFocus={() => focusDispatch(true)}
        onBlur={() => focusDispatch(false)}
        onPress={(e) => {
            if (e.target != null) {
                navigation.push('WatchTv', { title, videoUrl });
            }
        }}>
        <Text style={styles.episodeTitle}>{`${index + 1}. ${title}`}</Text>
    </TouchableOpacity>);
}

const styles = StyleSheet.create({
    list: {
        justifyContent: 'center',
        alignItems: 'center',
    },
    eps: {
        marginVertical: 3,
        paddingVertical: 5,
        paddingHorizontal: 10,
    },
    episodeTitle: {
        color: COLORS.primaryLighter,
        fontSize: 20,
    }
});