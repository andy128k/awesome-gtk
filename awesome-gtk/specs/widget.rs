use awesome_gtk::prelude::*;

pub fn test_iter() {
    let widget = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .build();

    let ch1 = gtk::Label::new("child #1".into());
    widget.append(&ch1);

    let ch2 = gtk::Label::new("child #2".into());
    widget.append(&ch2);

    let ch3 = gtk::Label::new("child #3".into());
    widget.append(&ch3);

    let ch4 = gtk::Label::new("child #4".into());
    widget.append(&ch4);

    let mut iter = widget.children();
    assert_eq!(iter.next().as_ref(), Some(ch1.upcast_ref()));
    assert_eq!(iter.next().as_ref(), Some(ch2.upcast_ref()));
    assert_eq!(iter.next().as_ref(), Some(ch3.upcast_ref()));
    assert_eq!(iter.next().as_ref(), Some(ch4.upcast_ref()));
    assert_eq!(iter.next(), None);
}

pub fn test_iter_rev() {
    let widget = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .build();

    let ch1 = gtk::Label::new("child #1".into());
    widget.append(&ch1);

    let ch2 = gtk::Label::new("child #2".into());
    widget.append(&ch2);

    let ch3 = gtk::Label::new("child #3".into());
    widget.append(&ch3);

    let ch4 = gtk::Label::new("child #4".into());
    widget.append(&ch4);

    let mut iter = widget.children_rev();
    assert_eq!(iter.next().as_ref(), Some(ch4.upcast_ref()));
    assert_eq!(iter.next().as_ref(), Some(ch3.upcast_ref()));
    assert_eq!(iter.next().as_ref(), Some(ch2.upcast_ref()));
    assert_eq!(iter.next().as_ref(), Some(ch1.upcast_ref()));
    assert_eq!(iter.next(), None);
}

pub fn test_traverse() {
    let widget: gtk::Widget = gtk::Builder::from_string(
        r#"<interface>
            <object class="GtkBox" id="root">
                <property name="name">root-box</property>
                <child>
                    <object class="GtkLabel">
                        <property name="name">ch1</property>
                        <property name="label">ch1</property>
                    </object>
                </child>
                <child>
                    <object class="GtkBox">
                        <property name="name">ch2</property>
                        <child>
                            <object class="GtkLabel">
                                <property name="name">ch21</property>
                                <property name="label">ch21</property>
                            </object>
                        </child>
                        <child>
                            <object class="GtkLabel">
                                <property name="name">ch22</property>
                                <property name="label">ch22</property>
                            </object>
                        </child>
                    </object>
                </child>
                <child>
                    <object class="GtkLabel">
                        <property name="name">ch3</property>
                        <property name="label">ch3</property>
                    </object>
                </child>
            </object>
        </interface>"#,
    )
    .object("root")
    .unwrap();

    let names: Vec<_> = widget.traverse().map(|w| w.widget_name()).collect();
    assert_eq!(names, vec!["root-box", "ch1", "ch2", "ch21", "ch22", "ch3"]);
}
