import React, { useEffect, useState } from 'react';
import { Link, useParams } from 'react-router-dom';

import { ErrMsg, Loader } from './common';

import { Channels, TvShow, TvSoap } from './Home';
import { get } from './util';


type State = {
    status: 'error' | 'loading' | 'done';
    tvShows: Array<TvShow>,
    error?: string;
};

export default function Channel(): JSX.Element {
    const { channel } = useParams();

    const [state, dispatch] = useState<State>({
        status: 'loading',
        tvShows: [],
    });

    useEffect(() => {
        if (channel == null) {
            dispatch({
                status: 'error',
                tvShows: [],
                error: 'Invalid Channel',
            });
            return;
        }
        get<Channels>('/home')
            .then(channels => {
                const tvShows = channels[channel];
                if (tvShows != null && tvShows.length > 0) {
                    dispatch({
                        status: 'done',
                        tvShows,
                    });
                } else {
                    dispatch({
                        status: 'error',
                        tvShows: [],
                        error: `Invalid Channel: ${channel}`,
                    });
                }
            })
            .catch(e => dispatch({
                status: 'error',
                tvShows: [],
                error: e.toString(),
            }));
    }, [channel]);


    return (<>
        <header>
            <ul className="nav">
                <li className="home"><Link to="/">Home</Link></li>
                <li>{channel} ({state.tvShows.length})</li>
            </ul>
        </header>
        {state.status === 'loading' && <Loader />}
        {state.status === 'error' && <ErrMsg msg={state.error} />}
        {state.status === 'done' &&
            <main className="container">
                <section className="channelWrapper">
                    <div className="channelTvShows">
                        {state.tvShows.map(({ title, icon }) =>
                            <TvSoap
                                key={title}
                                channel={channel!!}
                                showTitle={title}
                                icon={icon}
                            />
                        )}
                    </div>
                </section>
            </main>}
    </>);
}
