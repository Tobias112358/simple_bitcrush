//use atomic_float::AtomicF32;
use nih_plug::prelude::Editor;
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::widgets::*;
use nih_plug_vizia::{create_vizia_editor, ViziaState, ViziaTheming};

//use std::sync::atomic::Ordering;
use std::sync::Arc;
//use std::time::Duration;

use crate::SimpleBitcrushParams;

mod my_assets;

pub mod my_fonts;

#[derive(Lens)]
struct Data {
    params: Arc<SimpleBitcrushParams>,
}

impl Model for Data {}

// Makes sense to also define this here, makes it a bit easier to keep track of
pub(crate) fn default_state() -> Arc<ViziaState> {
    ViziaState::new(|| (350, 350))
}

pub(crate) fn create(
    params: Arc<SimpleBitcrushParams>,
    editor_state: Arc<ViziaState>,
) -> Option<Box<dyn Editor>> {
    create_vizia_editor(editor_state, ViziaTheming::Custom, move |cx, _| {
        //assets::register_noto_sans_light(cx);
        //assets::register_noto_sans_thin(cx);
        //vizia_assets::register_roboto_bold(cx);
        //vizia_assets::register_roboto_italic(cx);
        //vizia_assets::register_roboto(cx);


        cx.add_font_mem(&my_fonts::RED_ROSE_REGULAR);

        
        my_assets::register_red_rose_regular(cx);
        my_assets::register_red_rose_bold(cx);
        my_assets::register_red_rose_light(cx);
        my_assets::register_red_rose_semi_bold(cx);
        my_assets::register_red_rose_medium(cx);
        my_assets::register_red_rose_variable_weight(cx);


        Data {
            params: params.clone(),
        }
        .build(cx);

        VStack::new(cx, |cx| {
            
            Label::new(cx, "Rate");
            ParamSlider::new(cx, Data::params, |params| &params.rate);
        })
        .row_between(Pixels(0.0))
        .child_left(Stretch(1.0))
        .child_right(Stretch(1.0))
        .height(Pixels(350.0))
        .background_color(RGBA::rgb(0,0,0));

        ResizeHandle::new(cx);
    })
    
}