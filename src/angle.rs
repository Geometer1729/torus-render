mod proj;
mod trig;

use proj::angle_to_r3;
use trig::{atan2, cos, sin};
use std::f64::consts::TAU;
use vecmath::{vec3_dot, vec3_normalized, vec3_sub};

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author = "Brian Kuhns", version, about)]
struct Args {
    #[arg(long = "x1")]
    x1: f64,
    #[arg(long = "y1")]
    y1: f64,
    #[arg(long = "x2")]
    x2: f64,
    #[arg(long = "y2")]
    y2: f64,

}

const CONV : f64 = TAU/360.0;

fn main() {
    let args = Args::parse();
    let x1 = args.x1 * CONV;
    let y1 = args.y1 * CONV;
    let x2 = args.x2 * CONV;
    let y2 = args.y2 * CONV;

    let t1 = [sin(x1), -1.0 * cos(x1), 0.0];
    let t2 = vec3_normalized([
        sin(y1) * cos(x1) * -1.0,
        sin(y1) * sin(x1) * -1.0,
        cos(y1),
    ]);

    let delta = vec3_sub(angle_to_r3([x2,y2]),angle_to_r3([x1,y1]));
    let d1 = vec3_dot(delta,t1);
    let d2 = vec3_dot(delta,t2);
    let theta = atan2(d2,d1);
    println!("{}",theta/CONV);
}
