use crate::r#struct::AppConfig;
use terminal_size::{terminal_size, Height, Width};

pub fn get_console_size() -> (u32, u32) {
    let (Width(term_w), Height(term_h)) = terminal_size().expect("Cannot read terminal size");

    (u32::from(term_h), u32::from(term_w))
}

pub async fn console_size_change_watcher(tx: tokio::sync::watch::Sender<AppConfig>) {
    let (term_h, term_w) = crate::console_func::get_console_size();
    let (mut local_term_w, mut local_term_h) = (
        term_h - crate::consts::BOTTOM_MARGIN,
        term_w - crate::consts::RIGHT_MARGIN,
    );

    loop {
        let (term_h, term_w) = crate::console_func::get_console_size();
        let (curr_term_w, curr_term_h) = (
            term_h - crate::consts::BOTTOM_MARGIN,
            term_w - crate::consts::RIGHT_MARGIN,
        );

        if curr_term_h != local_term_h || curr_term_w != local_term_w {
            tx.send(AppConfig {
                term_height: term_h - crate::consts::BOTTOM_MARGIN,
                term_width: term_w - crate::consts::RIGHT_MARGIN,
            })
            .expect("Failed to write config");
        }

        local_term_h = curr_term_h;
        local_term_w = curr_term_w;

        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    }
}
