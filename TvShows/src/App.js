import React from 'react';
import {
  Dimensions,
  Image,
  StyleSheet,
  Text,
  View,
} from 'react-native';
import { NavigationContainer } from '@react-navigation/native';
import { createNativeStackNavigator } from '@react-navigation/native-stack';

import Home from './Home';
import TvShow from './TvShow';
import TvEpisode from './TvEpisode';
import WatchTv from './WatchTv';

import { COLORS } from './styles';

const Stack = createNativeStackNavigator();

export default function App() {
  return (
    <NavigationContainer theme={theme}>
      <Stack.Navigator initialRouteName='Home'>
        <Stack.Screen name='Home' component={Home} options={{ headerShown: false }} />
        <Stack.Screen name='TvShow'
          component={TvShow}
          options={({ route }) => ({
            headerTitle: () => <NavHeader {...route} />
          })} />
        <Stack.Screen name='TvEpisode'
          component={TvEpisode}
          options={({ route }) => ({
            headerTitle: () => <NavHeader {...route} />
          })} />
        <Stack.Screen name='WatchTv' component={WatchTv} options={{ headerShown: false }} />
      </Stack.Navigator>
    </NavigationContainer>
  );
};

function NavHeader({ params: { title, icon } }) {
  const { width } = Dimensions.get('window');
  return <View style={[styles.navHeader, { width: width - 150 }]} >
    {icon && <Image source={{ uri: icon }} style={styles.icon} />}
    <Text style={styles.header}>{title}</Text>
  </View>;
}

const theme = {
  dark: true,
  colors: {
    primary: COLORS.primary,
    background: COLORS.background,
    card: COLORS.backgroundSecondary,
    text: COLORS.text,
    border: COLORS.border,
    notification: COLORS.highlight,
  }
};

const styles = StyleSheet.create({
  navHeader: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
  },
  icon: {
    height: 25,
    width: 25,
  },
  header: {
    marginLeft: 12,
    color: '#FFF',
    fontWeight: 'bold',
    fontSize: 18,
  },
});