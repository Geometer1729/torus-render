// Object orientation was a mistake
// this is kinda silly but it's so much more readable without the `f64::` everywhere
pub fn cos(x: f64) -> f64 {
    x.cos()
}
pub fn sin(x: f64) -> f64 {
    x.sin()
}
pub fn sqrt(x: f64) -> f64 {
    x.sqrt()
}
pub fn atan2(x: f64, y: f64) -> f64 {
    f64::atan2(x, y)
}
