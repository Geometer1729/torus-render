extern crate bmp;

use std::f64::consts::TAU;

use clap::Parser;
use bmp::{Image, Pixel};
use vecmath::{ vec3_add, vec3_dot, vec3_len, vec3_normalized, vec3_scale, vec3_sub, Vector2, Vector3};

use rayon::prelude::*;


#[derive(Parser, Debug)]
#[clap(author = "Brian Kuhns", version, about)]
/// Application configuration
struct Args {
    /// whether to be verbose
    #[arg(short = 'v')]
    verbose: bool,

    /// an optional name to greet
    #[arg(long = "lat")]
    lat: Option<f64>,

    #[arg(long = "long")]
    long: Option<f64>,

    #[arg(long = "width", short='w')]
    width: Option<u32>,

    #[arg(long = "height", short='g')]
    height: Option<u32>,

    #[arg(long = "scale", short='s')]
    scale: Option<f64>,

    #[arg(long = "map", short='m')]
    map: Option<String>,
}

fn main() {
    let args = Args::parse();
    if args.verbose {
        println!("DEBUG {args:?}");
    }

    let map_name = args.map.unwrap_or("testMap.bmp".to_string());
    let source_map = bmp::open(map_name).expect("failed to open");
    let long = args.long.unwrap_or(0.0) * TAU/360.0;
    let lat = (args.lat.unwrap_or(0.0) + 90.0) * TAU/360.0;
    let width = args.width.unwrap_or(256);
    let height = args.height.unwrap_or(256);
    let scale = args.scale.unwrap_or(1.0);

    let mut img = Image::new(width, height);
    let writes : Vec<(u32,u32,Pixel)> =
        img.coordinates()
            .collect::<Vec<_>>()
            .par_iter()
            .map(|&(x,y)|(x,y,pixel_for(scale,width,height,lat,long,&source_map,x,y)))
            .collect();
    for &(x,y,p) in writes.iter() {
        img.set_pixel(x,y,p);
    }
    let _ = img.save("img.bmp");

}

fn pixel_for(scale:f64,width : u32, height :u32, lat : f64, long : f64, source_map : &Image, x : u32,y :u32) -> Pixel {
    let x1 = (x as f64 - (width/2) as f64)*scale as f64;
    let y1 = (y as f64 - (height/2) as f64)*scale as f64;
    let [x2,y2] = project([long,lat],[x1,y1]);
    let x3 = (x2 + TAU/2.0)/TAU*(source_map.get_width() as f64);
    let y3 = (y2 + TAU/2.0)/TAU*(source_map.get_height() as f64);
    source_map.get_pixel(x3 as u32,y3 as u32)
}

const RMAJ : f64 = 90_000.0;
const RMIN : f64 = 30_000.0;

type Pt = Vector3<f64>;
type Pt2 = Vector2<f64>;
type State = (Pt,Pt);

fn project(center : Pt2,p : Pt2) -> Pt2 {
    let [theta,phi] = center;
    let mut x =  angle_to_r3(center);
    let r = RMAJ + f64::cos(phi) * RMIN;
    let vdtheta = vec3_normalized(
        [ r * f64::sin(theta)
        , -r * f64::cos(theta)
        , 0.0
        ]);
    let vdphi = vec3_normalized(
        [ f64::sin(phi) * f64::cos(theta) * -RMIN
        , f64::sin(phi) * f64::sin(theta) * -RMIN
        , f64::cos(phi) * RMIN
        ]);
    let [dtheta,dphi] = p;
    let mut v = vec3_add(vec3_scale(vdtheta,dtheta),vec3_scale(vdphi,dphi));
    let l = vec3_len(v);
    v = vec3_normalized(v);
    if p != center {
        for _ in 0..(l as u32) {
            (x,v) = step((x,v));

        }
    }
    r3_to_angular(x)
}

fn angle_to_r3(angular : Pt2 ) -> Pt {
    let [theta,phi] = angular;
    let r = RMAJ + f64::cos(phi) * RMIN;
    [ r * f64::cos(theta)
    , r * f64::sin(theta)
    , RMIN * f64::sin(phi)
    ]
}

fn r3_to_angular(p : Pt) -> Pt2 {
    let [x,y,z] = p;
    let theta = f64::atan2(y,x);
    let phi = f64::atan2(z,f64::sqrt(x*x+y*y)-RMAJ);
    [theta,phi]
}

fn step((x,v) : State) -> State {
    let (x2,normal) = surface(vec3_add(x,v));
    //println!("x2 {x2:?} normal {normal:?}");
    let v2_1 = vec3_sub(x2,x);
    let v2_2 = vec3_sub(v2_1,vec3_scale(normal,vec3_dot(v2_1,normal)));
    (x2,vec3_normalized(v2_2))
}

// returns new surfaced point and normal
fn surface(p : Pt) -> (Pt,Pt) {
    let [x,y,_z] = p;
    let r_horiz = f64::sqrt(x*x+y*y);
    let center_pt = vec3_scale([x,y,0.0],RMAJ/r_horiz);
    // closest point on the circle at the center of the torus
    let delta = vec3_sub(p , center_pt);
    let normal = vec3_normalized(delta);
    let surfaced =vec3_add(center_pt,vec3_scale(delta,RMIN/vec3_len(delta)));
    (surfaced,normal)
}
