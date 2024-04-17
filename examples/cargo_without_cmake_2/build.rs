// SPDX-FileCopyrightText: 2022 Klarälvdalens Datakonsult AB, a KDAB Group company <info@kdab.com>
// SPDX-FileContributor: Be Wilson <be.wilson@kdab.com>
//
// SPDX-License-Identifier: MIT OR Apache-2.0

// ANCHOR: book_cargo_executable_build_rs
use cxx_qt_build::{CxxQtBuilder};

fn main() {
    CxxQtBuilder::new()
        // Link Qt's Network library
        // - Qt Core is always linked
        // - Qt Gui is linked by enabling the qt_gui Cargo feature (default).
        // - Qt Qml is linked by enabling the qt_qml Cargo feature (default).
        // - Qt Qml requires linking Qt Network on macOS
        .qt_module("Network")
        .qt_module("Widgets")
        // .qml_module(QmlModule {
        //     uri: "com.kdab.cxx_qt.demo",
        //     rust_files: &["src/cxxqt_object.rs"],
        //     qml_files: &["qml/main.qml"],
        //     ..Default::default()
        // })
        .file("src/cxxqt_object.rs")
        .with_opts(cxx_qt_lib_headers::build_opts())
        .build();
}
// ANCHOR_END: book_cargo_executable_build_rs
