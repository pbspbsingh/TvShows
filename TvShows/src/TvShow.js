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


export default function TvShow({ route: { params: { channelTitle, title } } }) {
    const [state, dispatch] = useState({ status: 'loading', episodes: [] });
    useEffect(() => {
        loadEpisodes(dispatch, channelTitle, title, false);
        return abortGet;
    }, []);

    return (
        <View style={STYLES.fullScreen}>
            {state.status === 'loading' && state.episodes.length === 0 && <Loader />}
            {state.status === 'error' && <ErrorScreen subTitle={state.error.message} />}
            {state.episodes.length > 0 && <FlatList
                contentContainerStyle={styles.list}
                data={state.episodes}
                focusable={true}
                keyExtractor={(item, index) => `${index}:${item}`}
                renderItem={({ item, index }) => <Episode
                    index={index}
                    tvChannel={channelTitle}
                    tvShow={title}
                    episode={item} />}
                onEndReachedThreshold={0.1}
                onEndReached={() => {
                    if (state.status === 'loaded' && state.has_more == true) {
                        dispatch({ ...state, status: 'loading' });
                        loadEpisodes(dispatch, channelTitle, title, true);
                    }
                }}
                ListFooterComponent={() => state.status === 'loading'
                    ? <ActivityIndicator size='small' color={COLORS.primaryLightest} />
                    : null}
            />}
        </View>
    );
}

function Episode({ index, tvChannel, tvShow, episode }) {
    const [isFocused, focusDispatch] = useState(false);
    const navigation = useNavigation();
    return (<TouchableOpacity
        style={[styles.eps, isFocused && STYLES.focused]}
        onFocus={() => focusDispatch(true)}
        onBlur={() => focusDispatch(false)}
        onPress={(e) => {
            if (e.target != null) {
                navigation.push('TvEpisode', { tvChannel, tvShow, episode, title: episode });
            }
        }}>
        <Text style={styles.episodeTitle}>{`${index + 1}. ${episode}`}</Text>
    </TouchableOpacity>);
}

function loadEpisodes(dispatch, tvChannel, tvShow, load_more) {
    get(`/episodes/${encodeURIComponent(tvChannel)}/${encodeURIComponent(tvShow)}`, { load_more })
        .then(res => dispatch({ status: 'loaded', ...res }))
        .catch(e => dispatch({
            status: 'error',
            error: e,
            episodes: [],
        }));
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