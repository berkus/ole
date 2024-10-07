//             DO WHAT THE FUCK YOU WANT TO PUBLIC LICENSE
//                    Version 2, December 2004
//
// Copyright (C) 2018 Thomas Bailleux <thomas@bailleux.me>
//
// Everyone is permitted to copy and distribute verbatim or modified
// copies of this license document, and changing it is allowed as long
// as the name is changed.
//
//            DO WHAT THE FUCK YOU WANT TO PUBLIC LICENSE
//   TERMS AND CONDITIONS FOR COPYING, DISTRIBUTION AND MODIFICATION
//
//  0. You just DO WHAT THE FUCK YOU WANT TO.
//
// Author: zadig <thomas chr(0x40) bailleux.me>

use thiserror::Error;

/// Errors related to the process of parsing.
#[derive(Debug, Error)]
pub enum Error {
    /// This happens when filesize is null, or to big to fit into an usize.
    #[error("Filesize is null or too big.")]
    BadFileSize,

    /// Classic std::io::Error.
    #[error("IO error {0}")]
    IOError(#[from] std::io::Error),

    /// Something is not implemented yet?
    #[error("Method not implemented yet")]
    NotImplementedYet,

    /// This is not a valid OLE file.
    #[error("Invalid OLE File")]
    InvalidOLEFile,

    /// Something has a bad size.
    #[error("{0}")]
    BadSizeValue(&'static str),

    /// MSAT is empty.
    #[error("MSAT is empty")]
    EmptyMasterSectorAllocationTable,

    /// Malformed SAT.
    #[error("Sector is not a sector used by the SAT.")]
    NotSectorUsedBySAT,

    /// Unknown node type.
    #[error("Unknown node type")]
    NodeTypeUnknown,

    /// Root storage has a bad size.
    #[error("Bad RootStorage size")]
    BadRootStorageSize,

    /// User query an empty entry
    #[error("Empty entry")]
    EmptyEntry,
}
