// This file is part of Moonfire NVR, a security camera network video recorder.
// Copyright (C) 2017 The Moonfire NVR Authors; see AUTHORS and LICENSE.txt.
// SPDX-License-Identifier: GPL-v3.0-or-later WITH GPL-3.0-linking-exception.

//! Text-based configuration interface.
//!
//! This code is a bit messy, but it's essentially a prototype. Eventually Moonfire NVR's
//! configuration will likely be almost entirely done through a web-based UI.

use base::clock;
use cursive::views;
use cursive::Cursive;
use failure::Error;
use std::path::PathBuf;
use std::sync::Arc;
use structopt::StructOpt;

mod cameras;
mod dirs;
mod users;

#[derive(StructOpt)]
pub struct Args {
    /// Directory holding the SQLite3 index database.
    #[structopt(
        long,
        default_value = "/var/lib/moonfire-nvr/db",
        value_name = "path",
        parse(from_os_str)
    )]
    db_dir: PathBuf,
}

pub fn run(args: &Args) -> Result<i32, Error> {
    let (_db_dir, conn) = super::open_conn(&args.db_dir, super::OpenMode::ReadWrite)?;
    let clocks = clock::RealClocks {};
    let db = Arc::new(db::Database::new(clocks, conn, true)?);

    let mut siv = cursive::default();
    //siv.add_global_callback('q', |s| s.quit());

    siv.add_layer(
        views::Dialog::around(
            views::SelectView::<fn(&Arc<db::Database>, &mut Cursive)>::new()
                .on_submit(move |siv, item| item(&db, siv))
                .item("Cameras and streams".to_string(), cameras::top_dialog)
                .item("Directories and retention".to_string(), dirs::top_dialog)
                .item("Users".to_string(), users::top_dialog),
        )
        .button("Quit", |siv| siv.quit())
        .title("Main menu"),
    );

    siv.run();

    Ok(0)
}
