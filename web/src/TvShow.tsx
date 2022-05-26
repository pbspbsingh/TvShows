import React, { useEffect, useReducer } from 'react';
import { Link, useParams } from 'react-router-dom';

import { ErrMsg, Loader } from './common';
import { get } from './util';

type State = {
    status: 'loading' | 'error' | 'done';
    error?: string;
    episodes: Array<string>;
    hasMore: boolean;
    loadingMore: boolean;
};

type Action = {
    name: 'ERROR';
    error: string;
} | {
    name: 'LOADED';
    episodes: string[];
    hasMore: boolean;
} | {
    name: 'LOADING_MORE';
};

function reduce(state: State, action: Action): State {
    switch (action.name) {
        case 'ERROR': return {
            ...state,
            status: 'error',
            error: action.error
        };
        case 'LOADED': return {
            status: 'done',
            episodes: action.episodes,
            hasMore: action.hasMore,
            loadingMore: false,
        };
        case 'LOADING_MORE': return {
            ...state,
            loadingMore: true,
        };
    }
    throw new Error(`Unexpected action ${action}`);
}

export default function TvShow(): JSX.Element {
    const { channel, tv_show } = useParams();

    const [state, dispatch] = useReducer(reduce, {
        status: 'loading',
        episodes: [],
        hasMore: false,
        loadingMore: false,
    });

    useEffect(() => loadEpisodes(dispatch, channel, tv_show), [channel, tv_show]);

    return (
        <>
            <header>
                <ul className="nav">
                    <li className="home"><Link to="/">Home</Link></li>
                    <li><Link to={`/channel/${channel}`}>{channel}</Link></li>
                    <li>{tv_show}</li>
                </ul>
            </header>
            <main className="container">
                {state.status === 'loading' && <Loader />}
                {state.status === 'error' && <ErrMsg msg={state.error} />}
                {state.status === 'done' && <div className="episodeWrapper">
                    <ul className="episodes">
                        {state.episodes.map((episode, idx) =>
                            <li key={`${episode}_ ${idx}`}>
                                {idx + 1} &nbsp;
                                <Link to={`/parts/${channel}/${tv_show}/${episode}`}>
                                    {episode}
                                </Link>
                            </li>
                        )}
                    </ul>

                    {state.hasMore && !state.loadingMore && <div>
                        <a href="#loadMore"
                            className="loadMore"
                            onClick={(e) => {
                                e.preventDefault();
                                dispatch({ name: 'LOADING_MORE' });
                                loadEpisodes(dispatch, channel, tv_show, true);
                            }}>
                            Load More
                        </a>
                    </div>}
                    {state.loadingMore && <img src="/loader.svg" className="loadingMore" alt="" />}
                </div>}
            </main>
        </>
    );
}

function loadEpisodes(dispatch: React.Dispatch<Action>,
    channel?: string,
    tv_show?: string,
    loadMore: boolean = false) {
    if (channel == null || tv_show == null) {
        dispatch({ name: 'ERROR', error: `Either ${channel} or ${tv_show} is undefind.` });
        return;
    }

    get<{ episodes: string[]; has_more: boolean; }>(
        `/episodes/${channel}/${tv_show}`,
        { "load_more": String(loadMore) }
    )
        .then(res => dispatch({
            name: 'LOADED',
            episodes: res.episodes,
            hasMore: res.has_more,
        }))
        .catch(err => dispatch({ name: 'ERROR', error: err.toString(), }));
}
