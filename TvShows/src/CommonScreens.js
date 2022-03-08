import * as React from 'react';
import {
    ActivityIndicator,
    StyleSheet,
    Text,
    View
} from 'react-native';
import { COLORS, STYLES } from './styles';

export const Loader = () => (
    <View style={STYLES.fullScreen}>
        <ActivityIndicator color={COLORS.primaryLighter} size='large' />
        <Text style={styles.loader}>Loading...</Text>
    </View>
);

export const ErrorScreen = ({title, subTitle}) => (
    <View style={STYLES.fullScreen}>
        <Text style={styles.errorTitle}>😒</Text>
        <Text style={styles.errorTitle}>{title || 'Something went wrong, please try again!'}</Text>
        {subTitle && <Text style={styles.errorSubTitle}>{subTitle}</Text>}
    </View>
);

const styles = StyleSheet.create({
    loader: {
        textAlign: 'center',
        color: COLORS.primaryLightest,
        marginTop: 10,
    },
    errorTitle: {
        textAlign: 'center',
        fontSize: 25,
        color: COLORS.error,
    },
    errorSubTitle: {
        textAlign: 'center',
        fontSize: 14,
        marginTop: 8,
        color: COLORS.primaryDark,
    }
});