mod bstat;
mod exciter;
mod ftbb;
use std::*;

const PULSE_WIDTH: time::Duration = time::Duration::from_millis(30);

fn dot<const N: usize>(xs: &[f64; N], ys: &[f64; N]) -> f64 {
    xs.iter().zip(ys).map(|(x, y)| x * y).sum()
}

fn run() -> Result<(), Box<dyn error::Error>> {
    let ftbb = ftbb::FtBitBang::new(b"FT245R USB FIFO\0")?;
    let exciter = exciter::BitExciter::new(Box::new(move |bits| ftbb.set_bits(bits).unwrap()));
    let mut bstat = bstat::BeatSaberStatus::new()?;

    loop {
        let ev = bstat.wait_note_cut()?;

        // XXX
        let Some(score) = ev.cut_distance_score else {
            continue;
        };
        let parity = match ev.note_cut_direction.as_str() {
            "Up" | "UpLeft" | "UpRight" => -1.0,
            "Down" | "DownLeft" | "DownRight" => 1.0,
            _ => continue,
        };
        let side = parity * dot(&ev.cut_point, &ev.cut_normal);

        let bit_side = if side < 0.0 { 0 } else { 1 };
        let bit_hand = match ev.saber_type.as_ref().unwrap().as_str() {
            "SaberA" => 0,
            "SaberB" => 2,
            _ => return Err("".into()),
        };
        exciter.excite(bit_hand | bit_side, PULSE_WIDTH * (15 - score as u32));
    }
}

fn main() -> Result<(), Box<dyn error::Error>> {
    loop {
        if let Err(err) = run() {
            eprintln!("{}", err);
        }
        thread::sleep(time::Duration::from_secs(1));
    }
}
