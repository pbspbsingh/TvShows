import React, { useEffect, useState } from 'react';
import { Link } from 'react-router-dom';
import { LazyLoadImage } from 'react-lazy-load-image-component';

import { ErrMsg, Loader } from './common';
import { get } from './util';

export type TvShow = {
    title: string;
    icon: string;
};

export type Channels = {
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
                        <h3 className="channelTitle">
                            <Link to={`/channel/${title}`}>
                                {title} ({tvShows.length})
                            </Link>
                        </h3>
                        <article className="tvShows">
                            {tvShows.map(({ title: showTitle, icon }) =>
                                <TvSoap
                                    key={`${title}/${showTitle}`}
                                    channel={title}
                                    showTitle={showTitle}
                                    icon={icon}
                                />
                            )}
                        </article>
                    </section>)}
            </main>}
        </>
    );
}

export const TvSoap = ({ channel, showTitle, icon }: { channel: string, showTitle: string, icon: string }) => (
    <div key={showTitle} className="tvShow">
        <Link to={`/tvshow/${channel}/${showTitle}`}>
            <LazyLoadImage src={icon} alt={channel} loading="lazy" />
        </Link>
        <Link to={`/tvshow/${channel}/${showTitle}`}>
            <h3>{showTitle}</h3>
        </Link>
    </div>
);