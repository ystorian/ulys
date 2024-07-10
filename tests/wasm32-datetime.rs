#![cfg(all(target_arch = "wasm32", target_os = "unknown"))]

use ulys::Ulys;

use wasm_bindgen_test::*;
use web_time::web::SystemTimeExt;

use std::time::{Duration, SystemTime};

fn now() -> std::time::SystemTime {
    return web_time::SystemTime::now().to_std();
}

#[wasm_bindgen_test]
fn test_dynamic() {
    let ulys = Ulys::new();
    let encoded = ulys.to_string();
    let ulys2 = Ulys::from_string(&encoded).expect("failed to deserialize");

    println!("{}", encoded);
    println!("{:?}", ulys);
    println!("{:?}", ulys2);
    assert_eq!(ulys, ulys2);
}

#[wasm_bindgen_test]
fn test_source() {
    use rand::rngs::mock::StepRng;
    let mut source = StepRng::new(123, 0);

    let u1 = Ulys::with_source(&mut source);
    let dt = now() + Duration::from_millis(1);
    let u2 = Ulys::from_datetime_with_source(dt, &mut source);
    let u3 = Ulys::from_datetime_with_source(dt, &mut source);

    assert!(u1 < u2);
    assert_eq!(u2, u3);
}

#[wasm_bindgen_test]
fn test_order() {
    let dt = now();
    let ulys1 = Ulys::from_datetime(dt);
    let ulys2 = Ulys::from_datetime(dt + Duration::from_millis(1));
    assert!(ulys1 < ulys2);
}

#[wasm_bindgen_test]
fn test_datetime() {
    let dt = now();
    let ulys = Ulys::from_datetime(dt);

    println!("{:?}, {:?}", dt, ulys.datetime());
    assert!(ulys.datetime() <= dt);
    assert!(ulys.datetime() + Duration::from_millis(1) >= dt);
}

#[wasm_bindgen_test]
fn test_timestamp() {
    let dt = now();
    let ulys = Ulys::from_datetime(dt);
    let ts = dt
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis();

    assert_eq!(u128::from(ulys.timestamp_ms()), ts);
}

#[wasm_bindgen_test]
fn default_is_nil() {
    assert_eq!(Ulys::default(), Ulys::nil());
}

#[wasm_bindgen_test]
fn nil_is_at_unix_epoch() {
    assert_eq!(Ulys::nil().datetime(), SystemTime::UNIX_EPOCH);
}
