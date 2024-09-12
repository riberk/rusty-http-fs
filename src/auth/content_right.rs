use bitflags::bitflags;

#[derive(Debug, PartialEq, Eq)]
pub struct ContentRight(u32);

bitflags! {
    impl ContentRight: u32 {
        const None =  0b_0000_0000;
        const Read =  0b_0000_0001;
        const Write = 0b_0000_0010;
        const All = !0;
    }
}
