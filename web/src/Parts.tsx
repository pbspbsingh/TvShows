import React, { useEffect, useReducer, useRef } from 'react';
import { Link, NavigateFunction, useNavigate, useParams } from 'react-router-dom';

import videojs from 'video.js';
import 'video.js/dist/video-js.css';

import { ErrMsg, Loader } from './common';
import { get } from './util';


type Action = {
    name: 'ERROR';
    msg: string;
} | {
    name: 'LOADED';
    parts: Array<Part>;
} | {
    name: 'PLAY';
    index: number;
};

type Part = {
    title: string;
    url: string;
};

type State = {
    status: 'loading' | 'error' | 'done';
    parts: Part[];
    curIdx: number;
    error?: string;
};

function reduce(state: State, action: Action): State {
    switch (action.name) {
        case 'ERROR': return {
            status: 'error',
            error: action.msg,
            parts: [],
            curIdx: -1,
        };
        case 'LOADED': return {
            status: 'done',
            parts: action.parts,
            curIdx: 0,
        };
        case 'PLAY': return {
            ...state,
            curIdx: action.index,
        }
    }
    throw new Error(`Unexpected action ${action}`);
}

export default function Parts(): JSX.Element {
    const { channel, tv_show, episode } = useParams();
    const navigate = useNavigate();

    const [state, dispatch] = useReducer(reduce, {
        status: 'loading',
        parts: [],
        curIdx: -1,
    });

    const videoRef = useRef(null);
    const videoPlayer = useRef<videojs.Player | null>(null);

    useEffect(() => {
        get<string[][]>(`/episode/${channel}/${tv_show}/${episode}`)
            .then(res => dispatch({
                name: 'LOADED',
                parts: res.map(([title, url]) => ({ title, url })),
            }))
            .catch(e => dispatch({ name: 'ERROR', msg: e.toString() }));
    }, [channel, tv_show, episode]);

    useEffect(() => {
        const { current: videoEl } = videoRef;
        if (videoEl == null || state.curIdx < 0) {
            return;
        }
        if (videoPlayer.current == null) {
            videoPlayer.current = videojs(videoEl, {
                autoplay: true,
                fluid: true,
                sources: [{
                    src: state.parts[state.curIdx].url
                }]
            });
        } else {
            videoPlayer.current.autoplay(true);
            videoPlayer.current.src([{
                src: state.parts[state.curIdx].url
            }]);
        }
        return () => {
            if (videoPlayer.current != null) {
                videoPlayer.current.pause();
            }
        };
    }, [state, videoRef])

    useEffect(() => {
        return () => {
            if (videoPlayer.current != null) {
                console.debug('Disposing video player');
                videoPlayer.current.dispose();
            }
        }
    }, [videoPlayer]);

    return (
        <>
            <header>
                <ul className="nav">
                    <li className="home"><Link to="/">Home</Link></li>
                    <li><Link to={`/channel/${channel}`}>{channel}</Link></li>
                    <li><Link to={`/tvshow/${channel}/${tv_show}`}>{tv_show}</Link></li>
                    <li>{episode}</li>
                </ul>
            </header>
            <main className="container">
                {state.status === 'loading' && <Loader />}
                {state.status === 'error' && <ErrMsg msg={state.error} />}
                {state.status === 'done' && <div className="partsWrapper">
                    <div className="partList">
                        <ul className="parts">
                            {state.parts.map(({ title, url }, index) =>
                                <li key={title}>
                                    <a href={`#${title}`}
                                        className={state.curIdx === index ? 'active' : ''}
                                        onClick={(e) => {
                                            e.preventDefault();
                                            dispatch({ name: 'PLAY', index });
                                        }}>
                                        {title}
                                    </a>
                                </li>)}
                        </ul>
                    </div>
                    <div className="videoPlayer">
                        <video ref={videoRef}
                            className="video-js vjs-big-play-centered"
                            autoPlay={true}
                            controls={true}
                            preload="auto"
                            onEnded={() => {
                                if (state.curIdx < state.parts.length - 1) {
                                    dispatch({ name: 'PLAY', index: state.curIdx + 1 });
                                } else {
                                    loadNextEpisode(navigate, { channel, tv_show, episode });
                                }
                            }} />
                    </div>
                </div>}
            </main>
        </>
    );
}

type TvSoap = {
    channel?: string;
    tv_show?: string;
    episode?: string;
};

async function loadNextEpisode(navigate: NavigateFunction, { channel, tv_show, episode }: TvSoap) {
    console.log(`Done playing all parts of this episode[${episode}], trying to play next..`);
    if (channel == null || tv_show == null || episode == null) {
        console.warn('One of the param is null', channel, tv_show, episode);
        return;
    }

    try {
        const { episodes } = await get<{ episodes: string[] }>(`/episodes/${channel}/${tv_show}`);
        let next: number | null = null;
        for (let i = episodes.length - 1; i > 0; i--) {
            if (episodes[i] === episode) {
                next = i - 1;
                break;
            }
        }
        if (next != null) {
            console.log('Playing next episode:', episodes[next]);
            navigate(`/parts/${channel}/${tv_show}/${episodes[next]}`);
        } else {
            console.log('This is the last part, can\'t play next');
        }
    } catch (e) {
        console.error(`Something went wrong while loading episodes for ${channel}\${tv_show}`, e);
    }
}
