use nih_plug::prelude::*;

use simple_bitcrush::SimpleBitcrush;

fn main() {
    nih_export_standalone::<SimpleBitcrush>();
}