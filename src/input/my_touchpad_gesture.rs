use crate::{niri::State, utils::with_toplevel_role};

static CHROME_CLOSE_TAB: &[&str] = &["key", "29:1", "17:1", "17:0", "29:0"];
static CHROME_LEFT_TAB: &[&str] = &["key", "29:1", "42:1", "15:1", "15:0", "42:0", "29:0"];
static CHROME_RIGHT_TAB: &[&str] = &["key", "29:1", "15:1", "15:0", "29:0"];
static CHROME_REFRESH: &[&str] = &["key", "29:1", "19:1", "19:0", "29:0"];
static CHROME_BACK: &[&str] = &["key", "158:1", "158:0"];

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum GestureState {
    Unknown,
    Deciding,
    Decided,
}
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum GestureDirection {
    Up,
    Down,
    Left,
    Right,
    Horizontal,
    Vertical,
    In,
    Out,
    Unknown,
}

pub struct SwipeGesture {
    cx: f64,
    cy: f64,
    direction: GestureDirection,
    decision: GestureState,
}
pub struct PinchGesture {
    scale: f64,
    direction: GestureDirection,
    decision: GestureState,
}
pub struct HoldGesture {
    // millisecond
    begin_ts: u32,
    decision: GestureState,
}

pub struct MyTouchpadGesture {
    pub swipe_3f: SwipeGesture,
    pub swipe_4f: SwipeGesture,
    pub pinch_3f: PinchGesture,
    pub pinch_4f: PinchGesture,
    pub hold_3f: HoldGesture,
    pub hold_4f: HoldGesture,
}

impl SwipeGesture {
    pub fn new() -> Self {
        Self {
            cx: 0.,
            cy: 0.,
            direction: GestureDirection::Unknown,
            decision: GestureState::Unknown,
        }
    }

    pub fn reset(&mut self) -> () {
        self.cx = 0.;
        self.cy = 0.;
        self.direction = GestureDirection::Unknown;
        self.decision = GestureState::Unknown;
    }

    pub fn begin(&mut self) -> () {
        self.decision = GestureState::Deciding;
    }

    pub fn update_and_maybe_decide(
        &mut self,
        delta_x: f64,
        delta_y: f64,
    ) -> Option<GestureDirection> {
        if self.decision != GestureState::Deciding {
            return None;
        }

        self.cx += delta_x;
        self.cy += delta_y;

        // Check if the gesture moved far enough to decide. Threshold copied from GNOME Shell.
        if self.cx * self.cx + self.cy * self.cy >= 16. * 16. {
            self.direction = if self.cx.abs() > self.cy.abs() {
                GestureDirection::Horizontal
            } else {
                GestureDirection::Vertical
            };
            self.decision = GestureState::Decided;
            Some(self.direction)
        } else {
            // Undecided, needs more data (movement)
            None
        }
    }
}

impl PinchGesture {
    pub fn new() -> Self {
        Self {
            scale: 0.,
            direction: GestureDirection::Unknown,
            decision: GestureState::Unknown,
        }
    }
    pub fn reset(&mut self) -> () {
        self.scale = 0.;
        self.direction = GestureDirection::Unknown;
        self.decision = GestureState::Unknown;
    }

    pub fn begin(&mut self) -> () {
        self.decision = GestureState::Deciding;
    }
}

impl HoldGesture {
    pub fn new() -> Self {
        Self {
            begin_ts: 0,
            decision: GestureState::Unknown,
        }
    }

    pub fn reset(&mut self) -> () {
        self.begin_ts = 0;
        self.decision = GestureState::Unknown;
    }

    pub fn begin(&mut self, ts: u32) -> () {
        self.begin_ts = ts;
        self.decision = GestureState::Decided;
    }
}

impl MyTouchpadGesture {
    pub fn new() -> Self {
        Self {
            swipe_3f: SwipeGesture::new(),
            swipe_4f: SwipeGesture::new(),
            pinch_3f: PinchGesture::new(),
            pinch_4f: PinchGesture::new(),
            hold_3f: HoldGesture::new(),
            hold_4f: HoldGesture::new(),
        }
    }
}

impl State {
    pub fn swipe_3f_on_update(&mut self) -> bool {
        if self.niri.my_touchpad_gesture.swipe_3f.decision != GestureState::Decided {
            return false;
        }
        false
    }

    pub fn swipe_4f_on_update(&mut self, dx: f64, _dy: f64) -> bool {
        let swipe = &mut self.niri.my_touchpad_gesture.swipe_4f;
        // if self.niri.my_touchpad_gesture.swipe_4f.decision != GestureState::Decided {
        if swipe.decision != GestureState::Decided {
            return false;
        }
        // if self.niri.my_touchpad_gesture.swipe_4f.direction == GestureDirection::Horizontal {
        if swipe.direction == GestureDirection::Horizontal {
            let is_chrome = if let Some(mapped) = self.niri.layout.focus() {
                with_toplevel_role(mapped.toplevel(), |role| {
                    if role.app_id.as_deref() == Some("google-chrome") {
                        true
                    } else {
                        false
                    }
                })
            } else {
                false
            };
            if is_chrome {
                swipe.cx += dx;
                if swipe.cx.abs() > 150.0 {
                    if swipe.cx < 0. {
                        spawn(CHROME_LEFT_TAB);
                    } else {
                        spawn(CHROME_RIGHT_TAB);
                    }
                    swipe.cx = 0.;
                }
            } else {
            }
            return true;
        }
        false
    }

    pub fn swipe_4f_on_end(&mut self, _cancelled: bool) -> bool {
        let swipe = &mut self.niri.my_touchpad_gesture.swipe_4f;
        if swipe.decision == GestureState::Unknown {
            return false;
        }
        swipe.reset();
        true
    }

    pub fn pinch_3f_on_update(&mut self, scale: f64) -> bool {
        let pinch = &mut self.niri.my_touchpad_gesture.pinch_3f;
        match pinch.decision {
            GestureState::Unknown => {
                return false;
            }
            GestureState::Deciding => {
                if scale < 0.9 {
                    pinch.direction = GestureDirection::In;
                } else if scale > 1.1 {
                    pinch.direction = GestureDirection::Out;
                } else {
                    // need more data
                    return true;
                }
                pinch.decision = GestureState::Decided;
            }
            GestureState::Decided => {
                pinch.scale = scale;
            }
        }
        true
    }

    pub fn pinch_3f_on_end(&mut self, cancelled: bool) -> bool {
        let niri = &mut self.niri;
        match niri.my_touchpad_gesture.pinch_3f.decision {
            GestureState::Unknown => {
                return false;
            }
            GestureState::Deciding => {}
            GestureState::Decided => {
                if cancelled {
                    return true;
                }
                if (0.7..1.3).contains(&niri.my_touchpad_gesture.pinch_3f.scale) {
                    return true;
                }
                let window = niri.window_under_cursor();
                if let Some(mapped) = window {
                    let w = mapped.window.clone();
                    if niri.my_touchpad_gesture.pinch_3f.direction == GestureDirection::In {
                        niri.layout.toggle_window_width(Some(&w), false);
                    } else {
                        niri.layout.toggle_window_width(Some(&w), true);
                    }
                }
            }
        }
        niri.my_touchpad_gesture.pinch_3f.reset();
        true
    }

    pub fn pinch_4f_on_update(&mut self, scale: f64) -> bool {
        let pinch = &mut self.niri.my_touchpad_gesture.pinch_4f;
        match pinch.decision {
            GestureState::Unknown => {
                return false;
            }
            GestureState::Deciding => {
                if scale < 0.9 {
                    pinch.direction = GestureDirection::In;
                } else if scale > 1.1 {
                    pinch.direction = GestureDirection::Out;
                } else {
                    // need more data
                    return true;
                }
                pinch.decision = GestureState::Decided;
            }
            GestureState::Decided => {
                pinch.scale = scale;
            }
        }
        true
    }

    pub fn pinch_4f_on_end(&mut self, cancelled: bool) -> bool {
        let niri = &mut self.niri;
        match niri.my_touchpad_gesture.pinch_4f.decision {
            GestureState::Unknown => {
                return false;
            }
            GestureState::Deciding => {}
            GestureState::Decided => {
                if cancelled {
                    return true;
                }
                if (0.7..1.3).contains(&niri.my_touchpad_gesture.pinch_4f.scale) {
                    return true;
                }
                let is_chrome = if let Some(mapped) = niri.layout.focus() {
                    with_toplevel_role(mapped.toplevel(), |role| {
                        if role.app_id.as_deref() == Some("google-chrome") {
                            true
                        } else {
                            false
                        }
                    })
                } else {
                    false
                };
                if niri.my_touchpad_gesture.pinch_4f.direction == GestureDirection::In {
                    if is_chrome {
                        spawn(CHROME_BACK);
                    } else {
                    }
                } else {
                    if is_chrome {
                        spawn(CHROME_REFRESH);
                    } else {
                    }
                }
            }
        }
        niri.my_touchpad_gesture.pinch_4f.reset();
        true
    }

    pub fn hold_4f_on_end(&mut self, ts: u32, cancelled: bool) -> bool {
        if self.niri.my_touchpad_gesture.hold_4f.decision != GestureState::Decided {
            return false;
        }
        if ts < self.niri.my_touchpad_gesture.hold_4f.begin_ts + 300 {
        } else if cancelled {
        } else {
            let is_chrome = if let Some(mapped) = self.niri.layout.focus() {
                with_toplevel_role(mapped.toplevel(), |role| {
                    if role.app_id.as_deref() == Some("google-chrome") {
                        true
                    } else {
                        false
                    }
                })
            } else {
                false
            };
            if is_chrome {
                spawn(CHROME_CLOSE_TAB);
            } else {
                let window = self.niri.window_under_cursor();
                if let Some(mapped) = window {
                    mapped.toplevel().send_close();
                }
            }
        }
        self.niri.my_touchpad_gesture.hold_4f.reset();
        true
    }
}

fn spawn(args: &[&str]) -> () {
    let _ = std::process::Command::new("ydotool")
        .args(args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .spawn();
}
