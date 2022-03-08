use std::time::Instant;

use mimalloc::MiMalloc;
use quick_js::{console, Context};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn main() -> anyhow::Result<()> {
    let start = Instant::now();
    let context = Context::builder().console(console::LogConsole).build()?;
    context.eval(PRELUDES)?;
    context.eval(SCRIPT)?;
    let video_url = context.eval_as::<String>("videoUrl")?;
    println!(
        "Found videoUrl: {} in {}ms",
        video_url,
        start.elapsed().as_millis()
    );
    Ok(())
}

const PRELUDES: &str = r"
    let videoUrl = '';
    const document = {};
    const FirePlayer = function(a, b, c) {
        videoUrl = b.videoUrl;
    };
    const $ = function(arg) {
        this.ready = function(a) {
            if (typeof a == 'function') {
                a();
            } else {
                console.log(a);
            }
        };
    
        if (typeof arg == 'function') {
            arg();
        } else {
            console.log(arg);
        }
        return this;	
    };
";

const SCRIPT: &str = r#"
    eval(function(p,a,c,k,e,d){e=function(c){return(c<a?'':e(parseInt(c/a)))+((c=c%a)>35?String.fromCharCode(c+29):c.toString(36))};if(!''.replace(/^/,String)){while(c--){d[e(c)]=k[c]||e(c)}k=[function(e){return d[e]}];e=function(){return'\\w+'};c=1};while(c--){if(k[c]){p=p.replace(new RegExp('\\b'+e(c)+'\\b','g'),k[c])}}return p}('36 d="1v";i k(d){1u(d,{"1t":{"1":["1s.0","1r.0","1q.0","1p.0","1o.0","1m.0","1d.0"],"2":["1l.0","1k.0","1j.0","1i.0","1h.0","1g.0","1f.0"],"3":["1e.0","1w.0","1n.0","1x.0","1J.0","1R.0"],"4":["1Q.0","1P.0","1O.0","1N.0","1M.0","1L.0"],"5":["1K.a","1I.a","1z.a","1H.a","1G.a","1b.a"],"6":["1E.a","1D.a","1C.a","1B.a","1A.a","1y.a"],"7":["1c.a","Y.a","1a.a","w.a","E.a","x.a"],"8":["F.0","G.0","D.0","C.0","B.0","A.0"],"9":["z.0","y.0","v.0","u.0","t.0","s.0"],"10":["I.0","T.0","19.0","18.0","17.0","Z.0"],"11":["H.0","X.0","W.0","V.0","U.0","S.0"],"12":["J.0","R.0","Q.0","P.0","O.0","N.0"],"13":["M.0","L.0","K.0","1S.0","1F.0"],"14":["1U.0","2V.0","2U.0","2T.0","1T.0"],"15":["2S.0","2R.0","2Q.0","2P.0","2O.0"],"16":["2M.0","2E.0","2L.0","2K.0","2J.0","2I.0"]},"2H":"\\/n\\/c\\/m\\/l.r","2G":"1","2F":q,"2X":"g","2W":b,"2N":"2Y+39\\/3a","38":"e:\\/\\/c.f.h\\/37\\/2Z\\/g-8.13.7\\/g.35","34":{"o":"","33":"","32":"","31":b},"30":[],"2D":{"2h":"20","2B":"2C"},"2e":"e:\\/\\/c.f.h\\/p\\/2d.2c","2b":b,"2a":b,"29":b,"28":"27 25 1V 24 23-1","22":j,"21":b,"1Z":{"1Y":"1X","1W":"e:\\/\\/c.f.h\\/p\\/2f\\/26.2g","2s":"2A","2z":{"2y":"2x-2w-2v","2u":2t,"2r":2i}},"2q":{"2p":q,"2o":[{"o":"e:\\/\\/1\\/n\\/c\\/m\\/l.r","2n":"2m","2l":"c"}]}},j)}$(i(){$(2k).2j(i(){k(d)})});',62,197,'xyz||||||||||club|true|hls|vhash|https|tvlogy|jwplayer|to|function|false|fireload|master|a03435cd00e325e24d10d4968a7c59e5|cdn|file|ads|null|txt|collectpresent|comprehensivefilm|kitchenreactor|browneducation|tiresequence|wholeentertainment|communicationskills|marriagefit|admitrelative|lengthgrace|soldiersquash|sausagegreet|vanpatient|regulationoffice|stimulationbrand|pepperbreast|claimnight|healthintegrity|mirrorpreach|researchertechnique|bracketcompetence|impactstop|seriesdiscuss|convincejudgment|villagefactor|coastswing|agilegutter|potentialmanage|layoutbundle|writerghost|commitmentunfair|publicationslip|reserveoffense|loyaltytube||||||||mountainentry|thankslevel|polemanagement|subwaytiptoe|crosswinner|putbet|minoritycontinuous|safewheat|telljust|anxietypatient|tacticdance|advertisingenlarge|spareexcitement|migrationplagiarize|amplealarm|aviationintegration|scramblejacket|architectureincredible|interventionoccupation|secretarydictionary|bangannouncement|quartermathematics|hostList|FirePlayer|11fdda320001f8432cb19623193ec2f9|favorlamb|royaltyrare|pressurejudicial|calmtemptation|portraitladder|managementimprovement|justiceracism|foldaccident|episodeinstal|counterdesigner|coretrace|collectionenhance|businessfoster|classroomdrown|acquaintanceecho|perfectaccountant|stretchdismissal|premiumrace|boldtrench|beheadmoon|memorialgraduate|mountainpersist|bottomappeal|conventionsay|advertiseroar|March|tag|googima|client|advertising||rememberPosition|displaytitle|Pt|2022|1st|index23|Meet|title|jwplayer8quality|jwplayer8button1|SubtitleManager|jpg|tvl|defaultImage|new|xml|fontSize|300|ready|document|type|HD|label|videoSources|videoImage|videoData|width|vpaidmode|250|height|div|companion|sample|id|companiondiv|insecure|fontfamily|Tahoma|captions|buildadmires|videoDisk|videoServer|videoUrl|feedapproval|skipview|fascinatetrade|shavehook|selfrelationships|jwPlayerKey|tracefree|stingenergy|sheepviolation|icelisten|delicatescreen|caseembryo|brotheruncertainty|arenalast|isJWPlayer8|videoPlayer|FvaHWSQaGQ96mtkOAZ8NAA|assets|tracks|hide|position|link|logo|js|var|player|jwPlayerURL|YSvvZ6|0AfO3U6t8T22XvLxZsKspzMX5Ss8xBvgFg'.split('|'),0,{}));
"#;
