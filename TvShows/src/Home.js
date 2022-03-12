import React, {
    useEffect,
    useState,
    useReducer
} from 'react';
import {
    FlatList,
    Image,
    StyleSheet,
    Text,
    TouchableOpacity,
    View
} from 'react-native';
import { useNavigation } from '@react-navigation/native';

import { ErrorScreen, Loader } from './CommonScreens';
import { COLORS, STYLES } from './styles';
import { abortGet, get } from './utils';

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
            throw new Error(`${action} is not handled`);
    }
    return newState;
}

export default function Home() {
    const [state, dispatch] = useReducer(reducer, { status: 'loading', tvChannels: [] });
    
    useEffect(() => {
        get('home')
            .then(tvChannels => dispatch({ name: 'LOADED', tvChannels }))
            .catch(e => dispatch({ name: 'ERROR', error: e }));
        return abortGet;
    }, []);

    return <View style={STYLES.fullScreen}>
        {state.status == 'loading' && <Loader />}
        {state.status == 'error' && <ErrorScreen subTitle={state.errorMessage} />}
        {state.status == 'done' && <View>
            <FlatList
                data={state.tvChannels}
                keyExtractor={(tv) => tv.title}
                numColumns={3}
                focusable={true}
                renderItem={({ item }) => <Channel tvChannel={item} />}
            />
        </View>}
    </View>;
}

function Channel({ tvChannel }) {
    const [isFocused, focusDispatch] = useState(false);
    const navigation = useNavigation();
    return (
        <TouchableOpacity style={styles.tvShowWrapper}
            onFocus={() => focusDispatch(true)}
            onBlur={() => focusDispatch(false)}
            onPress={(e) => {
                if (e.target != null) {
                    navigation.push("TvChannel", tvChannel);
                }
            }}>
            <View style={[styles.tvShow, isFocused && STYLES.focused]}>
                <Image source={{ uri: tvChannel.icon }} style={styles.icon} />
                <Text style={styles.title}>{tvChannel.title}</Text>
            </View>
        </TouchableOpacity>
    );
}

const styles = StyleSheet.create({
    tvShowWrapper: {
        width: '33%',
        padding: 5,
    },
    tvShow: {
        alignItems: 'center',
        padding: 10,
    },
    icon: {
        height: 50,
        width: 50,
    },
    title: {
        textAlign: 'center',
        marginTop: 10,
        fontSize: 20,
        color: COLORS.primaryLightest,
    },
});