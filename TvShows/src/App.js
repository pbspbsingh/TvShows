import React from 'react';
import {
  Image,
  StyleSheet,
  Text,
  View,
} from 'react-native';
import { NavigationContainer } from '@react-navigation/native';
import { createNativeStackNavigator } from '@react-navigation/native-stack';

import Home from './Home';
import TvChannel from './TvChannel';
import TvShow from './TvShow';
import { COLORS } from './styles';

const Stack = createNativeStackNavigator();

export default function App() {
  return (
    <NavigationContainer theme={theme}>
      <Stack.Navigator initialRouteName='Home'>
        <Stack.Screen name='Home' component={Home} options={{ headerShown: false }} />
        <Stack.Screen name='TvChannel'
          component={TvChannel}
          options={({ route }) => ({
            headerTitle: () => <NavHeader {...route} />
          })} />
        <Stack.Screen name='TvShow'
          component={TvShow}
          options={({ route }) => ({
            headerTitle: () => <NavHeader {...route} />
          })} />
      </Stack.Navigator>
    </NavigationContainer>
  );
};

function NavHeader({ params: { title, icon } }) {
  return <View style={styles.navHeader}>
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
    // marginLeft: -20,
    width: '90%',
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