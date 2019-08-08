extern crate intercom;
use intercom::*;
use std::convert::TryFrom;

use server::{IAnimal, IDomesticAnimal, ICat};

pub const CLSID_BRITISH_SHORT_HAIR_CAT: GUID = GUID {
    data1: 0xb70be3a9,
    data2: 0x6531,
    data3: 0x423b,
    data4: [0xb0, 0xd6, 0x30, 0x42, 0xa4, 0x7d, 0x06, 0x78],
};

fn test() {
    let ptr = ComRc::<IAnimal>::create(CLSID_BRITISH_SHORT_HAIR_CAT).unwrap();
    ptr.eat();

    // Switch interfaces
    let ptr = ComRc::<IDomesticAnimal>::try_from(&ptr).unwrap();
    ptr.train();

    let ptr = ComRc::<ICat>::try_from(&ptr).unwrap();
    ptr.ignore_humans();
}

fn main() {
    intercom::runtime::initialize()
        .map_err( |hr| format!( "Failed to initialize COM: {:?}", hr ) );

    test();

    intercom::runtime::uninitialize();
}