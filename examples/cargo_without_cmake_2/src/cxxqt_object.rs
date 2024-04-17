#[cxx_qt::bridge]
pub mod qobject {
    // ANCHOR_END: book_bridge_macro

    // ANCHOR: book_qstring_import
    unsafe extern "C++Qt" {
        include!("cxx-qt-lib/qstring.h");
        /// An alias to the QString type
        type QString = cxx_qt_lib::QString;

        include!(<QPushButton>);
        #[qobject]
        type QPushButton;

        include!(<QWidget>);
        #[qobject]
        type QWidget;

        #[qsignal]
        fn clicked(self: Pin<&mut QPushButton>, checked: bool);

        fn show(self: Pin<&mut QPushButton>);

        include!(<QMainWindow>);
        #[qobject]
        type QMainWindow;

        fn show(self: Pin<&mut QMainWindow>);
    }

    #[namespace = "rust::cxxqtlib1"]
    unsafe extern "C++" {
        include!("cxx-qt-lib/common.h");

        #[rust_name = "QPushButton_new_ptr"]
        fn new_ptr() -> *mut QPushButton;

        #[rust_name = "QPushButton_new_ptr_with_text"]
        fn new_ptr(text: &QString) -> *mut QPushButton;

        #[rust_name = "QPushButton_new_ptr_with_text_parent"]
        unsafe fn new_ptr(text: &QString, parent: *mut QWidget) -> *mut QPushButton;

        #[rust_name = "QMainWindow_new_ptr"]
        fn new_ptr() -> *mut QMainWindow;
    }
}
