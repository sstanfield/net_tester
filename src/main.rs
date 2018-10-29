extern crate glib;
extern crate gdk;
extern crate gtk;
#[macro_use]
extern crate relm;
#[macro_use]
extern crate relm_derive;
extern crate net_tester;

use gtk::{
    Button,
    ButtonExt,
    ContainerExt,
    Inhibit,
    Label,
    LabelExt,
    WidgetExt,
    Window,
    WindowType,
    TextView,
    TextViewExt,
    TextBuffer,
    TextBufferExt,
    Grid,
    GridExt,
    CssProvider,
    CssProviderExt,
    StyleContext,
    StyleContextExt,
    ScrolledWindow,
    ScrolledWindowExt,
};
use relm::{Relm, Update, Widget};
use std::thread;

use net_tester::Status;
use net_tester::test_network;

struct Model {
    counter: i32,
    text: Vec<String>,
}

#[derive(Msg)]
enum Msg {
    Message(Status),
    Decrement,
    Increment,
    Quit,
}

// Create the structure that holds the widgets used in the view.
#[derive(Clone)]
struct Widgets {
    text: TextView,
    text_buffer: TextBuffer,
    counter_label: Label,
    minus_button: Button,
    plus_button: Button,
    window: Window,
}

struct Win {
    model: Model,
    widgets: Widgets,
}

impl Update for Win {
    // Specify the model used for this widget.
    type Model = Model;
    // Specify the model parameter used to init the model.
    type ModelParam = ();
    // Specify the type of the messages sent to the update function.
    type Msg = Msg;

    fn model(_: &Relm<Self>, _: ()) -> Model {
        Model {
            counter: 0,
            text: Vec::new(),
        }
    }

    fn update(&mut self, event: Msg) {
        let label = &self.widgets.counter_label;
        let text = &self.widgets.text;
        let text_buffer = &self.widgets.text_buffer;

        match event {
            Msg::Message(status) => {
                match status {
                    Status::Working(message) => {
                        text_buffer.insert_at_cursor(&message);
                        text.get_style_context().unwrap().remove_class("red-background");
                        text.get_style_context().unwrap().remove_class("green-background");
                        text.get_style_context().unwrap().add_class("yellow-background");
                    },
                    Status::Error(message) => {
                        text_buffer.insert_at_cursor(&message);
                        text.get_style_context().unwrap().remove_class("yellow-background");
                        text.get_style_context().unwrap().remove_class("green-background");
                        text.get_style_context().unwrap().add_class("red-background");
                    },
                    Status::Good(message) => {
                        text_buffer.insert_at_cursor(&message);
                        text.get_style_context().unwrap().remove_class("yellow-background");
                        text.get_style_context().unwrap().remove_class("red-background");
                        text.get_style_context().unwrap().add_class("green-background");
                    },
                }
                text_buffer.insert_at_cursor("\n");
                text.scroll_mark_onscreen(&text_buffer.get_insert().unwrap());
            }
            Msg::Decrement => {
                self.model.counter -= 1;
                self.model.text.push("Decrement".to_string());
                //self.model.text = self.model.text.clone() + "\nDecrement";
                // Manually update the view.
                label.set_text(&self.model.counter.to_string());
                //text_buffer.set_text(&self.model.text.join("\n"));
                text_buffer.insert_at_cursor("Decrement XXXXXX\n");
                text.scroll_mark_onscreen(&text_buffer.get_insert().unwrap());
            },
            Msg::Increment => {
                //self.model.text = self.model.text.clone() + "\nIncrement";
                self.model.text.push("Increment".to_string());
                self.model.counter += 1;
                label.set_text(&self.model.counter.to_string());
                //text_buffer.set_text(&self.model.text.join("\n"));
                text_buffer.insert_at_cursor("Increment XXXXXX\n");
                text.scroll_mark_onscreen(&text_buffer.get_insert().unwrap());
            },
            Msg::Quit => gtk::main_quit(),
        }
    }
}

impl Widget for Win {
    // Specify the type of the root widget.
    type Root = Window;

    // Return the root widget.
    fn root(&self) -> Self::Root {
        self.widgets.window.clone()
    }

    fn view(relm: &Relm<Self>, model: Self::Model) -> Self {
        let screen = gdk::Screen::get_default().unwrap();
        let provider = CssProvider::new();
        let css_data = r#"
        .red-background { background-image: none; background-color: red;  font-size: 32px }
        .red-background text { background-image: none; background-color: red;  color: rgb(255,255,0); font-size: 32px }
        .yellow-background { background-image: none; background-color: rgb(255, 255, 0); font-size: 32px }
        .yellow-background text { background-image: none; background-color: rgb(255, 255, 0); font-size: 32px }
        .green-background { background-image: none; background-color: rgb(0, 255, 0); font-size: 32px  }
        .green-background text { background-image: none; background-color: rgb(0, 255, 0); font-size: 32px  }
        "#.as_bytes();
        provider.load_from_data(css_data);
        StyleContext::add_provider_for_screen(&screen, &provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);


        // Create the view using the normal GTK+ method calls.
        let grid = Grid::new();
        grid.set_column_homogeneous(true);

        let text = gtk::TextView::new();
        let text_buffer = text.get_buffer().unwrap();
        text.set_editable(false);
        text.set_cursor_visible(false);
        text.set_wrap_mode(gtk::WrapMode::Word);
        let scroll = ScrolledWindow::new(None, None);
        scroll.set_policy (gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);
        scroll.add(&text);
        grid.attach(&scroll, 0, 1, 3, 1);
        text.get_style_context().unwrap().add_class("yellow-background");
        text.set_hexpand(true);
        text.set_halign(gtk::Align::Fill);
        text.set_vexpand(true);
        text.set_valign(gtk::Align::Fill);

        let plus_button = Button::new_with_label("+");
        grid.attach(&plus_button, 0, 0, 1, 1);

        let counter_label = Label::new("0");
        grid.attach(&counter_label, 1, 0, 1, 1);

        let minus_button = Button::new_with_label("-");
        grid.attach(&minus_button, 2, 0, 1, 1);

        let window = Window::new(WindowType::Toplevel);

        window.add(&grid);

        window.show_all();

        // Send the message Increment when the button is clicked.
        connect!(relm, plus_button, connect_clicked(_), Msg::Increment);
        connect!(relm, minus_button, connect_clicked(_), Msg::Decrement);
        connect!(relm, window, connect_delete_event(_, _), return (Some(Msg::Quit), Inhibit(false)));

        Win {
            model,
            widgets: Widgets {
                text,
                text_buffer,
                counter_label,
                minus_button,
                plus_button,
                window,
            },
        }
    }
}

fn main() {
    let ifname = std::env::args().skip(1).next().unwrap();
    let (tx, rx) = std::sync::mpsc::channel();
    gtk::init().map_err(|_| ()).unwrap();
    let win = relm::init::<Win>(()).unwrap();
    thread::spawn(move || {
        test_network(&ifname, tx);
    });
    let stream = win.stream().clone();
    glib::source::idle_add(move || {
        match rx.try_recv() {
            Ok(mes) => {
                stream.emit(Msg::Message(mes.clone()));
            },
            _ => {}
        }
        glib::source::Continue(true)
    });
    //Win::run(()).expect("Win::run failed");
    gtk::main();
}

