use crate::{consts, r#struct};

use arraydeque::{ArrayDeque, Wrapping};
use tokio::sync::mpsc::{Receiver, Sender};

pub async fn run(mut rx: Receiver<r#struct::PingResult>, tx: Sender<Vec<u128>>) {
    // Structure:
    //  [index] - [index] - [index] - [index] - [index] - [index]
    //  [ rtt ]   [ rtt ]   [ rtt ]   [ rtt ]   [ rtt ]   [ rtt ]
    //  [ rtt ]             [ rtt ]   [ rtt ]   [ rtt ]   [ rtt ]
    //  [ rtt ]             [ rtt ]             [ rtt ]   [ rtt ]
    let mut ping_result: ArrayDeque<
        [r#struct::PingResultNoIP; consts::TIMESERIES_LENGTH],
        Wrapping,
    > = ArrayDeque::new();

    for _ in 0..consts::TIMESERIES_LENGTH {
        ping_result.push_back(r#struct::PingResultNoIP {
            seq: 0,
            rtt_arr: vec![],
        });
    }

    while let Some(r#struct::PingResult { ip_id: _, seq, rtt }) = rx.recv().await {
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
