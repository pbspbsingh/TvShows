import React, { useEffect, useState } from 'react';
import { Link } from 'react-router-dom';

import { ErrMsg, Loader } from './common';
import { get } from './util';

type TvShow = {
    title: string;
    icon: string;
};

type Channels = {
    [name: string]: Array<TvShow>,
};

type State = {
    status: 'error' | 'loading' | 'done';
    channels: Channels;
    error?: string;
};

export default function Home(): JSX.Element {
    const [state, dispatch] = useState<State>({
        status: 'loading',
        channels: {},
    });

    useEffect(() => {
        get<Channels>('/home')
            .then(channels => dispatch({
                status: 'done',
                channels,
            }))
            .catch(e => dispatch({
                status: 'error',
                channels: {},
                error: e.toString(),
            }));
    }, []);

    return (
        <>
            {state.status === 'loading' && <Loader />}
            {state.status === 'error' && <ErrMsg msg={state.error} />}
            {state.status === 'done' && <main className="container">
                {Object.entries(state.channels).map(([title, tvShows]) =>
                    <section key={title} className="channel" id={title}>
                        <h3 className="channelTitle">{title} ({tvShows.length})</h3>
                        <article className="tvShows">
                            {tvShows.map(({ title: showTitle, icon }) =>
                                <div key={showTitle} className="tvShow">
                                    <Link to={`/tvshow/${title}/${showTitle}`}>
                                        <img src={icon} alt={title} loading="lazy" />
                                    </Link>
                                    <Link to={`/tvshow/${title}/${showTitle}`}>
                                        <h3>{showTitle}</h3>
                                    </Link>
                                </div>)}
                        </article>
                    </section>)}
            </main>}
        </>
    );
}
