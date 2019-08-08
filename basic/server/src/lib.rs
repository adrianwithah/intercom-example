use intercom::*;

com_library!(BritishShortHairCat);

#[com_interface(
    com_iid = "014aadf1-6dee-4c6d-af1f-b94888992e4f",
    raw_iid = "8d825b8f-6f25-4c1c-a0fa-619e17a504b9")]
pub trait IAnimal {
    fn eat(&self);
}

#[com_interface(
    com_iid = "67c9e25f-ed5b-4c4c-a6fa-63468985580d",
    raw_iid = "976b644e-af68-480a-9918-f5376e263bac")]
pub trait ICat {
    fn ignore_humans(&self);
}

#[com_interface(
    com_iid = "f6fab37c-e998-4e40-a068-5d63f39ab580",
    raw_iid = "1f840be7-a34f-4ef7-8020-1aa29d1dc5c4")]
pub trait IDomesticAnimal {
    fn train(&self);
}

#[com_class(
    clsid = "b70be3a9-6531-423b-b0d6-3042a47d0678", IAnimal, ICat, IDomesticAnimal)]
#[derive(Default)]
struct BritishShortHairCat {
    foo: i32
}

impl BritishShortHairCat {
    fn new() -> Self {
        BritishShortHairCat::default()
    }
}

#[com_impl]
impl IAnimal for BritishShortHairCat {
    fn eat(&self) {
        println!("Eat");
    }
}

#[com_impl]
impl ICat for BritishShortHairCat {
    fn ignore_humans(&self) {
        println!("Ignore Humans");
    }
}

#[com_impl]
impl IDomesticAnimal for BritishShortHairCat {
    fn train(&self) {
        println!("Training");
    }
}

