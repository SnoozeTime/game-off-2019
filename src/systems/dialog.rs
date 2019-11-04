//! Manage flow of dialogs.
use amethyst::{
    assets::Loader,
    core::{shrev::EventChannel, SystemDesc},
    derive::SystemDesc,
    ecs::{
        Component, DenseVecStorage, Entities, Entity, Join, Read, System, SystemData, World, Write,
        WriteStorage,
    },
    input::{InputHandler, StringBindings},
    prelude::*,
    ui::{Anchor, TtfFormat, UiText, UiTransform},
};

use crate::event::AppEvent;

#[derive(Default, Debug, Component)]
#[storage(DenseVecStorage)]
pub struct Dialog {
    sentences: Vec<String>,
    current_sentence: usize,
    can_display_next: bool,
}

pub fn create_dialog(world: &mut World, sentences: Vec<String>) -> Entity {
    assert!(sentences.len() > 0);
    let font = world.read_resource::<Loader>().load(
        "font/square.ttf",
        TtfFormat,
        (),
        &world.read_resource(),
    );
    let text = UiText::new(
        font.clone(),
        sentences[0].clone(),
        [1.0, 1.0, 1.0, 1.0],
        50.,
    );
    let dialog = Dialog {
        sentences,
        current_sentence: 0,
        can_display_next: true,
    };
    let transform = UiTransform::new(
        "P1".to_string(),
        Anchor::TopMiddle,
        Anchor::Middle,
        -50.,
        -50.,
        91.,
        500.,
        50.,
    );

    world
        .create_entity()
        .with(transform)
        .with(dialog)
        .with(text)
        .build()
}

/// System to manage the active dialog. Just one dialog component should exist at the same time.
/// Every frame, it will check whether the dialog should move to the next sentence or whether it is
/// finished and the game can resume.
#[derive(SystemDesc)]
pub struct DialogSystem;

impl<'s> System<'s> for DialogSystem {
    type SystemData = (
        WriteStorage<'s, Dialog>,
        WriteStorage<'s, UiText>,
        Read<'s, InputHandler<StringBindings>>,
        Entities<'s>,
        Write<'s, EventChannel<AppEvent>>,
    );

    fn run(&mut self, (mut dialogs, mut texts, input, entities, mut events): Self::SystemData) {
        // let mut should_delete = vec![];
        for (dialog, text, _entity) in (&mut dialogs, &mut texts, &entities).join() {
            // can_display_next is necessary here as we just want to process the first
            // action_is_down. Maybe using events would be better?
            if input.action_is_down("confirm").unwrap_or(false) {
                if dialog.can_display_next {
                    dialog.current_sentence += 1;
                    if dialog.current_sentence < dialog.sentences.len() {
                        text.text = dialog.sentences[dialog.current_sentence].clone();
                    } else {
                        // At the end of the dialog, so we can delete the entity.
                        // state will delete the dialog
                        events.single_write(AppEvent::DialogOver);
                    }
                    dialog.can_display_next = false;
                }
            } else {
                dialog.can_display_next = true;
            }
        }

        // Get rid of the dialogs that are finished
        //        for e in should_delete {
        //            entities.delete(e).unwrap();
        //
        //            // TODO emit some event here.
        //            events.single_write(AppEvent::DialogOver);
        //        }
    }
}
