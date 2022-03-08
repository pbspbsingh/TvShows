import { StyleSheet } from "react-native";

export const COLORS = {
    primary: '#a697ce',
    primaryDark: '#917fc3',
    primaryDarker: '#8773bd',
    primaryDarkest: '#6950aa',
    primaryLight: '#bbafd9',
    primaryLighter: '#c5bbdf',
    primaryLightest: '#e4e0f0',
    text: '#f7f7ff',
    background: '#414360',
    backgroundSecondary: '#585b82',
    highlight: '#9692ff',
    border: '#f7f7ff',
    error: 'rgb(255,121,198)'
};

export const STYLES = StyleSheet.create({
    fullScreen: {
        flex: 1,
        justifyContent: 'center',
        alignContent: 'center',
        padding: 10,
    },
    debug: {
        borderColor: '#fff',
        borderWidth: 1,
    },
    focused: {
        backgroundColor: COLORS.backgroundSecondary,
        shadowColor: '#fff',
        borderRadius: 4,
        elevation: 3,
    },
});