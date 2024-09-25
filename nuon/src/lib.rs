pub mod debug_ui;

pub use euclid;

pub type Point = euclid::default::Point2D<f32>;
pub type Size = euclid::default::Size2D<f32>;
pub type Box2D = euclid::default::Box2D<f32>;
pub type Rect = euclid::default::Rect<f32>;

pub use elements_map::{Element, ElementBuilder, ElementId, ElementsMap};
mod elements_map;

pub use row_layout::{RowItem, RowLayout};
mod row_layout;

pub use tri_row_layout::TriRowLayout;
mod tri_row_layout;

mod stuff {
    use std::{any::Any, cell::RefCell, marker::PhantomData};

    #[derive(Default)]
    struct Elements {
        elements: Vec<RefCell<Box<dyn erased::ComponentErased>>>,
    }

    impl Elements {
        fn add_element<C: Component>(&mut self, element: C) -> Proxy<C::Message> {
            let id = self.elements.len();
            self.elements.push(RefCell::new(Box::new(element)));
            Proxy::new(id)
        }

        fn on_click(&mut self, id: usize) {
            let element = &self.elements[id];

            let msg = element.borrow_mut().on_click();

            if let Some(msg) = msg {
                self.elements[msg.target].borrow_mut().update(msg.msg);
            }
        }
    }

    pub struct Proxy<MSG> {
        target: usize,
        _ph: PhantomData<MSG>,
    }

    impl<MSG> Copy for Proxy<MSG> {}
    impl<MSG> Clone for Proxy<MSG> {
        fn clone(&self) -> Self {
            *self
        }
    }

    impl<MSG> Proxy<MSG> {
        fn new(target: usize) -> Self {
            Self {
                target,
                _ph: PhantomData,
            }
        }
    }

    #[derive(Debug, Clone)]
    struct ElementMessage<MSG> {
        target: usize,
        msg: MSG,
    }

    #[derive(Debug)]
    struct ElementMessageErased {
        target: usize,
        msg: Box<dyn Any>,
    }

    impl<MSG: 'static> From<ElementMessage<MSG>> for ElementMessageErased {
        fn from(value: ElementMessage<MSG>) -> Self {
            Self {
                target: value.target,
                msg: Box::new(value.msg),
            }
        }
    }

    #[derive(Debug, Clone)]
    enum AppMessage {
        CounterSave(i32),
        Add,
        Sub,
        Exit,
    }

    mod erased {
        use std::any::Any;

        use super::*;

        pub trait ComponentErased {
            fn on_click(&mut self) -> Option<ElementMessageErased>;
            fn update(&mut self, msg: Box<dyn Any>);
        }

        impl<T: Component> ComponentErased for T {
            fn on_click(&mut self) -> Option<ElementMessageErased> {
                Component::on_click(self)
            }
            fn update(&mut self, msg: Box<dyn Any>) {
                Component::update(self, *msg.downcast().unwrap());
            }
        }
    }

    trait Component: 'static {
        type Message: Any;

        fn on_click(&mut self) -> Option<ElementMessageErased> {
            None
        }
        fn update(&mut self, _msg: Self::Message) -> Option<ElementMessageErased>;
    }

    struct App;

    impl App {
        pub fn new(elements: &mut Elements) -> Proxy<AppMessage> {
            elements.add_element(App)
        }
    }

    impl Component for App {
        type Message = AppMessage;

        fn update(&mut self, msg: Self::Message) -> Option<ElementMessageErased> {
            dbg!(msg);
            None
        }
    }

    mod button {
        use std::marker::PhantomData;

        use super::*;

        #[derive(Debug)]
        pub enum ButtonMessage {
            Click,
        }

        pub struct Button<MSG> {
            on_click: Option<(Proxy<MSG>, Box<dyn Fn() -> MSG>)>,
            _msg: PhantomData<MSG>,
        }

        impl<MSG: Any> Button<MSG> {
            pub fn new() -> Self {
                Self {
                    on_click: None,
                    _msg: PhantomData,
                }
            }

            pub fn on_click(mut self, proxy: Proxy<MSG>, msg: impl Fn() -> MSG + 'static) -> Self {
                self.on_click = Some((proxy, Box::new(msg)));
                self
            }

            pub fn build(self, elements: &mut Elements) -> Proxy<ButtonMessage> {
                elements.add_element(self)
            }
        }

        impl<MSG: Any> Component for Button<MSG> {
            type Message = ButtonMessage;

            fn on_click(&mut self) -> Option<ElementMessageErased> {
                self.update(ButtonMessage::Click)
            }

            fn update(&mut self, msg: Self::Message) -> Option<ElementMessageErased> {
                match msg {
                    ButtonMessage::Click => {
                        self.on_click
                            .as_ref()
                            .map(|(proxy, f)| ElementMessageErased {
                                target: proxy.target,
                                msg: Box::new(f()),
                            })
                    }
                }
            }
        }
    }

    mod counter {
        use std::marker::PhantomData;

        use super::*;

        #[derive(Debug)]
        pub enum CounterMessage {
            Add,
            Sub,
            Save,
        }

        pub struct Counter<MSG> {
            state: i32,
            on_save: Option<(Proxy<MSG>, Box<dyn Fn(i32) -> MSG>)>,
            _msg: PhantomData<MSG>,
        }

        impl<MSG: Any> Counter<MSG> {
            pub fn new() -> Self {
                Self {
                    state: 0,
                    on_save: None,
                    _msg: PhantomData,
                }
            }

            pub fn on_save(
                mut self,
                proxy: Proxy<MSG>,
                msg: impl Fn(i32) -> MSG + 'static,
            ) -> Self {
                self.on_save = Some((proxy, Box::new(msg)));
                self
            }

            pub fn build(self, elements: &mut Elements) -> Proxy<CounterMessage> {
                let counter = elements.add_element(self);

                button::Button::new()
                    .on_click(counter, || CounterMessage::Add)
                    .build(elements);

                button::Button::new()
                    .on_click(counter, || CounterMessage::Sub)
                    .build(elements);

                counter
            }
        }

        impl<MSG: Any> Component for Counter<MSG> {
            type Message = CounterMessage;

            fn on_click(&mut self) -> Option<ElementMessageErased> {
                self.update(CounterMessage::Save)
            }

            fn update(&mut self, msg: Self::Message) -> Option<ElementMessageErased> {
                match msg {
                    CounterMessage::Add => self.state += 1,
                    CounterMessage::Sub => self.state -= 1,
                    CounterMessage::Save => {
                        return self
                            .on_save
                            .as_ref()
                            .map(|(proxy, f)| ElementMessageErased {
                                target: proxy.target,
                                msg: Box::new(f(self.state)),
                            })
                    }
                }

                None
            }
        }
    }

    #[test]
    fn cba() {
        let mut elements = Elements::default();

        let app = App::new(&mut elements);

        let _counter = counter::Counter::new()
            .on_save(app, AppMessage::CounterSave)
            .build(&mut elements);

        elements.on_click(2);
        elements.on_click(2);
        elements.on_click(1);
        elements.on_click(3);
        elements.on_click(1);
    }
}
