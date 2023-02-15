use gtk::prelude::*;
use std::collections::VecDeque;

pub struct WidgetChildrenIter(Option<gtk::Widget>);

impl Iterator for WidgetChildrenIter {
    type Item = gtk::Widget;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.0.take()?;
        self.0 = current.next_sibling();
        Some(current)
    }
}

pub struct WidgetChildrenRevIter(Option<gtk::Widget>);

impl Iterator for WidgetChildrenRevIter {
    type Item = gtk::Widget;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.0.take()?;
        self.0 = current.prev_sibling();
        Some(current)
    }
}

pub struct WidgetTraverseDepthFirstIter {
    widgets: VecDeque<gtk::Widget>,
}

impl WidgetTraverseDepthFirstIter {
    pub fn new(widget: gtk::Widget) -> Self {
        let mut widgets = VecDeque::new();
        widgets.push_front(widget);
        Self { widgets }
    }
}

impl Iterator for WidgetTraverseDepthFirstIter {
    type Item = gtk::Widget;

    fn next(&mut self) -> Option<Self::Item> {
        let front = self.widgets.pop_front();
        if let Some(ref container) = front {
            for child in container.children_rev() {
                self.widgets.push_front(child);
            }
        }
        front
    }
}

pub trait AwesomeWidgetTraverseExt {
    fn children(&self) -> WidgetChildrenIter;
    fn children_rev(&self) -> WidgetChildrenRevIter;
    fn traverse(&self) -> WidgetTraverseDepthFirstIter;
}

impl<P: IsA<gtk::Widget>> AwesomeWidgetTraverseExt for P {
    fn children(&self) -> WidgetChildrenIter {
        WidgetChildrenIter(self.first_child())
    }

    fn children_rev(&self) -> WidgetChildrenRevIter {
        WidgetChildrenRevIter(self.last_child())
    }

    fn traverse(&self) -> WidgetTraverseDepthFirstIter {
        WidgetTraverseDepthFirstIter::new(self.clone().upcast())
    }
}
