use crate::*;
use near_sdk_sim::to_yocto;

#[test]
fn total_supply() {
    let mut runner = Runner::default();
    runner.time_travel_to(MomentInTime::InPrivate);
    assert_eq!(runner.nft_total_supply(), 0);
}

#[test]
fn private_not_enough() {
    let mut runner = Runner::default();
    runner.time_travel_to(MomentInTime::InPrivate);
    assert!(!runner.personal_sacrifice(to_yocto("1"), 1));
    runner.assert_spend_about(&runner.alice, to_yocto("0"));
    assert_eq!(runner.nft_total_supply(), 0);
}

#[test]
fn private_ok() {
    let mut runner = Runner::default();
    runner.time_travel_to(MomentInTime::InPrivate);
    assert!(runner.personal_sacrifice(to_yocto("17.5") + to_yocto("0.01"), 1));
    runner.assert_spend_about(&runner.alice, to_yocto("17.5"));
    assert_eq!(runner.nft_total_supply(), 1);
}

#[test]
fn private_sold() {
    let runner = Runner::new(10);
    runner.take_out(10);
    assert!(!runner.personal_sacrifice(to_yocto("50"), 1));
    runner.assert_spend_about(&runner.alice, to_yocto("0"));
    assert_eq!(runner.nft_total_supply(), 10);
}

#[test]
fn private_over_quote() {
    let mut runner = Runner::default();
    runner.time_travel_to(MomentInTime::InPrivate);
    assert!(runner.personal_sacrifice(to_yocto("50"), 2));
    runner.assert_spend_about(&runner.alice, to_yocto("17.5") * 2);
    assert_eq!(runner.nft_total_supply(), 2);

    assert!(!runner.personal_sacrifice(to_yocto("50"), 1));
    runner.assert_spend_about(&runner.alice, to_yocto("17.5") * 2);
    assert_eq!(runner.nft_total_supply(), 2);
}

#[test]
fn private_wrong_signature() {
    let mut runner = Runner::default();
    runner.time_travel_to(MomentInTime::InPrivate);
    assert!(!runner.personal_sacrifice_signed(to_yocto("17.6"), 1, 2, "".to_string()));
    runner.assert_spend_about(&runner.alice, to_yocto("0"));
    assert_eq!(runner.nft_total_supply(), 0);
    let zero: [u8; 64] = [0; 64];
    assert!(!runner.personal_sacrifice_signed(to_yocto("17.6"), 1, 2, hex::encode(zero)));
    runner.assert_spend_about(&runner.alice, to_yocto("0"));
    assert_eq!(runner.nft_total_supply(), 0);
}

#[test]
fn private_before_start() {
    let mut runner = Runner::default();
    runner.time_travel_to(MomentInTime::BeforePrivate);
    assert!(!runner.personal_sacrifice(to_yocto("17.6"), 1));
    runner.assert_spend_about(&runner.alice, to_yocto("0"));
    assert_eq!(runner.nft_total_supply(), 0);
}
