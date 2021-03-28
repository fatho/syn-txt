use std::{cell::RefCell, ops::Deref, rc::Rc};

use yew::prelude::*;

pub mod editor;
pub mod list;

pub struct WeakComponentLink<C: Component>(Rc<RefCell<Option<ComponentLink<C>>>>);

impl<C: Component> Clone for WeakComponentLink<C> {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl<C: Component> Default for WeakComponentLink<C> {
    fn default() -> Self {
        Self(Rc::default())
    }
}

impl<C: Component> Deref for WeakComponentLink<C> {
    type Target = Rc<RefCell<Option<ComponentLink<C>>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<C: Component> PartialEq for WeakComponentLink<C> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl<C: Component> WeakComponentLink<C> {
    pub fn attach(&self, link: ComponentLink<C>) {
        self.borrow_mut().replace(link);
    }

    pub fn detach(&self) {
        self.borrow_mut().take();
    }

    pub fn send_message(&self, msg: C::Message) {
        if let Some(link) = self.borrow().as_ref() {
            link.send_message(msg);
        }
    }
}
