import React, { useState } from 'react';
import {
    FlatList,
    StyleSheet,
    Text,
    TouchableOpacity,
    View
} from 'react-native';
import { useNavigation } from '@react-navigation/native';

import { COLORS, STYLES } from './styles';

export default function TvChannel({ route: { params: { title: channelTitle, soaps, completed } } }) {
    const sectionData = [{
        title: 'On Going',
        data: soaps,
    }];
    if (completed.length > 0) {
        sectionData.push({
            title: 'Completed',
            data: completed,
        });
    }
    return (
        <View style={[STYLES.fullScreen, styles.main]}>
            {sectionData.map((section) => (<View
                key={section.title}>
                <Text style={styles.headerText}>{section.title}:</Text>
                <FlatList
                    style={styles.list}
                    focusable={true}
                    data={section.data}
                    keyExtractor={(item) => item}
                    renderItem={({ item }) => <SaopTitle title={item} channelTitle={channelTitle} />}
                />
            </View>))}
        </View>
    );
}

function SaopTitle({ channelTitle, title }) {
    const [isFocused, focusDispatch] = useState(false);
    const navigation = useNavigation();
    return (<TouchableOpacity
        style={[styles.soap, isFocused && STYLES.focused]}
        onFocus={() => focusDispatch(true)}
        onBlur={() => focusDispatch(false)}
        onPress={(e) => {
            if (e.target != null) {
                navigation.push('TvShow', { channelTitle, title });
            }
        }}>
        <Text style={styles.soapTitle}>{title}</Text>
    </TouchableOpacity>);
}
const styles = StyleSheet.create({
    main: {
        flexDirection: 'row',
        justifyContent: 'space-around',
        padding: 20,
    },
    list: {
        marginLeft: 7,
    },
    headerText: {
        color: COLORS.primaryDarker,
        fontSize: 15,
    },
    soap: {
        marginVertical: 3,
        paddingVertical: 5,
        paddingHorizontal: 10,
    },
    soapTitle: {
        color: COLORS.primaryLighter,
        fontSize: 18,
    }
});