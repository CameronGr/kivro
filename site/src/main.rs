mod app;
mod content;
mod nav;
mod pages;
mod shell;
mod ui;

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(app::App);
}
