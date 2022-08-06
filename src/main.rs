use r#struct::AppConfig;

mod console_func;
mod consts;
mod func;
mod graph_writer;
mod ping_processor;
mod pinger;
mod r#struct;

#[tokio::main]
async fn main() {
    let (term_h, term_w) = crate::console_func::get_console_size();

    let (pinger_tx, pinger_rx) = tokio::sync::mpsc::channel(32);
    let (processor_tx, processor_rx) = tokio::sync::mpsc::channel(32);
    let (config_tx, config_rx) = tokio::sync::watch::channel(AppConfig {
        term_height: term_h - crate::consts::BOTTOM_MARGIN,
        term_width: term_w - crate::consts::RIGHT_MARGIN,
    });

    tokio::spawn(async move { crate::ping_processor::run(pinger_rx, processor_tx).await });
    tokio::spawn(async move { crate::graph_writer::run(processor_rx, config_rx).await });
    tokio::spawn(async move { crate::console_func::console_size_change_watcher(config_tx).await });

    let start_time = tokio::time::Instant::now();
    for (ip_id, addr) in consts::ADDR_LIST.iter().enumerate() {
        let tx_clone = pinger_tx.clone();
        tokio::spawn(async move {
            pinger::pinger(addr.parse().unwrap(), ip_id, tx_clone, start_time).await
        });
    }

    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}
