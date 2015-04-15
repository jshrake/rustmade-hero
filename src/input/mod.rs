use std::default::Default;

/*During a given frame, pressing/releasing a button
produces the following signal:

Up   --  ---    ---
           |   |   |     
Down --     ---     ---

This signal is encoded by keeping track of the number
of times the button transitions from up to down and the
final state of the button. In the above example,
half_transition_count = 3 and is_down = true.
*/
#[derive(Clone, Copy, Debug, Default)]
pub struct ButtonState {
  pub half_transition_count: i32,
  pub is_down: bool
}

#[derive(Clone, Copy, Debug, Default)]
pub struct StickInterval {
  pub min: f32,
  pub max: f32,
  pub start: f32,
  pub stop: f32
}

#[derive(Clone, Copy, Debug, Default)]
pub struct StickState {
  pub x: StickInterval,
  pub y: StickInterval
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ControllerInput {
  pub stick: StickState,
  pub a_button: ButtonState,
  pub b_button: ButtonState,
  pub x_button: ButtonState,
  pub y_button: ButtonState,
  pub left_shoulder: ButtonState,
  pub right_shoulder: ButtonState
}

#[derive(Clone, Copy, Debug)]
pub struct GameInput {
  pub controllers: [ControllerInput; 4]
}

impl Default for GameInput {
  fn default() -> GameInput {
    let cid = ControllerInput::default();
    GameInput{
      controllers: [cid, cid, cid, cid]
    }
  }
}