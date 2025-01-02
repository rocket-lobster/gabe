use std::{
    collections::VecDeque,
    fs::{File, OpenOptions},
    io::{Read, Seek, Write},
};

use egui::{load::SizedTexture, ColorImage, Image, Key, TextureHandle, TextureOptions, Vec2};
use gabe_core::gb::{Gameboy, GbKeys};
use gabe_core::sink::{AudioFrame, Sink};

use crate::{audio_driver::AudioDriver, video_sinks};

const CYCLE_TIME_NS: f32 = 238.41858;

struct SimpleAudioSink {
    inner: VecDeque<AudioFrame>,
}

impl Sink<AudioFrame> for SimpleAudioSink {
    fn append(&mut self, value: AudioFrame) {
        self.inner.push_back(value);
    }
}

pub struct GabeApp {
    emu: Option<gabe_core::gb::Gameboy>,
    emulated_cycles: u64,
    start_time: u64,
    save_file: Option<File>,
    audio_driver: AudioDriver,
    framebuffer: TextureHandle,
}

impl GabeApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        Self {
            emu: None,
            emulated_cycles: 0,
            start_time: 0,
            save_file: None,
            audio_driver: AudioDriver::new(gabe_core::SAMPLE_RATE, 100),
            framebuffer: cc.egui_ctx.load_texture(
                "framebuffer",
                ColorImage::default(),
                Default::default(),
            ),
        }
    }
}

impl eframe::App for GabeApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Menu Bar UI
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open File...").clicked() {
                        if let Some(mut path) = rfd::FileDialog::new().pick_file() {
                            let mut rom_file = std::fs::File::open(&path).unwrap();
                            path.set_extension("sav");
                            let mut save_file = OpenOptions::new()
                                .write(true)
                                .read(true)
                                .create(true)
                                .open(path)
                                .unwrap();
                            let mut rom_data = vec![];
                            rom_file.read_to_end(&mut rom_data).unwrap();
                            let mut save_data = vec![];
                            save_file.read_to_end(&mut save_data).unwrap();
                            self.emu = Some(gabe_core::gb::Gameboy::power_on(
                                rom_data.into_boxed_slice(),
                                Some(save_data.into_boxed_slice()),
                            ));
                            self.save_file = Some(save_file);
                            self.audio_driver.play();
                            self.start_time = self.audio_driver.time_source().time_ns();
                        }
                        ui.close_menu();
                    }
                });
                ui.menu_button("Emulation", |ui| {
                    ui.add_enabled_ui(self.emu.is_some(), |ui| {
                        if ui.button("Stop").clicked() {
                            if let Some(emu) = &mut self.emu {
                                // Stop all emulation, reset state
                                self.audio_driver.stop();
                                // Save the data to the save file, if valid
                                if let (Some(data), Some(save_file)) =
                                    (emu.get_save_data(), &mut self.save_file)
                                {
                                    if let Err(e) = save_file.rewind() {
                                        println! {"{}: No save file written.", e};
                                    }
                                    if let Err(e) = save_file.write_all(&data) {
                                        println! {"{}: Corrupt save file written.", e};
                                    }
                                }
                                // Setting to None drops the Gameboy object
                                self.emu = None;
                                self.emulated_cycles = 0;
                                // Clear framebuffer
                                self.framebuffer
                                    .set(ColorImage::default(), Default::default());
                            }
                            ui.close_menu();
                        }
                    })
                });
            });
        });

        // Main Render Panel
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(emu) = &mut self.emu {
                // Currently running a game
                let mut video_sink = video_sinks::BlendVideoSink::new();
                let mut audio_sink = SimpleAudioSink {
                    inner: VecDeque::new(),
                };
                let time_source = self.audio_driver.time_source();
                let mut audio_buffer_sink = self.audio_driver.sink();

                let target_emu_time_ns = time_source.time_ns() - self.start_time;
                let target_emu_cycles = (target_emu_time_ns as f32 / CYCLE_TIME_NS).floor() as u64;
                while self.emulated_cycles < target_emu_cycles {
                    self.emulated_cycles += emu.step(&mut video_sink, &mut audio_sink) as u64;

                    if let Some(frame) = video_sink.get_frame() {
                        self.framebuffer.set(
                            ColorImage::from_rgb([160, 144], &frame),
                            TextureOptions {
                                magnification: egui::TextureFilter::Nearest,
                                minification: egui::TextureFilter::Nearest,
                            },
                        );
                    }
                    update_key_states(ctx, emu);
                }
                audio_buffer_sink.append(audio_sink.inner.as_slices().0);
                ui.add(
                    Image::new(SizedTexture::from_handle(&self.framebuffer))
                        .fit_to_fraction(Vec2::new(1.0, 1.0)),
                );
                ctx.request_repaint();
            } else {
                ui.heading("Use File->Open File to select and run a valid ROM file.");
            }
        });
    }
}

fn update_key_states(ctx: &egui::Context, gb: &mut Gameboy) {
    ctx.input(|i| {
        gb.update_key_state(GbKeys::A, i.key_down(Key::X));
        gb.update_key_state(GbKeys::B, i.key_down(Key::Z));
        gb.update_key_state(GbKeys::Start, i.key_down(Key::Enter));
        gb.update_key_state(GbKeys::Select, i.key_down(Key::Backspace));
        gb.update_key_state(GbKeys::Up, i.key_down(Key::ArrowUp));
        gb.update_key_state(GbKeys::Down, i.key_down(Key::ArrowDown));
        gb.update_key_state(GbKeys::Left, i.key_down(Key::ArrowLeft));
        gb.update_key_state(GbKeys::Right, i.key_down(Key::ArrowRight));
    });
}
