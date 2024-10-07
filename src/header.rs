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

use crate::constants;
use crate::error::Error;
use culpa::{throw, throws};
use std::io::Read;

impl<'ole> crate::ole::Reader<'ole> {
    #[throws]
    pub(crate) fn parse_header(&mut self) {
        use crate::util::FromSlice;

        // read the header
        let mut header = vec![0u8; constants::HEADER_SIZE];
        self.read(&mut header)?;

        // Check file identifier
        if &constants::IDENTIFIER != &header[0..8] {
            throw!(Error::InvalidOLEFile);
        } else {
            // UID
            self.uid = header[8..24].to_vec();

            // Revision number & version number
            let mut rv_number = usize::from_slice(&header[24..26]);
            self.revision_number = Some(rv_number as u16);
            rv_number = usize::from_slice(&header[26..28]);
            self.version_number = Some(rv_number as u16);

            // Check little-endianness; big endian not yet supported
            if &header[28..30] == &constants::BIG_ENDIAN_IDENTIFIER {
                throw!(Error::NotImplementedYet);
            } else if &header[28..30] != &constants::LITTLE_ENDIAN_IDENTIFIER {
                throw!(Error::InvalidOLEFile);
            } else {
                // Sector size
                let mut k = usize::from_slice(&header[30..32]);

                // if k >= 16, it means that the sector size equals 2 ^ k, which
                // is impossible.
                if k >= 16 {
                    throw!(Error::BadSizeValue("Overflow on sector size"));
                } else {
                    self.sec_size = Some(2usize.pow(k as u32));

                    // Short sector size
                    k = usize::from_slice(&header[32..34]);

                    // same for sector size
                    if k >= 16 {
                        throw!(Error::BadSizeValue("Overflow on short sector size"));
                    } else {
                        self.short_sec_size = Some(2usize.pow(k as u32));

                        // Total number of sectors used for the sector allocation table
                        let sat = Vec::with_capacity(
                            (*self.sec_size.as_ref().unwrap() / 4)
                                * usize::from_slice(&header[44..48]),
                        );

                        // SecID of the first sector of directory stream
                        let mut dsat = vec![];
                        dsat.push(u32::from_slice(&header[48..52]));

                        // Minimum size of a standard stream (bytes)
                        self.minimum_standard_stream_size =
                            Some(usize::from_slice(&header[56..60]));

                        // standard says that this value has to be greater
                        // or equals to 4096
                        if *self.minimum_standard_stream_size.as_ref().unwrap() < 4096usize {
                            throw!(Error::InvalidOLEFile);
                        } else {
                            // secID of the first sector of the SSAT & Total number
                            // of sectors used for the short-sector allocation table
                            let mut ssat = Vec::with_capacity(
                                usize::from_slice(&header[64..68])
                                    * (*self.sec_size.as_ref().unwrap() / 4),
                            );
                            ssat.push(u32::from_slice(&header[60..64]));

                            // secID of first sector of the master sector allocation table
                            // & Total number of sectors used for
                            // the master sector allocation table
                            let mut msat = vec![constants::FREE_SECID_U32; 109];
                            if &header[68..72] != &constants::END_OF_CHAIN_SECID {
                                msat.resize(
                                    109usize
                                        + usize::from_slice(&header[72..76])
                                            * (*self.sec_size.as_ref().unwrap() / 4),
                                    constants::FREE_SECID_U32,
                                );
                            }
                            self.sat = Some(sat);
                            self.msat = Some(msat);
                            self.dsat = Some(dsat);
                            self.ssat = Some(ssat);

                            // now we build the MSAT
                            self.build_master_sector_allocation_table(&header)?;
                        }
                    }
                }
            }
        }
    }

    /// Build the Master Sector Allocation Table (MSAT)
    fn build_master_sector_allocation_table(&mut self, header: &[u8]) -> Result<(), Error> {
        use crate::util::FromSlice;

        // First, we build the master sector allocation table from the header
        let mut total_sec_id_read = self.read_sec_ids(&header[76..], 0);

        // Check if additional sectors are used for building the msat
        if total_sec_id_read == 109 {
            let sec_size = *self.sec_size.as_ref().unwrap();
            let mut sec_id = usize::from_slice(&header[68..72]);
            let mut buffer = vec![0u8; 0];

            while sec_id != constants::END_OF_CHAIN_SECID_U32 as usize {
                let relative_offset = sec_id * sec_size;

                // check if we need to read more data
                if buffer.len() <= relative_offset + sec_size {
                    let new_len = (sec_id + 1) * sec_size;
                    buffer.resize(new_len, 0xFFu8);
                    self.read(&mut buffer[relative_offset..relative_offset + sec_size])?;
                }
                total_sec_id_read += self.read_sec_ids(
                    &buffer[relative_offset..relative_offset + sec_size - 4],
                    total_sec_id_read,
                );
                sec_id = usize::from_slice(&buffer[buffer.len() - 4..]);
            }
            // save the buffer for later usage
            self.body = Some(buffer);
        }
        self.msat
            .as_mut()
            .unwrap()
            .resize(total_sec_id_read, constants::FREE_SECID_U32);

        // Now, we read the all file
        let mut buf: &mut std::vec::Vec<u8>;
        if !self.body.is_some() {
            self.body = Some(std::vec::Vec::new());
        }
        buf = self.body.as_mut().unwrap();

        self.buf_reader
            .as_mut()
            .unwrap()
            .read_to_end(&mut buf)
            .map_err(Error::IOError)?;
        Ok(())
    }

    fn read_sec_ids(&mut self, buffer: &[u8], msat_offset: usize) -> usize {
        use crate::util::FromSlice;
        let mut i = 0usize;
        let mut offset = 0usize;
        let max_sec_ids = buffer.len() / 4;
        let msat = &mut self.msat.as_mut().unwrap()[msat_offset..];
        while i < max_sec_ids && &buffer[offset..offset + 4] != &constants::FREE_SECID {
            msat[i] = u32::from_slice(&buffer[offset..offset + 4]);
            offset += 4;
            i += 1;
        }

        i
    }
}
