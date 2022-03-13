import React, { useEffect, useState } from 'react';
import {
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
    const navigation = useNavigation();

    useEffect(() => {
        get(`/episode/${encodeURIComponent(tvChannel)}/${encodeURIComponent(tvShow)}/${encodeURIComponent(episode)}`)
            .then(res => {
                if (res.length === 0) {
                    dispatch({
                        status: 'error',
                        error: { message: 'Failed to download episode' },
                    })
                }
                else if (res.length === 1) {
                    navigation.replace('WatchTv', {
                        episodeParts: res,
                        index: 0
                    });
                } else {
                    dispatch({
                        status: 'loaded',
                        episodeParts: res
                    });
                }
            })
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
                renderItem={({ item, index }) => <Episode index={index} episodeParts={state.episodeParts} />}
            />}
        </View>
    );
}

function Episode({ index, episodeParts }) {
    const navigation = useNavigation();
    const [isFocused, focusDispatch] = useState(false);
    const [title] = episodeParts[index];
    return (<TouchableOpacity
        style={[styles.eps, isFocused && STYLES.focused]}
        onFocus={() => focusDispatch(true)}
        onBlur={() => focusDispatch(false)}
        onPress={(e) => {
            if (e.target != null) {
                navigation.push('WatchTv', { episodeParts, index });
            }
        }}>
        <Text style={styles.episodeTitle}>{`${index + 1}. ${title}`}</Text>
    </TouchableOpacity>);
}

const styles = StyleSheet.create({
    list: {
        flex: 1,
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