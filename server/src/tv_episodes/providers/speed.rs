use anyhow::anyhow;
use quick_js::{console, Context};
use reqwest::header;
use tokio::time::Instant;
use tracing::*;

use crate::http_util::{find_host, http_client, normalize_url};
use crate::tv_episodes::providers::tv_logy::find_eval;

use super::find_iframe;

pub async fn find_mp4(html: &str, referer: &str) -> anyhow::Result<(String, String)> {
    let start = Instant::now();
    let iframe_src = find_iframe(html, referer)?;
    debug!("Got iframe src: {iframe_src}");
    let html = http_client()
        .get(&iframe_src)
        .header(header::REFERER, find_host(referer)?)
        .send()
        .await?
        .text()
        .await?;
    let eval_src = find_eval(&html).ok_or_else(|| anyhow!("Couldn't find eval script"))?;
    let video_src = eval_script(eval_src)?;
    info!(
        "Time taken to resolve Speed: {}",
        start.elapsed().as_millis()
    );
    Ok((
        normalize_url(&video_src, &iframe_src)?.into_owned(),
        iframe_src,
    ))
}

fn eval_script(eval_script: &str) -> anyhow::Result<String> {
    let context = Context::builder().console(console::LogConsole).build()?;
    context.eval(PRELUDE)?;
    context.eval(eval_script)?;
    Ok(context.eval_as("source")?)
}

const PRELUDE: &str = r"
    let source = null;
    
    function jwplayer() {
        return {
            setup: function(config) {
                const arr = config.sources.sort((a, b) => parseInt(b.label) - parseInt(a.label));
                console.log(arr);
                source = arr[0].file;
    
                return {
                    addButton: () => console.log('called setup.addButton'),
                    seek: () => console.log('called setup.seek'),
                };
            },
            on: () => console.log('called jwplayer.on'),
        };
    }
";

#[cfg(test)]
mod test {
    use super::eval_script;

    const SCRIPT: &str = r##"eval(function(p,a,c,k,e,d){while(c--)if(k[c])p=p.replace(new RegExp('\\b'+c.toString(a)+'\\b','g'),k[c]);return p}('9.4i={"1m":{"1l":{}}};1c 8=9("4h").4g({4f:[{13:"4://15.14.6/4e/v.1y",1x:"4d"},{13:"4://15.14.6/4c/v.1y",1x:"4b"}],4a:"4://15.14.6/i/49/48/17.47",46:\'./z/\',"45":{"13":"","44":"","18":"11","d":"43-42"},1q:"1w%",1p:"1w%",41:"40",3z:"3y.28",3x:{3w:{"3v":"#1v","3u":"#3t"},3s:{"3r":"#1v","3q":"3p(12,12,12,0.3)"}},3o:{3n:\'p\',3m:{a:{e:"a",r:10,b:\'\'},3l:{e:"3k",r:10,b:[\'4://q.o-x.6/n-m-l-k-j/a-h/\',\'4://p.1u.1t/?1s=1r\',\'\',\'\']},3j:{e:"3i",r:10,b:[\'4://q.o-x.6/n-m-l-k-j/a-h/\',\'4://p.1u.1t/?1s=1r\',\'\',\'\',\'\']},3h:{e:"3g",r:10,b:[\'\',\'4://q.o-x.6/n-m-l-k-j/a-h/\',\'4://v.3f.6/b/p?3e=3d&3c=s.6&1q=1o-3b&1p=1o-3a&39=38-37\',\'\']},1n:{e:"1n",b:[\'\',\'4://q.o-x.6/n-m-l-k-j/a-h/\']}}},36:{},35:\'34\',33:"11",32:"11",31:"30",2z:"2y",1m:{"1l":{}},2x:[],2w:"",2v:"4://s.6"});c(2u==\'2t\'){}2s{8.1j("./z/1i/1k.1h","2r 1g",7(){8.1f(8.1e()+10)},"1k");8.1j("./z/1i/1d.1h","2q 1g",7(){8.1f(8.1e()-10)},"1d")}1c t,y,w=0;9().f(\'2p\',7(x){c(5>0&&x.d>=5&&y!=1){y=1;$(\'u.2o\').2n(\'2m\')}c(w==0&&x.d>=1b&&x.d<=(1b+2)){w=x.d}});9().f(\'2l\',7(x){1a(x)});9().f(\'2k\',7(){$(\'u.19\').2j()});7 1a(x){$(\'u.19\').18();c(t)2i;t=1;g=0;c(2h.2g===2f){g=1}$.2e(\'4://s.6/2d?2c=2b&2a=17&29=27-26-25-24-23&22=1&g=\'+g,7(16){$(\'#21\').20(16)})}9().f(\'1z\',7(){});',36,163,'||||https||com|function|player8|jwplayer|pre|tag|if|position|offset|on|adb|roll||1c35782c6341|9e82|4fb3|dcaf|fe3f21e4|reyden|vast||skipoffset|vkspeed|vvplay|div||x2ok||vvad|player8177||true|255|file|vkcdn5|hetremove|data|q2d6bwoycfmp|hide|video_ad|doPlay|188|var|backward|getPosition|seek|10s|svg|skins|addButton|forward1|ping|plugins|post|__player|height|width|11627|tcid|xyz|yomeno|3298da|100|label|mp4|ready|html|fviews|embed|49e254942d41f88f848744903868ec2b|1647128682|254|104|1527648||hash|file_code|view|op|dl|get|undefined|cRAds|window|return|show|complete|play|slow|fadeIn|video_ad_fadein|time|Backward|Forward|else|1200|678|aboutlink|abouttext|tracks|start|startparam|html5|primary|hlshtml|androidhls|none|preload|cast|number__|__random|cb|height__|width__|page_url|11510|pzoneid|adtrue|720|thirdmid|420|secmid|120|firstmid|schedule|client|advertising|rgba|rail|progress|timeslider|29b765|iconsActive|icons|controlbar|skin|629|duration|uniform|stretching|bar|control|link|logo|base|jpg|00305|01|image|192p|olaxkjugr3uiolyobgx2bqlpmzmkjkq3yfkudworii66jl7tvhvhwuk73rrq|360p|olaxkjugr3uiolyobgx2bqlpmzmkjkq3yfkudworiaqoll7tvhvgyvzb2ipa|sources|setup|vplayer|defaults'.split('|')))"##;

    #[test]
    fn test_eval() {
        println!("{}", eval_script(SCRIPT).unwrap());
    }
}
