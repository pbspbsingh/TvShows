use anyhow::anyhow;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use quick_js::{console, Context};
use reqwest::header;
use serde::Deserialize;
use tokio::time::Instant;
use tracing::*;

use crate::http_util::{find_host, http_client, normalize_url};

use super::find_iframe;

pub async fn find_m3u8(html: &str, referer: &str) -> anyhow::Result<(String, String)> {
    let start = Instant::now();
    let iframe_src = find_iframe(html, referer)?;
    debug!("Got iframe src: {iframe_src}");
    let m3u8_url = match tv_logy_v2(&iframe_src).await {
        Ok(m3u8_url) => {
            info!("Successfully resolved m3u8 url via tv_logy v2");
            m3u8_url
        }
        Err(e) => {
            warn!("Failed to resolve m3u8 url via v2 {e:?}");
            tv_logy_v1(&iframe_src, referer).await?
        }
    };
    info!("Time taken to resolve TVLogy: {:?}", start.elapsed());
    Ok((m3u8_url, iframe_src))
}

async fn tv_logy_v2(iframe_src: &str) -> anyhow::Result<String> {
    #[derive(Deserialize)]
    struct VideoSrc {
        #[serde(rename(deserialize = "videoSource"))]
        video_src: String,
    }

    let video_src = format!("{iframe_src}&do=getVideo");
    let json = http_client()
        .post(&video_src)
        .header(header::REFERER, iframe_src)
        .header("X-Requested-With", "XMLHttpRequest")
        .send()
        .await?
        .text()
        .await?;
    let video_src = serde_json::from_str::<VideoSrc>(&json)?;
    Ok(video_src.video_src)
}

async fn tv_logy_v1(iframe_src: &str, referer: &str) -> anyhow::Result<String> {
    let html = http_client()
        .get(iframe_src)
        .header(header::REFERER, find_host(referer)?)
        .send()
        .await?
        .text()
        .await?;
    let eval_src = find_eval(&html).ok_or_else(|| anyhow!("Couldn't find eval script"))?;
    let (m3u8_url, server, disk) = eval_script(eval_src)?;
    let url = normalize_url(&m3u8_url, iframe_src)?;
    Ok(format!("{url}?s={server}&d={disk}"))
}

fn eval_script(eval_script: &str) -> anyhow::Result<(String, String, String)> {
    let context = Context::builder().console(console::LogConsole).build()?;
    context.eval(PRELUDES)?;
    context.eval(eval_script)?;
    let video_url = context.eval_as::<String>("videoUrl")?;
    let server = context.eval_as::<String>("videoServer")?;
    let disk = context.eval_as::<String>("videoDisk")?;

    Ok((video_url, server, STANDARD.encode(disk)))
}

pub fn find_eval(html: &str) -> Option<&str> {
    html.find("eval(").map(|start| {
        let text = &html[start..];
        let mut stack = 0;
        let mut end = start;
        for (idx, ch) in text.char_indices() {
            stack += match ch {
                '(' => 1,
                ')' => -1,
                _ => continue,
            };
            if stack == 0 {
                end = idx;
                break;
            }
        }
        &text[..=end]
    })
}

const PRELUDES: &str = r"
    let videoUrl = '';
    let videoServer = 12;
    let videoDisk = '';
    
    const document = {};
    
    const FirePlayer = function(a, b, c) {
        videoUrl = b.videoUrl;
        videoServer = b.videoServer;
        videoDisk = b.videoDisk ? b.videoDisk : ''; 
    };
    
    const $ = function(arg) {
        if (typeof arg == 'function') {
            arg();
        } else {
            console.log('In $', arg);
        }
        return {
            ready: function(a) {
                if (typeof a == 'function') {
                    a();
                } else {
                    console.log('In $.ready', a);
                }
            }
        };
    };
";

#[cfg(test)]
mod test {
    const SCRIPT: &str = r###"
        eval(function(p,a,c,k,e,d){e=function(c){return(c<a?'':e(parseInt(c/a)))+((c=c%a)>35?String.fromCharCode(c+29):c.toString(36))};if(!''.replace(/^/,String)){while(c--){d[e(c)]=k[c]||e(c)}k=[function(e){return d[e]}];e=function(){return'\\w+'};c=1};while(c--){if(k[c]){p=p.replace(new RegExp('\\b'+e(c)+'\\b','g'),k[c])}}return p}('33 d="1x";i j(d){1w(d,{"1v":{"1":["1u.0","1t.0","1s.0","1r.0","1q.0","1p.0","1n.0"],"2":["1e.0","1m.0","1l.0","1k.0","1j.0","1i.0","1h.0"],"3":["1g.0","1f.0","1y.0","1o.0","1z.0","1L.0"],"4":["1U.0","1T.0","1S.0","1R.0","1Q.0","1P.0"],"5":["1O.a","1N.a","1M.a","1K.a","1B.a","1J.a"],"6":["1I.a","1H.a","1c.a","1F.a","1E.a","1D.a"],"7":["1C.a","1A.a","1d.a","17.a","1b.a","w.a"],"8":["E.0","r.0","F.0","G.0","D.0","C.0"],"9":["B.0","y.0","A.0","z.0","x.0","v.0"],"10":["u.0","t.0","s.0","I.0","U.0","1a.0"],"11":["19.0","18.0","H.0","Z.0","Y.0","X.0"],"12":["W.0","V.0","T.0","J.0","S.0","R.0"],"13":["Q.0","P.0","O.0","N.0","M.0"],"14":["L.0","K.0","1V.0","1G.0","1X.0"],"15":["31.0","1W.0","30.0","2Z.0","2Y.0"],"16":["2X.0","2W.0","2V.0","2U.0","2T.0","2R.0"]},"2I":"\\/m\\/c\\/l\\/k.p","2Q":"10","2P":"2O","2N":"g","2M":b,"2L":"2K+32\\/2S","34":"e:\\/\\/c.f.h\\/37\\/3e\\/g-8.13.7\\/g.35","39":{"n":"","38":"","36":"","3c":b},"3b":[],"3a":{"3d":"20","3f":"2J"},"2H":"e:\\/\\/c.f.h\\/o\\/2k.2F","2h":b,"2G":b,"2g":b,"2f":"2e 2d 2c 2b 2a 28 1Y 27 26-1","25":q,"24":b,"23":{"22":"21","1Z":"e:\\/\\/c.f.h\\/o\\/2i\\/29.2j","2v":"2E","2D":{"2C":"2B-2A-2z","2y":2x,"2w":2u}},"2l":{"2t":2s,"2r":[{"n":"e:\\/\\/10\\/m\\/c\\/l\\/k.p","2q":"2p","2o":"c"}]}},q)}$(i(){$(2n).2m(i(){j(d)})});',62,202,'xyz||||||||||club|true|hls|vhash|https|tvlogy|jwplayer|to|function|fireload|master|e0e2d4219396e5f966227bc79d04301b|cdn|file|ads|txt|false|stimulationbrand|polemanagement|potentialmanage|claimnight|collectpresent|wholeentertainment|comprehensivefilm|communicationskills|kitchenreactor|browneducation|marriagefit|admitrelative|lengthgrace|regulationoffice|sausagegreet|soldiersquash|commitmentunfair|thankslevel|convincejudgment|arenalast|advertiseroar|counterdesigner|bottomappeal|mirrorpreach|researchertechnique|bracketcompetence|impactstop|seriesdiscuss|villagefactor|mountainentry|coastswing|healthintegrity|agilegutter|layoutbundle|writerghost||||||||tiresequence|publicationslip|pepperbreast|loyaltytube|vanpatient|justiceracism|subwaytiptoe|amplealarm|favorlamb|safewheat|telljust|anxietypatient|tacticdance|advertisingenlarge|spareexcitement|migrationplagiarize|minoritycontinuous|royaltyrare|aviationintegration|architectureincredible|interventionoccupation|secretarydictionary|bangannouncement|quartermathematics|hostList|FirePlayer|dcd0985760d5621b9279ccaa313601cf|scramblejacket|classroomdrown|reserveoffense|coretrace|putbet|pressurejudicial|portraitladder|managementimprovement|caseembryo|foldaccident|episodeinstal|crosswinner|collectionenhance|mountainpersist|calmtemptation|businessfoster|acquaintanceecho|perfectaccountant|stretchdismissal|premiumrace|boldtrench|beheadmoon|memorialgraduate|brotheruncertainty|icelisten|conventionsay|October|tag||googima|client|advertising|rememberPosition|displaytitle|Pt|2021|21st|index24|Hai|Kehlata|Kya|Rishta|Yeh|title|jwplayer8quality|SubtitleManager|new|xml|tvl|videoData|ready|document|type|HD|label|videoSources|null|videoImage|300|vpaidmode|width|250|height|div|companion|sample|id|companiondiv|insecure|jpg|jwplayer8button1|defaultImage|videoUrl|Tahoma|FvaHWSQaGQ96mtkOAZ8NAA|jwPlayerKey|isJWPlayer8|videoPlayer|disk2|videoDisk|videoServer|feedapproval|0AfO3U6t8T22XvLxZsKspzMX5Ss8xBvgFg|skipview|fascinatetrade|shavehook|buildadmires|selfrelationships|tracefree|stingenergy|sheepviolation|delicatescreen|YSvvZ6|var|jwPlayerURL|js|position|player|link|logo|captions|tracks|hide|fontSize|assets|fontfamily'.split('|'),0,{}))
    "###;

    #[test]
    fn test_script() {
        let (url, server, disk) = super::eval_script(SCRIPT).unwrap();
        println!("URL: {url}");
        println!("?s={server}&d={disk}");
    }

    #[test]
    fn test_decode() {
        let arr = [
            0x59_u8, 0x7a, 0x68, 0x6b, 0x4d, 0x6d, 0x49, 0x7a, 0x4e, 0x54, 0x55, 0x33, 0x4f, 0x44,
            0x6c, 0x6d, 0x5a, 0x6a, 0x46, 0x6b, 0x4d, 0x32, 0x56, 0x6c, 0x5a, 0x6d, 0x55, 0x79,
            0x5a, 0x54, 0x49, 0x31, 0x4e, 0x6d, 0x49, 0x77, 0x5a, 0x47, 0x4a, 0x6c, 0x59, 0x6d,
            0x55, 0x3d,
        ];
        let res = String::from_utf8_lossy(&arr);
        println!("{res}");
    }
}
