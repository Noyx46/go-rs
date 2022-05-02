use yew::prelude::*;

pub fn make_sidebar(children: Html, shown: bool, hide_callback: Callback<MouseEvent>) -> Html {
    let mut sidebar_classes = classes!("sidebar");
    if !shown {
        sidebar_classes.push("hide-sidebar");
    }

    let mut back_classes = classes!("sd-back");
    if !shown {
        back_classes.push("hide-sidebar");
    }

    html! {
        <>
            <div class={ sidebar_classes }>
                { children }
            </div>
            <div class={ back_classes } onclick={ hide_callback }>
            </div>
        </>
    }
}
