use assets::Assets;
use gpui::*;
use settings::{default_settings, Settings, SettingsStore};
use theme::{ThemeRegistry, ThemeSettings};

struct HelloWorld {
    text: SharedString,
}

impl Render for HelloWorld {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .flex()
            .bg(cx.theme().colors().background)
            .size_full()
            .justify_center()
            .items_center()
            .text_xl()
            .text_color(cx.theme().colors().text)
            .child(format!("Hello, {}!", &self.text))
    }
}

fn main() {
    App::new().run(|cx: &mut AppContext| {
        let theme_name: String = "One Dark".to_string();

        // let mut store = SettingsStore();
        let mut store = SettingsStore::new(cx);
        store
            .set_default_settings(default_settings().as_ref(), cx)
            .unwrap();
        cx.set_global(store);

        // settings::init(cx);
        theme::init(theme::LoadThemes::All(Box::new(Assets)), cx);

        let theme_registry: &ThemeRegistry = cx.global::<ThemeRegistry>();
        let mut theme_settings: ThemeSettings = ThemeSettings::get_global(cx).clone();
        theme_settings.active_theme = theme_registry.get(&theme_name).unwrap();
        ThemeSettings::override_global(theme_settings, cx);

        cx.open_window(WindowOptions::default(), |cx| {
            cx.new_view(|_cx| HelloWorld {
                text: "World".into(),
            })
        })
        .unwrap();
    });
}
