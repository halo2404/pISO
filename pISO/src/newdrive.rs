use action;
use bitmap;
use controller;
use displaymanager::{DisplayManager, Position, Widget, Window, WindowId};
use error;
use font;
use input;
use lvm;
use render;
use usb;
use utils;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::path::{Path, PathBuf};

enum NewDriveState {
    Unselected,
    PickingSize(DriveSize),
}

pub struct NewDrive {
    pub window: WindowId,
    pub usb: Arc<Mutex<usb::UsbGadget>>,
    vg: lvm::VolumeGroup,
    state: NewDriveState,
}

impl NewDrive {
    pub fn new(
        disp: &mut DisplayManager,
        usb: Arc<Mutex<usb::UsbGadget>>,
        vg: lvm::VolumeGroup,
    ) -> error::Result<NewDrive> {
        let our_window = disp.add_child(Position::Normal)?;
        Ok(NewDrive {
            window: our_window,
            state: NewDriveState::Unselected,
            usb: usb,
            vg: vg,
        })
    }
}

impl render::Render for NewDrive {
    fn render(&self, window: &Window) -> error::Result<bitmap::Bitmap> {
        let mut base = bitmap::Bitmap::new(10, 1);
        base.blit(
            &bitmap::with_border(font::render_text("New Drive"), bitmap::BorderStyle::Top, 2),
            (12, 0),
        );
        if window.focus {
            base.blit(&bitmap::Bitmap::from_slice(font::ARROW), (0, 0));
        }
        Ok(base)
    }
}

impl input::Input for NewDrive {
    fn on_event(
        &mut self,
        event: &controller::Event,
    ) -> error::Result<(bool, Vec<action::Action>)> {
        match *event {
            controller::Event::Select => Ok((true, vec![action::Action::OpenSizeMenu])),
            _ => Ok((false, vec![])),
        }
    }

    fn do_action(
        &mut self,
        disp: &mut DisplayManager,
        action: &action::Action,
    ) -> error::Result<(bool, Vec<action::Action>)> {
        match *action {
            action::Action::OpenSizeMenu => {
                let menu = DriveSize::new(disp, self.usb.clone(), self.vg.clone())?;
                disp.shift_focus(&menu);
                self.state = NewDriveState::PickingSize(menu);
                Ok((true, vec![]))
            }
            action::Action::CloseFormatMenu => {
                disp.shift_focus(self);
                self.state = NewDriveState::Unselected;
                Ok((true, vec![]))
            }
            _ => Ok((false, vec![])),
        }
    }
}

impl Widget for NewDrive {
    fn mut_children(&mut self) -> Vec<&mut Widget> {
        match self.state {
            NewDriveState::PickingSize(ref mut widget) => vec![widget],
            NewDriveState::Unselected => vec![],
        }
    }

    fn children(&self) -> Vec<&Widget> {
        match self.state {
            NewDriveState::PickingSize(ref widget) => vec![widget],
            NewDriveState::Unselected => vec![],
        }
    }

    fn windowid(&self) -> WindowId {
        self.window
    }
}

enum DriveSizeState {
    Unselected,
    Selected(DriveFormat),
}

struct DriveSize {
    pub window: WindowId,
    pub current_percent: u32,
    pub usb: Arc<Mutex<usb::UsbGadget>>,
    vg: lvm::VolumeGroup,
    state: DriveSizeState,
}

impl DriveSize {
    fn new(
        disp: &mut DisplayManager,
        usb: Arc<Mutex<usb::UsbGadget>>,
        vg: lvm::VolumeGroup,
    ) -> error::Result<DriveSize> {
        Ok(DriveSize {
            window: disp.add_child(Position::Fixed(0, 0))?,
            current_percent: 50,
            usb: usb,
            vg: vg,
            state: DriveSizeState::Unselected,
        })
    }

    fn current_size(&self) -> u64 {
        let bytes = self.vg.report().expect("Failed to get vg report").vg_size as f32
            * (self.current_percent as f32 / 100.0);
        ((bytes as u64 + 512 - 1) / 512) * 512
    }
}

impl render::Render for DriveSize {
    fn render(&self, _window: &Window) -> error::Result<bitmap::Bitmap> {
        let mut base = bitmap::Bitmap::new(0, 0);
        base.blit(&font::render_text("New drive capacity:"), (0, 0));

        let short_size = self.current_size() as f64 / (1024 * 1024 * 1024) as f64;
        base.blit(
            &font::render_text(format!("{}% ({:.2}GB)", self.current_percent, short_size)),
            (10, 30),
        );
        Ok(base)
    }
}

impl input::Input for DriveSize {
    fn on_event(
        &mut self,
        event: &controller::Event,
    ) -> error::Result<(bool, Vec<action::Action>)> {
        match *event {
            controller::Event::Select => Ok((true, vec![action::Action::OpenFormatMenu])),
            controller::Event::Up => Ok((true, vec![action::Action::IncDriveSize])),
            controller::Event::Down => Ok((true, vec![action::Action::DecDriveSize])),
        }
    }

    fn do_action(
        &mut self,
        disp: &mut DisplayManager,
        action: &action::Action,
    ) -> error::Result<(bool, Vec<action::Action>)> {
        match *action {
            action::Action::IncDriveSize => {
                self.current_percent += 5;
                Ok((true, vec![]))
            }
            action::Action::DecDriveSize => {
                self.current_percent -= 5;
                Ok((true, vec![]))
            }
            action::Action::OpenFormatMenu => {
                let menu = DriveFormat::new(disp, self.vg.clone(), self.current_size())?;
                disp.shift_focus(&menu);
                self.state = DriveSizeState::Selected(menu);
                Ok((true, vec![]))
            }
            action::Action::CloseFormatMenu => {
                self.state = DriveSizeState::Unselected;
                Ok((true, vec![]))
            }
            _ => Ok((false, vec![])),
        }
    }
}

impl Widget for DriveSize {
    fn mut_children(&mut self) -> Vec<&mut Widget> {
        match self.state {
            DriveSizeState::Selected(ref mut menu) => vec![menu],
            DriveSizeState::Unselected => vec![],
        }
    }

    fn children(&self) -> Vec<&Widget> {
        match self.state {
            DriveSizeState::Selected(ref menu) => vec![menu],
            DriveSizeState::Unselected => vec![],
        }
    }

    fn windowid(&self) -> WindowId {
        self.window
    }
}

enum InitialDriveFormat {
    Windows,
    MacOs,
    Linux,
    Universal,
}

#[derive(PartialEq)]
enum DriveFormatState {
    Selecting,
    Formatting,
    Done,
}

struct DriveFormat {
    pub windowid: WindowId,
    vg: lvm::VolumeGroup,
    size: u64,
    selected: InitialDriveFormat,
    state: DriveFormatState,
}

impl DriveFormat {
    fn new(
        disp: &mut DisplayManager,
        vg: lvm::VolumeGroup,
        size: u64,
    ) -> error::Result<DriveFormat> {
        Ok(DriveFormat {
            windowid: disp.add_child(Position::Fixed(0, 0))?,
            vg: vg,
            size: size,
            selected: InitialDriveFormat::Windows,
            state: DriveFormatState::Selecting,
        })
    }

    fn next_format(&self) -> InitialDriveFormat {
        match self.selected {
            InitialDriveFormat::Windows => InitialDriveFormat::MacOs,
            InitialDriveFormat::MacOs => InitialDriveFormat::Linux,
            InitialDriveFormat::Linux => InitialDriveFormat::Universal,
            InitialDriveFormat::Universal => InitialDriveFormat::Universal,
        }
    }

    fn prev_format(&self) -> InitialDriveFormat {
        match self.selected {
            InitialDriveFormat::Windows => InitialDriveFormat::Windows,
            InitialDriveFormat::MacOs => InitialDriveFormat::Windows,
            InitialDriveFormat::Linux => InitialDriveFormat::MacOs,
            InitialDriveFormat::Universal => InitialDriveFormat::Linux,
        }
    }

    fn format_volume(
        volume: &mut lvm::LogicalVolume,
        format: &InitialDriveFormat,
    ) -> error::Result<()> {
        // First create the partition table
        match *format {
            InitialDriveFormat::Windows
            | InitialDriveFormat::MacOs
            | InitialDriveFormat::Universal => {
                utils::run_check_output(
                    "parted",
                    &[
                        "--script",
                        &volume.path.to_string_lossy(),
                        "mklabel msdos",
                        "mkpart primary ntfs 0% 100%",
                    ],
                )?;
            }
            InitialDriveFormat::Linux => {
                utils::run_check_output(
                    "parted",
                    &[
                        "--script",
                        &volume.path.to_string_lossy(),
                        "mklabel msdos",
                        "mkpart primary ext3 0% 100%",
                    ],
                )?;
            }
        };

        let volume_path = &volume.path.to_string_lossy();
        let loopback_path =
            PathBuf::from(utils::run_check_output("losetup", &["-f"])?.trim_right());

        utils::run_check_output("losetup", &["-fPL", volume_path])?;
        utils::wait_for_path(&loopback_path, Duration::from_millis(1000))?;
        utils::run_check_output("partprobe", &[&loopback_path])?;

        let first_part_path = (loopback_path.to_string_lossy() + "p1").into_owned();
        utils::wait_for_path(&Path::new(&first_part_path), Duration::from_millis(1000))?;

        // Now do the format
        match *format {
            InitialDriveFormat::Windows => {
                utils::run_check_output("mkfs.ntfs", &["-f", &first_part_path])?;
            }
            InitialDriveFormat::MacOs | InitialDriveFormat::Universal => {
                utils::run_check_output("mkfs.exfat", &[&first_part_path as &str])?;
            }
            InitialDriveFormat::Linux => {
                utils::run_check_output("mkfs.ext3", &[&first_part_path as &str])?;
            }
        };

        //TODO: undo the loopback mount
        Ok(())
    }
}

impl render::Render for DriveFormat {
    fn render(&self, _window: &Window) -> error::Result<bitmap::Bitmap> {
        // TODO: this should pull from Display
        let mut base = bitmap::Bitmap::new(128, 64);

        if self.state != DriveFormatState::Selecting {
            base.blit(&font::render_text("Formatting new drive"), (0, 0));
            return Ok(base);
        }

        base.blit(&font::render_text("Select Format:"), (0, 0));

        base.blit(&font::render_text("Windows (NTFS)"), (10, 9));
        base.blit(&font::render_text("MacOS (EXFAT)"), (10, 9 * 2));
        base.blit(&font::render_text("Linux (EXT3)"), (10, 9 * 3));
        base.blit(&font::render_text("Universal (FAT32)"), (10, 9 * 4));

        let pos = match self.selected {
            InitialDriveFormat::Windows => (2, 9),
            InitialDriveFormat::MacOs => (2, 9 * 2),
            InitialDriveFormat::Linux => (2, 9 * 3),
            InitialDriveFormat::Universal => (2, 9 * 4),
        };
        base.blit(&bitmap::Bitmap::from_slice(font::ARROW), pos);

        Ok(base)
    }
}

impl input::Input for DriveFormat {
    fn on_event(
        &mut self,
        event: &controller::Event,
    ) -> error::Result<(bool, Vec<action::Action>)> {
        match *event {
            controller::Event::Select => Ok((true, vec![action::Action::FormatDrive])),
            controller::Event::Up => {
                self.selected = self.prev_format();
                Ok((true, vec![]))
            }
            controller::Event::Down => {
                self.selected = self.next_format();
                Ok((true, vec![]))
            }
        }
    }

    fn do_action(
        &mut self,
        _disp: &mut DisplayManager,
        action: &action::Action,
    ) -> error::Result<(bool, Vec<action::Action>)> {
        match *action {
            action::Action::FormatDrive => match self.state {
                DriveFormatState::Selecting => {
                    self.state = DriveFormatState::Formatting;
                    return Ok((false, vec![]));
                }
                _ => {
                    let count = self.vg.volumes()?.len();

                    let mut volume = self.vg
                        .create_volume(&format!("Drive{}", count), self.size)?;

                    DriveFormat::format_volume(&mut volume, &self.selected)?;

                    self.state = DriveFormatState::Done;
                    return Ok((
                        true,
                        vec![
                            action::Action::CloseFormatMenu,
                            action::Action::CreateDrive(volume),
                        ],
                    ));
                }
            },
            _ => Ok((false, vec![])),
        }
    }
}

impl Widget for DriveFormat {
    fn mut_children(&mut self) -> Vec<&mut Widget> {
        vec![]
    }

    fn children(&self) -> Vec<&Widget> {
        vec![]
    }

    fn windowid(&self) -> WindowId {
        self.windowid
    }
}
