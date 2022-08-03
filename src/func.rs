#![allow(dead_code)]

use yata::methods::WMA;
use yata::prelude::*;

pub fn moving_avg(arr: Vec<f64>) -> Vec<f64> {
    let mut wma = WMA::new(5, &arr[0]).unwrap();
    arr.iter().map(|e| wma.next(e)).collect()
}

pub fn get_samples_window_partial(arr: &Vec<f64>, window_size: i32, index: i32) -> Vec<f64> {
    let int_arr_len = i32::try_from(arr.len()).unwrap();
    let halved_window_size = window_size / 2; // Will inherently rounds down to lower floor

    let is_even = window_size % 2 == 0;

    // On even, offset to the right by 1
    let lower_bound: i32 = index - halved_window_size + if is_even { 1 } else { 0 };
    let upper_bound: i32 = index + halved_window_size;

    // Reassign all bounds but clampped
    let lower_bound = if lower_bound < 0 { 0 } else { lower_bound };
    let upper_bound = if upper_bound > int_arr_len - 1 {
        int_arr_len - 1
    } else {
        upper_bound
    };

    arr[(lower_bound as usize)..(upper_bound as usize)].to_vec()
}

pub fn get_samples_window(arr: &[f64], window_size: i32, index: i32) -> Vec<f64> {
    let halved_window_size = window_size / 2; // Will inherently rounds down to lower floor

    let is_even = window_size % 2 == 0;
    let lower_bound: i32 = index - halved_window_size + if is_even { 1 } else { 0 };

    // On Even window, add one offset
    let upper_bound: i32 = index + halved_window_size;

    let mut arr_res: Vec<f64> = vec![];

    if lower_bound < 0 {
        let diff = lower_bound.abs();
        arr_res.resize(diff.try_into().unwrap(), 0.0);
    }

    let final_lower_bound = if lower_bound < 0 { 0 } else { lower_bound };
    for el in final_lower_bound..upper_bound + 1 {
        match arr.get(el as usize) {
            Some(e) => arr_res.push(*e),
            None => arr_res.push(0.0),
        };
    }

    arr_res
}
