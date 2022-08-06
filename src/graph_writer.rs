use crate::{consts::TIMESERIES_LENGTH, r#struct::AppConfig};
use tokio::sync::mpsc::Receiver;

use crossterm::{
    cursor, queue, style,
    terminal::{Clear, ClearType},
};
use std::io::{stdout, Write};

use rasciigraph::plot;

pub async fn run(mut rx: Receiver<Vec<u128>>, config_rx: tokio::sync::watch::Receiver<AppConfig>) {
    let mut stdout = stdout();

    crossterm::execute!(stdout, Clear(ClearType::All), cursor::Hide).unwrap();

    while let Some(arr) = rx.recv().await {
        let config = config_rx.borrow();
        let (term_w, term_h) = (config.term_width, config.term_height);

        let mut min: u128 = 0;
        let mut max: u128 = 0;
        let mut avg: f64 = 0.0;

        for value in &arr {
            if value < &min {
                min = *value
            };
            if value > &max {
                max = *value
            };
            if *value != 0 {
                avg += *value as f64 / TIMESERIES_LENGTH as f64
            };
        }

        let conv_arr = arr.iter().map(|&x| x as f64).collect();
        let processed_arr = crate::func::moving_avg(conv_arr);
        let conf = rasciigraph::Config::default()
            .with_height(term_h)
            .with_width(term_w);
        let plot_res = plot(processed_arr, conf);

        // print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
        // clearscreen::clear().expect("Somehow this program can't clear screen.");
        // crate::console_func::clear_screen();

        let stats = format!("AVG: {:.2} ms | MIN: {} ms | MAX: {} ms", avg, min, max);

        queue!(
            stdout,
            // Clear two line above for stats
            cursor::MoveTo(0, 1),
            Clear(ClearType::CurrentLine),
            cursor::MoveTo(0, 0),
            Clear(ClearType::CurrentLine),
            // Print stats
            style::Print(stats),
            // Print graph by replacing
            cursor::MoveTo(0, 2),
            style::Print(plot_res),
        )
        .unwrap();

        stdout.flush().unwrap();
        // println!("{}", plot_res);
    }
}
