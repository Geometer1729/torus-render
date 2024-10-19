use crate::trig::*; // rexports float methods as functions

use vecmath::{ vec3_add, vec3_dot, vec3_len, vec3_normalized, vec3_scale, vec3_sub, Vector2, Vector3};

const RMAJ : f64 = 90_000.0;
const RMIN : f64 = 30_000.0;

type Pt = Vector3<f64>;
type Pt2 = Vector2<f64>;
type State = (Pt,Pt);


pub fn project(center : Pt2,p : Pt2) -> Pt2 {
    let [theta,phi] = center;
    let mut x =  angle_to_r3(center);
    let vdtheta =
        [ sin(theta)
        , -1.0 * cos(theta)
        , 0.0
        ];
    let vdphi = vec3_normalized(
        [ sin(phi) * cos(theta) * -1.0
        , sin(phi) * sin(theta) * -1.0
        , cos(phi)
        ]);
    let [dtheta,dphi] = p;
    let mut v = vec3_add(vec3_scale(vdtheta,dtheta),vec3_scale(vdphi,dphi));
    let l = vec3_len(v);
    v = vec3_normalized(v);
    let target_step = 10.0;
    let steps = (l/target_step) as u32;
    let d : f64 = l / steps as f64;
    if p != center {
        for _ in 0..steps {
            (x,v) = step((x,v),d);

        }
    }
    r3_to_angular(x)
}

pub fn angle_to_r3(angular : Pt2 ) -> Pt {
    let [theta,phi] = angular;
    let r = RMAJ + cos(phi) * RMIN;
    [ r * cos(theta)
    , r * sin(theta)
    , RMIN * sin(phi)
    ]
}

pub fn r3_to_angular(p : Pt) -> Pt2 {
    let [x,y,z] = p;
    let theta = atan2(y,x);
    let phi = atan2(z,sqrt(x*x+y*y)-RMAJ);
    [theta,phi]
}

pub fn step((x,v) : State,d : f64) -> State {
    let (x2,normal) = surface(vec3_add(x,vec3_scale(v,d)));
    //println!("x2 {x2:?} normal {normal:?}");
    let v2_1 = vec3_sub(x2,x);
    let v2_2 = vec3_sub(v2_1,vec3_scale(normal,vec3_dot(v2_1,normal)));
    (x2,vec3_normalized(v2_2))
}

// returns new surfaced point and normal
pub fn surface(p : Pt) -> (Pt,Pt) {
    let [x,y,_z] = p;
    let r_horiz = sqrt(x*x+y*y);
    let center_pt = vec3_scale([x,y,0.0],RMAJ/r_horiz);
    // closest point on the circle at the center of the torus
    let delta = vec3_sub(p , center_pt);
    let normal = vec3_normalized(delta);
    let surfaced = vec3_add(center_pt,vec3_scale(delta,RMIN/vec3_len(delta)));
    (surfaced,normal)
}
