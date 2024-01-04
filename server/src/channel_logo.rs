use std::collections::HashMap;
use std::iter::{IntoIterator, Iterator};

use axum::body::Body;
use axum::extract::Path;
use axum::response::{IntoResponse, Redirect, Response};
use once_cell::sync::Lazy;
use tracing::*;

use crate::http_util::http_client;
use crate::tv_channels::NO_ICON;

static LOGO_MAP: Lazy<HashMap<&str, &str>> = Lazy::new(|| {
    [
        ("Star Plus", "https://static.wikia.nocookie.net/logopedia/images/3/32/StarPlus_logo_%282018%29.png/revision/latest/scale-to-width-down/200?cb=20201128160713"), 
        ("Colors", "https://static.wikia.nocookie.net/logopedia/images/f/fb/Colors_2016.svg/revision/latest/scale-to-width-down/250?cb=20210207143131"),
        ("Zee TV", "https://static.wikia.nocookie.net/logopedia/images/d/dd/Zee_TV_2017.svg/revision/latest/scale-to-width-down/200?cb=20191222192526"),
        ("Sony TV", "https://www.pngfind.com/pngs/m/50-505569_sony-tv-logo-png-channel-sony-entertainment-television.png"),
        ("& TV", "https://static.wikia.nocookie.net/logopedia/images/8/8c/%26TV.jpg/revision/latest/scale-to-width-down/220?cb=20161205163128"),
        ("Sab TV", "https://static.wikia.nocookie.net/logopedia/images/1/18/SONY_SAB_SD.png/revision/latest/scale-to-width-down/250?cb=20221023220045"),
        ("Star Bharat", "https://static.wikia.nocookie.net/logopedia/images/7/7b/Star_Bharat_2022.png/revision/latest/scale-to-width-down/250?cb=20220802114101"),
        ("ALT Balaji", "https://static.wikia.nocookie.net/logopedia/images/b/b9/Alt_Balaji.jpg/revision/latest/scale-to-width-down/250?cb=20200222162603"),
        ("Amazon", "https://www.yodesitv.info/wp-content/uploads/2020/04/amazonvideo-768x432.png"),
        ("Hotstar", "https://static.wikia.nocookie.net/logopedia/images/e/e9/Disney%2B_Hotstar.svg/revision/latest/scale-to-width-down/300?cb=20220114145230"),
        ("Netflix", "https://static.wikia.nocookie.net/logopedia/images/5/5d/Netflix_2014.svg/revision/latest/scale-to-width-down/250?cb=20201124111638"),
        ("Zee5", "https://static.wikia.nocookie.net/logopedia/images/d/d0/Zee5.svg/revision/latest?cb=20210807175347"),
        ("VOOT Web Series", "https://www.yodesitv.info/wp-content/uploads/2020/04/voot-370x208.jpg"),
        ("Hoichoi Web Series", "https://www.yodesitv.info/wp-content/uploads/2020/04/hoichoi-768x432.png"),
        ("MX Web Series", "https://www.yodesitv.info/wp-content/uploads/2020/04/mx-370x208.jpg"),
        ("Vikram Bhatt Web Series", "https://www.yodesitv.info/wp-content/uploads/2020/04/vikram-370x208.jpg"),
        ("Eros NOW Web Series", "https://www.yodesitv.info/wp-content/uploads/2020/05/erosi-370x208.jpg"),
    ].into_iter().collect()
});

pub async fn logo(Path(title): Path<String>) -> Response {
    let title = title.trim();
    let &logo_url = LOGO_MAP.get(title).unwrap_or(&NO_ICON);
    info!("Got logo {title} => {logo_url}");
    match _logo(logo_url).await {
        Ok(res) => res.into_response(),
        Err(e) => {
            warn!("Error while fetching {logo_url}: {e:?}");
            Redirect::temporary(NO_ICON).into_response()
        }
    }
}

async fn _logo(logo_url: &str) -> anyhow::Result<Response<Body>> {
    let mut logo_res = http_client().get(logo_url).send().await?;
    let mut response = Response::builder().status(logo_res.status());
    for (key, value) in logo_res.headers() {
        response = response.header(key, value);
    }
    let (mut sender, receiver) = Body::channel();
    tokio::spawn(async move {
        while let Ok(Some(chunk)) = logo_res.chunk().await {
            if sender.send_data(chunk).await.is_err() {
                break;
            }
        }
    });
    Ok(response.body(receiver)?)
}
