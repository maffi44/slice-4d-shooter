use std::sync::{Arc, Mutex};

use egui::{ClippedPrimitive, Context, FullOutput};
use egui_winit::State;
use winit::window::Window;


pub struct UISystem {
    // pub egui_context: Context,
    // pub egui_state: State,
    // pub egui_paint_jobs: Arc<Mutex<FullOutput>>,
}

impl UISystem {
    // pub fn new(window: &Window) -> Self {

    //     UISystem {

    //     }
    // }

    pub fn update_ui(&mut self, window: &Window) {

        // let raw_input = self.egui_state.take_egui_input(window);

        // let full_output = self.egui_context.run(
        //     raw_input,
        //     |ui| {
        //         egui::CentralPanel::default()
        //         .show(
        //             &ui,
        //             |ui| {
        //                 ui.heading("HEADING");
        //                 ui.label("1");
        //                 ui.label("2");
        //                 ui.label("3");
        //                 ui.label("4");
        //                 ui.image(egui::include_image!(
        //                     "/home/maffi/Pictures/Screenshots/Screenshot from 2024-01-10 09-44-32.png"
        //                 ));
        //             }
        //         );
        //         egui::Window::new("title")
        //         .show(
        //             &ui,
        //             |ui| {
        //                 ui.heading("HEADING");
        //                 ui.label("1");
        //                 ui.label("2");
        //                 ui.label("3");
        //                 ui.label("4");
        //                 ui.image(egui::include_image!(
        //                     "/home/maffi/Pictures/Screenshots/Screenshot from 2024-01-10 09-44-32.png"
        //                 ));
        //             }
        //         );
        //     }
        // );

        

    }
}