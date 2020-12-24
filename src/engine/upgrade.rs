use crate::engine::Engine;
use crate::input::MyGameInput;
use crate::message::{MainMessage, Message};
use crate::server::Server;
use crate::ui::widget::button;
use crate::ui::widget::button::Button;
use crate::ui::widget::text::Text;
use crate::ui::{Column, Element};
use crate::util;
use coffee::graphics::{Color, Frame, Window};
use coffee::input::keyboard;
use coffee::ui::{Align, Justify};
use coffee::Timer;
use std::io::Write;
use std::{env, fs};

pub struct UpgradeEngine {
    version: (u8, u8, u8),
    mandatory: bool,
    server: Server,
    already_on_disk: bool,
    display_downloading: bool,
    make_download: bool,
    downloaded: bool,
    download_button: button::State,
    cancel_button: button::State,
    continue_button: button::State,
}

impl UpgradeEngine {
    pub fn new(version: (u8, u8, u8), mandatory: bool, server: Server) -> Self {
        // FIXME: test it
        let (major, minor, correction) = version;
        let current_exe = env::current_exe().unwrap();
        let rolling_folder = current_exe.parent().unwrap();
        let already_on_disk = rolling_folder
            .join(format!("{}.{}.{}", major, minor, correction))
            .is_dir();

        Self {
            version,
            mandatory,
            server,
            already_on_disk,
            display_downloading: false,
            make_download: false,
            downloaded: false,
            download_button: button::State::new(),
            cancel_button: button::State::new(),
            continue_button: button::State::new(),
        }
    }
}

impl Engine for UpgradeEngine {
    fn draw(&mut self, frame: &mut Frame, _timer: &Timer) {
        frame.clear(Color::BLACK);
    }

    fn update(&mut self, _window: &Window) -> Option<MainMessage> {
        if self.make_download {
            let (major, minor, correction) = self.version;
            let remote_file_name = if cfg!(windows) {
                format!(
                    "Rolling_Windows_x86-64_{}.{}.{}.zip",
                    major, minor, correction
                )
            } else {
                format!(
                    "Rolling_Linux_x86-64_{}.{}.{}.zip",
                    major, minor, correction
                )
            };
            let extracted_folder_name = if cfg!(windows) {
                "Rolling_Windows_x86-64"
            } else {
                "Rolling_Linux_x86-64"
            };

            let client = reqwest::blocking::Client::new();
            let url = &format!("http://rolling.bux.fr/release/{}", remote_file_name);
            println!("download {} ...", url);
            let response = client.get(url).send().unwrap();
            let zip_content = response.bytes().unwrap();
            let mut zip_file = fs::File::create(&remote_file_name).unwrap();
            zip_file.write(&zip_content).unwrap();
            let zip_file = fs::File::open(&remote_file_name).unwrap();
            util::unzip_to(zip_file);
            fs::rename(
                extracted_folder_name,
                &format!("{}.{}.{}", major, minor, correction),
            )
            .unwrap();

            self.display_downloading = false;
            self.make_download = false;
            self.downloaded = true;
        }

        None
    }

    fn interact(&mut self, input: &mut MyGameInput, _window: &mut Window) -> Option<MainMessage> {
        match input.key_code {
            Some(keyboard::KeyCode::Escape) => {
                input.key_code = None;
                return Some(MainMessage::ToStartup);
            }
            _ => {}
        }

        None
    }

    fn react(&mut self, event: Message, _window: &mut Window) -> Option<MainMessage> {
        match event {
            Message::CancelButtonPressed => return Some(MainMessage::ToStartup),
            Message::DownloadButtonPressed => {
                self.display_downloading = true;
            }
            Message::ContinueButtonPressed => {
                return Some(MainMessage::StartupToZone {
                    server_ip: self.server.config.ip.clone(),
                    server_port: self.server.config.port,
                    disable_version_check: true,
                })
            }
            Message::ExitMenuButtonPressed => return Some(MainMessage::ExitRequested),
            _ => {}
        }

        None
    }

    fn layout(&mut self, window: &Window) -> Element {
        let title = if self.mandatory {
            "Votre version n'est plus compatible avec ce serveur"
        } else {
            "Une nouvelle version est disponible pour ce serveur"
        };

        let mut column = Column::new()
            .max_width(768)
            .height(window.height() as u32)
            .align_items(Align::Center)
            .justify_content(Justify::Center)
            .spacing(20)
            .push(Text::new(title).size(50));
        let (major, minor, correction) = self.version;

        if self.display_downloading {
            self.make_download = true;
            column = column.push(Text::new("Téléchargement en cours ...").size(20));
        } else if self.downloaded {
            column = column.push(
                Text::new(&format!("Téléchargement terminé. Quittez puis lancez rolling depuis le dossier {}.{}.{} pour jouer sur ce serveur", major, minor, correction))
                    .size(20),
            ).push(
                Button::new(&mut self.cancel_button, "Quitter")
                    .on_press(Message::ExitMenuButtonPressed)
                    .class(button::Class::Primary),
            );
        } else {
            if self.already_on_disk {
                column = column.push(
                    Text::new(&format!(
                        "Pour l'utiliser, quitter, puis lancer rolling dans le dossier {}.{}.{}",
                        major, minor, correction
                    ))
                    .size(20),
                );
            } else {
                column = column.push(
                    Button::new(&mut self.download_button, "Télécharger")
                        .on_press(Message::DownloadButtonPressed)
                        .class(button::Class::Primary),
                );
            };

            if self.mandatory {
                column = column.push(
                    Button::new(&mut self.cancel_button, "Retour")
                        .on_press(Message::CancelButtonPressed)
                        .class(button::Class::Secondary),
                );
            } else {
                column = column.push(
                    Button::new(&mut self.continue_button, "Non merci")
                        .on_press(Message::ContinueButtonPressed)
                        .class(button::Class::Secondary),
                );
            }
        }

        column.into()
    }

    fn teardown(&mut self) {}
}
