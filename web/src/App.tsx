import React from 'react';
import { BrowserRouter, Route, Routes } from 'react-router-dom';

import Home from './Home';
import Parts from './Parts';
import TvShow from './TvShow';

export default function App(): JSX.Element {
  return (
    <BrowserRouter>
      <div className="app">
        <Routes>
          <Route path="/" element={<Home />} />
          <Route path="/tvshow/:channel/:tv_show" element={<TvShow />} />
          <Route path="/parts/:channel/:tv_show/:episode" element={<Parts />} />
        </Routes>
      </div>
    </BrowserRouter>
  );
}
