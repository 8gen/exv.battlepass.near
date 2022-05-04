use crate::*;
use near_sdk_sim::to_yocto;

#[test]
fn total_supply() {
    let mut runner = Runner::default();
    runner.time_travel_to(MomentInTime::InPrivate);
    assert_eq!(runner.nft_total_supply(), 0);
}

#[test]
fn open_ok_many() {
    let mut runner = Runner::default();
    runner.time_travel_to(MomentInTime::InPrivate);
    runner.change_price(1e24 as u128);
    assert!(runner.personal_sacrifice_force(to_yocto("51"), 20));
    assert_eq!(runner.nft_total_supply(), 20);
    runner.assert_spend_about(&runner.alice, to_yocto("20"));
}

#[test]
fn open_ok() {
    let mut runner = Runner::default();
    runner.time_travel_to(MomentInTime::AfterPrivate);
    assert!(runner.sacrifice(to_yocto("17.5") * 2 + to_yocto("0.05") * 2, 2));
    assert_eq!(runner.nft_total_supply(), 2);
    runner.assert_spend_about(&runner.alice, to_yocto("17.5") * 2);
}

#[test]
fn open_wrong_signature() {
    let mut runner = Runner::default();
    runner.time_travel_to(MomentInTime::AfterPrivate);
    assert!(runner.personal_sacrifice_signed(to_yocto("17.6"), 1, 2, "".to_string()));
    runner.assert_spend_about(&runner.alice, to_yocto("17.5"));
    assert_eq!(runner.nft_total_supply(), 1);
}

#[test]
fn open_before_start() {
    let mut runner = Runner::default();
    runner.time_travel_to(MomentInTime::InPrivate);
    assert!(!runner.sacrifice(to_yocto("50"), 2));
    runner.assert_spend_about(&runner.alice, to_yocto("0"));
    assert_eq!(runner.nft_total_supply(), 0);
}

#[test]
fn open_not_enough() {
    let mut runner = Runner::default();
    runner.time_travel_to(MomentInTime::AfterPrivate);
    assert!(!runner.sacrifice(to_yocto("20"), 2));
    runner.assert_spend_about(&runner.alice, to_yocto("0"));
    assert_eq!(runner.nft_total_supply(), 0);
}

#[test]
fn open_sold() {
    let runner = Runner::new(10);
    runner.take_out(10);
    assert!(!runner.sacrifice(to_yocto("50"), 1));
    runner.assert_spend_about(&runner.alice, to_yocto("0"));
    assert_eq!(runner.nft_total_supply(), 10);
}
