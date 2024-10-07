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
use crate::error::Error;

impl<'ole> crate::ole::Reader<'ole> {
    pub(crate) fn read_sector(&self, sector_index: usize) -> Result<&[u8], Error> {
        let sector_size = self.sec_size.unwrap();
        let offset = sector_size * sector_index;
        let max_size = offset + sector_size;

        let body_size = if self.body.is_some() {
            self.body.as_ref().unwrap().len()
        } else {
            0
        };

        // Check if the sector has already been read
        if body_size >= max_size {
            let body = self.body.as_ref().unwrap();
            Ok(&body[offset..offset + sector_size])
        } else {
            Err(Error::BadSizeValue("File is too short"))
        }
    }
}
