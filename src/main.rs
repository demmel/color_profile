use std::collections::VecDeque;

use css_style::{
  color::{
    named::BLACK,
    palette::{FromColor, Hsl, Srgb},
  },
  prelude::*,
  unit::px,
};
use enum_ordinalize::Ordinalize;
use sycamore::prelude::*;

#[derive(Debug, Clone, Copy, Ordinalize, Hash, PartialEq, Eq)]
enum Color {
  Red,
  Orange,
  Yellow,
  Green,
  Cyan,
  Blue,
  Magenta,
}

impl Color {
  fn label(&self) -> &'static str {
    match self {
      Color::Red => "Red",
      Color::Orange => "Orange",
      Color::Yellow => "Yellow",
      Color::Green => "Green",
      Color::Cyan => "Cyan",
      Color::Blue => "Blue",
      Color::Magenta => "Magenta",
    }
  }

  fn color(&self) -> Srgb<u8> {
    match self {
      Color::Red => Srgb::new(255, 0, 0),
      Color::Orange => Srgb::new(255, 127, 0),
      Color::Yellow => Srgb::new(255, 255, 0),
      Color::Green => Srgb::new(0, 255, 0),
      Color::Cyan => Srgb::new(0, 255, 255),
      Color::Blue => Srgb::new(0, 0, 255),
      Color::Magenta => Srgb::new(255, 0, 255),
    }
  }
}

fn main() {
  let profile: Signal<Vec<(_, Option<Color>)>> =
    Signal::new((0..360).map(|h| (h, None)).collect());
  let color = Signal::new(None);
  let hue = Signal::new(Some((0, (0, 360))));
  let queue = Signal::new(VecDeque::new());

  create_effect(cloned!(profile, hue, queue, color => move || {
    let c = if let Some(c) = *color.get() {
      color.set(None);
      c
    } else {
      return;
    };

    let h = if let Some(h) = *hue.get() {
      h
    } else {
      return;
    };

    let mut q = (*queue.get()).clone();
    let mut p = (*profile.get()).clone();

    p[h.0].1 = Some(c);

    let mut maybe_fill_or_queue = |low: usize, high: usize| {
      let lm = low % 360;
      let hm = high % 360;
      let lv = p[lm].1;
      let hv = p[hm].1;
      if lm != hm && lv.is_some() && lv == hv  {
        for p in p.iter_mut().take(high).skip(low) {
          p.1 = Some(c);
        }
      } else if (high - low) > 1 {
        q.push_back((low,high));
      }
    };

    maybe_fill_or_queue(h.1.0, h.0);
    maybe_fill_or_queue(h.0, h.1.1);

    if let Some((low, high)) = q.pop_front() {
      let mid = (low + high) / 2;
      hue.set(Some((mid, (low,high))));
    } else {
      hue.set(None)
    }

    queue.set(q);
    profile.set(p);
  }));

  sycamore::render(|| {
    view! {
      div(style="display: flex; align-items: center; justify-content: center;") {
        div(style="display: flex; flex-direction: column") {
          h1(style="margin-top: 16px;") { "Color Profile" }
          (cloned!(color =>if let Some(h) = *hue.get() {
            view! {
              h3(style="margin-top: 16px;") { "What color is this?"}
              div(style="
                margin-top: 4px; 
                display: flex; 
                flex-direction: row; 
                border: solid 1px #cccccc;
                background-color: #fdfdfd;
                border-radius: 8px;
                overflow: hidden;
                box-shadow: 0px 2px 4px #bbbbbb;
              ") {
                div(style=(style()
                  .and_size(|c| c.width(px(300)).height(px(300)))
                  .and_background(|c| {
                    c.color(Srgb::from_color(Hsl::new(h.0 as f32, 1.0, 0.5)))
                  })
                  .and_border(|c| c.and_right(|c| c.solid().width(px(1)).color(BLACK)))
                ))
                div {
                  ColorButtons(ColorButtonsProps {
                    color_signal: color
                  })
                }
              }
            }
          } else {
            view!{}
          }))
          h3(style="margin-top: 16px;") { "Your Color Profile"}
          div(style="
            margin-top: 4px; 
            border: solid 1px #cccccc;
            position: relative; 
            width: 300px; 
            height: 300px; 
            background-color: #fdfdfd;
            border-radius: 8px;
            box-shadow: 0px 2px 4px #bbbbbb;
          ") {
            Keyed(KeyedProps {
              iterable: profile.handle(),
              template: |(hue, color)| view! {
                div(style=(format!(
                  "
                    width: 2px; 
                    height: 40px; 
                    position: absolute; 
                    left: 50%; 
                    top: 50%;
                    margin-left: -1px;
                    margin-top: -20px; 
                    transform: rotate({}deg) translate(100px) rotate(90deg); 
                  ",
                  hue as f32,
                ))) {
                  div(style=(style()
                    .and_size(|c| c.width(px(2)).height(px(20)))
                    .and_background(|c| {
                      c.color(Srgb::from_color(Hsl::new(hue as f32, 1.0, 0.5)))
                    })
                  ))
                  div(style=(style()
                    .and_size(|c| c.width(px(2)).height(px(20)))
                    .and_background(|c| {
                      c.color(color
                        .map(|c| c.color())
                        .unwrap_or_else(|| Srgb::new(253, 253, 253))
                      )
                    })
                  ))
                }
              },
              key: |h| *h,
            })
          }
        }
      }
    }
  });
}

struct ColorButtonsProps {
  color_signal: Signal<Option<Color>>,
}

#[component(ColorButtons<G>)]
fn color_buttons(
  ColorButtonsProps { color_signal }: ColorButtonsProps,
) -> View<G> {
  let disabled = color_signal.get().is_some();
  view! {
    div(style=
      "
        position: relative; 
        width: 300px; 
        height: 300px; 
      "
    ) {
      (View::new_fragment(
        Color::variants()
          .into_iter()
          .enumerate()
          .map(|(i, color)| {
            view! {
              button(
                style=(format!(
                  "
                    width: 64px; 
                    height: 24px; 
                    position: absolute; 
                    left: 50%; 
                    top: 50%; 
                    transform: rotate({}deg) translate(100px) rotate(-{}deg); 
                    display: flex; 
                    flex-direction: row;
                    margin-left: -32px;
                    margin-top: -12px; 
                  ",
                  (i as f32 / Color::variant_count() as f32) * 360.0,
                  (i as f32 / Color::variant_count() as f32) * 360.0
                )),
                disabled=disabled,
                on:click=cloned!(color_signal => move |_| {
                  color_signal.set(Some(color))
                })
              ) {
                (color.label())
              }
            }
          })
          .collect(),
      ))
    }
  }
}
