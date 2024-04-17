
pub mod cxxqt_object;

use std::pin::Pin;
use cxx_qt_lib_extras::QApplication;
use crate::cxxqt_object::qobject::{QPushButton_new_ptr_with_text_parent, QMainWindow_new_ptr, QString};

fn main() {
    // Create the application and engine
    let mut app = QApplication::new();

    let mut main_window = QMainWindow_new_ptr();

    let mut button = unsafe { QPushButton_new_ptr_with_text_parent(&QString::from("test"), main_window as *mut _) };
    let mut button = unsafe {Pin::new_unchecked(&mut *button)};
    button.as_mut().on_clicked(|btn, status| println!("button pressed! {status}"));
    // button.show();

    unsafe {Pin::new_unchecked(&mut *main_window)}.show();

    // Start the app
    if let Some(app) = app.as_mut() {
        app.exec();
    }
}
