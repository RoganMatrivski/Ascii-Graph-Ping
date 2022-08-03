use std::time::Duration;

use arraydeque::{ArrayDeque, Wrapping};
use r#struct::{AppConfig, PingResult};
use tokio::{
    sync::mpsc::{Receiver, Sender},
    time::Instant,
};
use tokio_stream::StreamExt;

use rasciigraph::plot;
use terminal_size::{terminal_size, Height, Width};

const ADDR_LIST: [&str; 4] = ["8.8.8.8", "8.8.4.4", "9.9.9.9", "149.112.112.112"];
const TIMESERIES_LENGTH: usize = 100;
const DELAY_TIME_MILLIS: u64 = 500;

mod func;
mod r#struct;

#[tokio::main]
async fn main() {
    let (Width(term_w), Height(term_h)) = terminal_size().expect("Cannot read terminal size");

    let (pinger_tx, pinger_rx) = tokio::sync::mpsc::channel(32);
    let (processor_tx, processor_rx) = tokio::sync::mpsc::channel(32);
    let (config_tx, config_rx) = tokio::sync::watch::channel(AppConfig {
        term_height: u32::from(term_h) - 3,
        term_width: u32::from(term_w) - 10,
    });

    tokio::spawn(async move { ping_processor(pinger_rx, processor_tx).await });
    tokio::spawn(async move { graph_writer(processor_rx, config_rx).await });
    tokio::spawn(async move { console_size_change_watcher(config_tx).await });

    let start_time = Instant::now();
    for (ip_id, addr) in ADDR_LIST.iter().enumerate() {
        let tx_clone = pinger_tx.clone();
        tokio::spawn(
            async move { pinger(addr.parse().unwrap(), ip_id, tx_clone, start_time).await },
        );
    }

    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

async fn console_size_change_watcher(tx: tokio::sync::watch::Sender<AppConfig>) {
    let (Width(term_w), Height(term_h)) = terminal_size().expect("Cannot read terminal size");
    let (mut local_term_w, mut local_term_h) = (u32::from(term_h) - 3, u32::from(term_w) - 10);

    loop {
        let (Width(term_w), Height(term_h)) = terminal_size().expect("Cannot read terminal size");
        let (curr_term_w, curr_term_h) = (u32::from(term_h) - 3, u32::from(term_w) - 10);

        if curr_term_h != local_term_h || curr_term_w != local_term_w {
            tx.send(AppConfig {
                term_height: u32::from(term_h) - 3,
                term_width: u32::from(term_w) - 10,
            })
            .expect("Failed to write config");
        }

        local_term_h = curr_term_h;
        local_term_w = curr_term_w;

        tokio::time::sleep(Duration::from_millis(200)).await;
    }
}

async fn graph_writer(
    mut rx: Receiver<Vec<u128>>,
    config_rx: tokio::sync::watch::Receiver<AppConfig>,
) {
    while let Some(arr) = rx.recv().await {
        let config = config_rx.borrow();
        let (term_w, term_h) = (config.term_width, config.term_height);

        let conv_arr = arr.iter().map(|&x| x as f64).collect();
        let processed_arr = func::moving_avg(conv_arr);
        let conf = rasciigraph::Config::default()
            .with_height(term_h)
            .with_width(term_w);
        let plot_res = plot(processed_arr, conf);

        print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
        println!("{}", plot_res);
    }
}

async fn ping_processor(mut rx: Receiver<r#struct::PingResult>, tx: Sender<Vec<u128>>) {
    // Structure:
    //  [index] - [index] - [index] - [index] - [index] - [index]
    //  [ rtt ]   [ rtt ]   [ rtt ]   [ rtt ]   [ rtt ]   [ rtt ]
    //  [ rtt ]             [ rtt ]   [ rtt ]   [ rtt ]   [ rtt ]
    //  [ rtt ]             [ rtt ]             [ rtt ]   [ rtt ]
    let mut ping_result: ArrayDeque<[r#struct::PingResultNoIP; TIMESERIES_LENGTH], Wrapping> =
        ArrayDeque::new();

    for _ in 0..TIMESERIES_LENGTH {
        ping_result.push_back(r#struct::PingResultNoIP {
            seq: 0,
            rtt_arr: vec![],
        });
    }

    while let Some(PingResult { ip_id: _, seq, rtt }) = rx.recv().await {
        if rtt.is_none() {
            continue;
        }

        let rtt = rtt.unwrap();

        if ping_result.is_empty() || seq > ping_result.back().unwrap().seq {
            // If it's a recent result
            ping_result.push_back(r#struct::PingResultNoIP {
                seq,
                rtt_arr: vec![rtt],
            });
        } else {
            let arr_position = match ping_result.iter().position(|x| x.seq == seq) {
                Some(pos) => pos, // Return array position
                None => continue, // Or continue if result timeindex doesn't exist
            };

            let ping_array = &mut ping_result[arr_position].rtt_arr;
            ping_array.push(rtt);
        }

        let final_mut: Vec<u128> = ping_result
            .iter()
            .map(|arr| {
                let arr = &arr.rtt_arr;
                let arr_len = u128::try_from(arr.len()).unwrap();

                if arr_len == 0 {
                    return 0;
                };

                arr.iter().map(|x| x.as_millis()).sum::<u128>() / arr_len
            })
            .collect();

        tx.send(final_mut).await.unwrap();
        // println!("{}: {:?}", ip_id, final_mut);
    }
}

async fn pinger(
    ip: std::net::IpAddr,
    ip_id: usize,
    tx: Sender<r#struct::PingResult>,
    start_time: Instant,
) {
    let pinger = tokio_icmp_echo::Pinger::new().await.unwrap();
    let mut stream = pinger.chain(ip).stream();
    let delay_time = Duration::from_millis(DELAY_TIME_MILLIS);
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
