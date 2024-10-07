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

impl<'ole> crate::ole::Reader<'ole> {
    pub(crate) fn build_sat(&mut self) -> Result<(), Error> {
        let sector_size = self.sec_size.unwrap();
        let mut sec_ids = vec![crate::constants::FREE_SECID_U32; sector_size / 4];
        if self.msat.as_ref().unwrap().len() == 0 {
            Err(Error::EmptyMasterSectorAllocationTable)
        } else {
            for i in 0..self.msat.as_ref().unwrap().len() {
                let sector_index = self.msat.as_ref().unwrap()[i];
                self.read_sat_sector(sector_index as usize, &mut sec_ids)?;
                self.sat.as_mut().unwrap().extend_from_slice(&sec_ids);
            }
            self.build_ssat()?;
            self.build_dsat()?;
            Ok(())
        }
    }

    pub(crate) fn read_sat_sector(
        &mut self,
        sector_index: usize,
        sec_ids: &mut Vec<u32>,
    ) -> Result<(), Error> {
        let sector = self.read_sector(sector_index)?;
        use crate::util::FromSlice;
        for i in 0..sec_ids.capacity() {
            sec_ids[i] = u32::from_slice(&sector[i * 4..i * 4 + 4]);
        }

        Ok(())
    }

    pub(crate) fn build_chain_from_sat(&mut self, start: u32) -> Vec<u32> {
        let mut chain = vec![];
        let mut sector_index = start;
        let sat = self.sat.as_mut().unwrap();
        while sector_index != constants::END_OF_CHAIN_SECID_U32 {
            chain.push(sector_index);
            sector_index = sat[sector_index as usize];
        }

        chain
    }

    pub(crate) fn build_chain_from_ssat(&mut self, start: u32) -> Vec<u32> {
        let mut chain = vec![];
        let mut sector_index = start;
        let sat = self.ssat.as_mut().unwrap();
        while sector_index != constants::END_OF_CHAIN_SECID_U32
            && sector_index != constants::FREE_SECID_U32
        {
            chain.push(sector_index);

            sector_index = sat[sector_index as usize];
        }

        chain
    }

    pub(crate) fn build_ssat(&mut self) -> Result<(), Error> {
        let mut sec_ids = vec![constants::FREE_SECID_U32; self.sec_size.as_ref().unwrap() / 4];

        let sector_index = self.ssat.as_mut().unwrap().remove(0);
        let chain = self.build_chain_from_sat(sector_index);

        for sector_index in chain {
            self.read_sat_sector(sector_index as usize, &mut sec_ids)?;
            self.ssat.as_mut().unwrap().extend_from_slice(&sec_ids);
        }
        Ok(())
    }

    pub(crate) fn build_dsat(&mut self) -> Result<(), Error> {
        let sector_index = self.dsat.as_mut().unwrap().remove(0);
        let chain = self.build_chain_from_sat(sector_index);

        for sector_index in chain {
            self.dsat.as_mut().unwrap().push(sector_index);
        }

        Ok(())
    }
}
