import React, {
    useEffect,
    useState,
    useReducer
} from 'react';
import {
    FlatList,
    Image,
    StyleSheet,
    ScrollView,
    Text,
    TouchableOpacity,
    View
} from 'react-native';
import { useNavigation } from '@react-navigation/native';

import { ErrorScreen, Loader } from './CommonScreens';
import { COLORS, STYLES } from './styles';
import { abortGet, get, hostname } from './utils';

function reducer(state, action) {
    let newState;
    switch (action.name) {
        case 'ERROR':
            newState = { ...state, status: 'error', errorMessage: action.error.toString() };
            break;
        case 'LOADED':
            newState = { ...state, status: 'done', tvChannels: action.tvChannels };
            break;
        default:
            throw new Error(`${action.name} is not handled`);
    }
    return newState;
}

export default function Home() {
    const [state, dispatch] = useReducer(reducer, {
        status: 'loading',
        tvChannels: {},
    });

    useEffect(() => {
        get('home')
            .then(tvChannels => dispatch({ name: 'LOADED', tvChannels: tvChannels }))
            .catch(e => dispatch({ name: 'ERROR', error: e }));
        return abortGet;
    }, []);

    return <View style={STYLES.fullScreen}>
        {state.status == 'loading' && <Loader />}
        {state.status == 'error' && <ErrorScreen subTitle={state.errorMessage} />}
        {state.status == 'done' && <ScrollView isFocused={true}>
            {Object.entries(state.tvChannels).map(([chnTitle, tvShows]) => <View key={chnTitle}>
                <Text style={styles.channelTitle}>{chnTitle} ({tvShows.length})</Text>
                {<FlatList
                    data={tvShows}
                    horizontal={true}
                    keyExtractor={(tvshow) => tvshow.title}
                    focusable={true}
                    renderItem={({ item }) => <TvShow tvChannel={chnTitle} tvShow={item} />}
                />}
            </View>)}
        </ScrollView>}
    </View>;
}

function TvShow({ tvChannel, tvShow, }) {
    const navigation = useNavigation();
    const [isFocused, focusDispatch] = useState(false);
    return (
        <TouchableOpacity
            style={[styles.tvShowWrapper, isFocused && STYLES.focused]}
            onFocus={() => focusDispatch(true)}
            onBlur={() => focusDispatch(false)}
            onPress={(e) => {
                if (e.target != null) {
                    navigation.push('TvShow', { tvChannel, title: tvShow.title, icon: `http://${hostname()}${tvShow.icon}` });
                    focusDispatch(false);
                }
            }}>
            <Image source={{ uri: `http://${hostname()}${tvShow.icon}` }} style={styles.icon} />
            <Text style={styles.tvShowTitle} numberOfLines={1}>{tvShow.title}</Text>
        </TouchableOpacity>
    );
}

const styles = StyleSheet.create({
    channelTitle: {
        color: COLORS.primary,
        marginBottom: 10,
        fontSize: 16,
    },
    tvShowWrapper: {
        width: 120,
        padding: 5,
        marginBottom: 10,
        alignItems: 'center',
    },
    icon: {
        height: 100,
        width: 100,
    },
    tvShowTitle: {
        color: COLORS.primaryLightest,
        marginTop: 2,
        fontSize: 12,
    },
});