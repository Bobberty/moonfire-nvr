// This file is part of Moonfire NVR, a security camera network video recorder.
// Copyright (C) 2016 The Moonfire NVR Authors; see AUTHORS and LICENSE.txt.
// SPDX-License-Identifier: GPL-v3.0-or-later WITH GPL-3.0-linking-exception.

use db::dir;
use failure::{Error, Fail};
use log::info;
use nix::fcntl::FlockArg;
use std::path::Path;

pub mod check;
pub mod config;
pub mod init;
pub mod login;
pub mod run;
pub mod sql;
pub mod ts;
pub mod upgrade;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum OpenMode {
    ReadOnly,
    ReadWrite,
    Create,
}

/// Locks the directory without opening the database.
/// The returned `dir::Fd` holds the lock and should be kept open as long as the `Connection` is.
fn open_dir(db_dir: &Path, mode: OpenMode) -> Result<dir::Fd, Error> {
    let dir = dir::Fd::open(db_dir, mode == OpenMode::Create).map_err(|e| {
        e.context(if e == nix::Error::ENOENT {
            format!(
                "db dir {} not found; try running moonfire-nvr init",
                db_dir.display()
            )
        } else {
            format!("unable to open db dir {}", db_dir.display())
        })
    })?;
    let ro = mode == OpenMode::ReadOnly;
    dir.lock(if ro {
        FlockArg::LockSharedNonblock
    } else {
        FlockArg::LockExclusiveNonblock
    })
    .map_err(|e| {
        e.context(format!(
            "unable to get {} lock on db dir {} ",
            if ro { "shared" } else { "exclusive" },
            db_dir.display()
        ))
    })?;
    Ok(dir)
}

/// Locks and opens the database.
/// The returned `dir::Fd` holds the lock and should be kept open as long as the `Connection` is.
fn open_conn(db_dir: &Path, mode: OpenMode) -> Result<(dir::Fd, rusqlite::Connection), Error> {
    let dir = open_dir(db_dir, mode)?;
    let db_path = db_dir.join("db");
    info!(
        "Opening {} in {:?} mode with SQLite version {}",
        db_path.display(),
        mode,
        rusqlite::version()
    );
    let conn = rusqlite::Connection::open_with_flags(
        db_path,
        match mode {
            OpenMode::ReadOnly => rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
            OpenMode::ReadWrite => rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE,
            OpenMode::Create => {
                rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE | rusqlite::OpenFlags::SQLITE_OPEN_CREATE
            },
        } |
        // rusqlite::Connection is not Sync, so there's no reason to tell SQLite3 to use the
        // serialized threading mode.
        rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;
    Ok((dir, conn))
}
