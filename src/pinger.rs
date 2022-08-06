use crate::{consts, r#struct};

use tokio::{sync::mpsc::Sender, time::Instant};

use std::time::Duration;

#[cfg(target_os = "linux")]
use tokio_stream::StreamExt;

#[cfg(target_os = "windows")]
pub async fn pinger(
    ip: std::net::IpAddr,
    ip_id: usize,
    tx: Sender<r#struct::PingResult>,
    start_time: Instant,
) {
    let pinger = winping::AsyncPinger::new();
    let delay_time = Duration::from_millis(consts::DELAY_TIME_MILLIS);
    let mut next_wait = start_time + delay_time;

    let mut seq: usize = 0;

    loop {
        let buf = winping::Buffer::new();
        let probably_rtt = pinger.send(ip, buf).await.result;
        tx.send(r#struct::PingResult {
            ip_id,
            seq,
            rtt: match probably_rtt {
                Ok(rtt) => Some(Duration::from_millis(rtt.into())),
                Err(err) => {
                    println!("{:?}", err);
                    continue;
                }
            },
        })
        .await
        .unwrap();

        tokio::time::sleep_until(next_wait).await;
        next_wait += delay_time;
        seq += 1;
    }
}

#[cfg(target_os = "linux")]
pub async fn pinger(
    ip: std::net::IpAddr,
    ip_id: usize,
    tx: Sender<r#struct::PingResult>,
    start_time: Instant,
) {
    let pinger = tokio_icmp_echo::Pinger::new().await.unwrap();
    let mut stream = pinger.chain(ip).stream();
    let delay_time = Duration::from_millis(consts::DELAY_TIME_MILLIS);
    let mut next_wait = start_time + delay_time;

    let mut seq: usize = 0;

    while let Some(probably_rtt) = stream.next().await {
        tx.send(r#struct::PingResult {
            ip_id,
            seq,
            rtt: match probably_rtt {
                Ok(rtt) => rtt,
                Err(err) => {
                    println!("{:?}", err);
                    continue;
                }
            },
        })
        .await
        .unwrap();

        tokio::time::sleep_until(next_wait).await;
        next_wait += delay_time;
        seq += 1;
    }
}
