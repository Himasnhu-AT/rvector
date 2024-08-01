use assets::Assets;
use gpui::*;
use settings::Settings;
use theme::{ThemeRegistry, ThemeSettings};
use ui::prelude::*;
use ui::Button;

// ignore unused variables
#[allow(unused_variables)]

struct CountNumber {
    count: usize,
}

impl Render for CountNumber {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let button = Button::new("test-id", "click me")
            .style(ui::ButtonStyle::Filled)
            .size(ui::ButtonSize::Large)
            .on_click(cx.listener(
                move |this: &mut CountNumber,
                      _selection: &ClickEvent,
                      _cx: &mut ViewContext<CountNumber>| {
                    this.count += 1;
                },
            ));

        div()
            .flex()
            .flex_col()
            .bg(rgb(0x282c34))
            .text_color(rgb(0xffffff))
            // .text(rgb(0xffffff))
            .size_full()
            .justify_center()
            .items_center()
            .text_xl()
            .child(format!("Pressed {}! times", &self.count))
            .child(button)
    }
}

fn main() {
    App::new().run(|cx: &mut AppContext| {
        let theme_name: String = "One Dark".to_string();

        settings::init(cx);
        theme::init(theme::LoadThemes::All(Box::new(Assets)), cx);

        let theme_registry = ThemeRegistry::global(cx);
        let mut theme_settings = ThemeSettings::get_global(cx).clone();
        theme_settings.active_theme = theme_registry.get(&theme_name).unwrap();
        ThemeSettings::override_global(theme_settings, cx);

        cx.open_window(WindowOptions::default(), |cx| {
            cx.new_view(|_cx| CountNumber { count: 0 })
        })
        .unwrap();
    });
}
