use crate::{CMachine, CMACHINE, machine};
use crate::lib::constants;

/*
impl CMachineThreadGaps for CMachine {

    //old_name_was getCoinbaseImportGap
    fn get_coinbase_import_gap(&self) -> TimeBySecT
    {
        let mut gap_by_seconds: TimeBySecT;
        if constants::TIME_GAIN == 1 {
            // live mode
            if machine().is_in_sync_process(false)
            {
                gap_by_seconds = 5; // every 1 second controls
                if constants::DATABASAE_AGENT == "sqlite"
                {
                    // TODO: remove this block(variable/mechanism) after fixing sqlite database lock problem
                    if machine().m_recorded_blocks_in_db < 500 {
                        gap_by_seconds = 1; // every 3 second check parsing q
                    } else if machine().m_recorded_blocks_in_db < 1000 {
                        gap_by_seconds = 1;
                    } else if machine().m_recorded_blocks_in_db < 1500 {
                        gap_by_seconds = 1;
                    } else if machine().m_recorded_blocks_in_db < 2000 {
                        gap_by_seconds = 2;
                    } else {
                        gap_by_seconds = 15;
                    }
                }
            } else {
                // TODO: improve it in order to reduce gap if we are around midnight or midday, and increase gap if for current block the coinbase already created and the older coinbase already imported in UTXOs
                gap_by_seconds = 1200; // every 20 minute
            }
        } else {
            // develope mode
            if machine().is_in_sync_process(false)
            {
                gap_by_seconds = (constants::TIME_GAIN * 3) as TimeBySecT;
            } else {
                gap_by_seconds = (constants::TIME_GAIN * 6) as TimeBySecT;
            }
        }

        return gap_by_seconds;
    }


    //old_name_was getBlockInvokeGap
   fn get_block_invoke_gap(&self) -> TimeBySecT
    {
        //      return 500;
        let mut gap_by_seconds: TimeBySecT;
        if constants::TIME_GAIN == 1
        {
            if machine().is_in_sync_process(false)
            {
                gap_by_seconds = 27;  // every 27 second
            } else {
                gap_by_seconds = 120; // every 120 second
            }
        } else {
            if machine().is_in_sync_process(false)
            {
                gap_by_seconds = 17;  // every 17 second
            } else {
                gap_by_seconds = 120; // every 120 second
            }
        }
        return gap_by_seconds;
    }

    //old_name_was getNBUTXOsImportGap
    fn get_nb_coins_import_gap(&mut self) -> TimeBySecT
    {
        let mut gap_by_seconds: TimeBySecT = 11;

        if machine().is_in_sync_process(false)
        { return 333; }

        if constants::TIME_GAIN == 1
        {
            // live mode
            //     if machine().is_in_sync_process(false)
            //     {
            //         gap_by_seconds = 3;  // every 5 second controls
            //         if constants::DATABASAE_AGENT == "sqlite"
            //         {
            //             // TODO: remove this block(variable/mechanism) after fixing sqlite database lock problem
            //             if machine().m_recorded_blocks_in_db < 500 {
            //                 gap_by_seconds = 5; // every 3 second check parsing q
            //             } else if machine().m_recorded_blocks_in_db < 1000 {
            //                 gap_by_seconds = 9;
            //             } else if machine().m_recorded_blocks_in_db < 1500 {
            //                 gap_by_seconds = 11;
            //             } else if machine().m_recorded_blocks_in_db < 2000 {
            //                 gap_by_seconds = 21;
            //             } else {
            //                 gap_by_seconds = 31;
            //             }
            //         }
            //     } else {
            //         gap_by_seconds = 180; // every 3 minute
            //     }
            // } else {
            //     // develope mode
            //     if machine().is_in_sync_process(false)
            //     {
            //         gap_by_seconds = (constants::TIME_GAIN * 3) as TimeBySecT;
            //     } else {
            //         gap_by_seconds = (constants::TIME_GAIN * 6) as TimeBySecT;
            //     }
        }

        return gap_by_seconds;
    }






    // it means maximum how long we suppose some nodes creae a new block(except coinbase block)
    TimeBySecT CMachine::getAcceptableBlocksGap()
    {
      uint32_t gapByMinutes;
      if (constants::TIME_GAIN == 1)
      {
        // live
        gapByMinutes = isInSyncProcess() ? 600 : 1200;
      } else {
        // devel
        gapByMinutes = isInSyncProcess() ? (uint32_t)(constants::TIME_GAIN / 0.15) : (uint32_t)(constants::TIME_GAIN / 0.5);
      }

      CLog::log("acceptable block gap By Minutes(" + QString::number(gapByMinutes) + ") ", "app", "trace");
      return gapByMinutes;
    }

    TimeBySecT CMachine::getInvokeLeavesGap()
    {
    //      return 500;
      TimeBySecT gap_by_seconds;
      if (constants::TIME_GAIN == 1)
      {
        if (isInSyncProcess())
        {
          gap_by_seconds = 60 * 17;  // every 17 minutesd
        }else{
          gap_by_seconds = 60 * 71; // every 71 minutes
        }
      }else{
        if (isInSyncProcess())
        {
          gap_by_seconds = (constants::TIME_GAIN*60)/9;  // every 17 second
        }else{
          gap_by_seconds = (constants::TIME_GAIN*60)/3;
        }
      }
      return gap_by_seconds;
    }


        TimeBySecT CMachine::getPrerequisitiesRemoverGap()
        {
        //      return 500;
          if (constants::TIME_GAIN == 1)
          {
            if (isInSyncProcess())
            {
              return 17;  // every 17 second
            }else{
              return 120; // every 120 second
            }

          }else{
            if (isInSyncProcess())
            {
              return 17;  // every 17 second
            }else{
              return 120; // every 120 second
            }

          }
        }

        TimeBySecT CMachine::getParsingQGap()
        {
          TimeBySecT gap_by_seconds;
          if (constants::TIME_GAIN == 1)
          {
            // live
            if (isInSyncProcess())
            {
              gap_by_seconds = 1; // every 3 second check parsing q

              if (constants::DATABASAE_AGENT == "sqlite")
              {
                // TODO: remove this block(variable/mechanism) after fixing sqlite database lock problem
                if (CMachine::get().m_recorded_blocks_in_db < 500)
                {
                   gap_by_seconds = 2; // every 3 second check parsing q

                 } else if (CMachine::get().m_recorded_blocks_in_db < 1000) {
                   gap_by_seconds = 4;

                 } else if (CMachine::get().m_recorded_blocks_in_db < 1500) {
                   gap_by_seconds = 5;

                 } else if (CMachine::get().m_recorded_blocks_in_db < 2000) {
                   gap_by_seconds = 9;

                 } else {
                   gap_by_seconds = 19;

                 }
              }

            } else {
              gap_by_seconds = 63; // every 1 minutes check parsing q

              // if still
            }
          } else {
            //develop
            if (isInSyncProcess())
            {
              gap_by_seconds = constants::TIME_GAIN / 5;
            } else {
              gap_by_seconds = constants::TIME_GAIN / 1;
            }
          }
          CLog::log("parsing Q Gap every " + QString::number(gap_by_seconds) + " second");
          return gap_by_seconds;
        }

        TimeBySecT CMachine::getCoinbaseImportGap()
        {
          TimeBySecT gap_by_seconds;
          if (constants::TIME_GAIN == 1)
          {
            // live mode
            if (isInSyncProcess())
            {
              gap_by_seconds = 5; // every 1 second controls
              if (constants::DATABASAE_AGENT == "sqlite")
              {
                // TODO: remove this block(variable/mechanism) after fixing sqlite database lock problem
                 if (CMachine::get().m_recorded_blocks_in_db < 500){
                   gap_by_seconds = 1; // every 3 second check parsing q

                 } else if (CMachine::get().m_recorded_blocks_in_db < 1000) {
                   gap_by_seconds = 1;

                 } else if (CMachine::get().m_recorded_blocks_in_db < 1500) {
                   gap_by_seconds = 1;

                 } else if (CMachine::get().m_recorded_blocks_in_db < 2000) {
                   gap_by_seconds = 2;

                 } else {
                   gap_by_seconds = 15;

                 }
              }

            } else {
              // TODO: improve it in order to reduce gap if we are around midnight or midday, and increase gap if for current block the coinbase already created and the older coinbase already imported in UTXOs
              gap_by_seconds = 1200; // every 20 minute
            }

          } else {
            // develope mode
            if (isInSyncProcess())
            {
                gap_by_seconds = constants::TIME_GAIN * 3;
            } else {
                gap_by_seconds = constants::TIME_GAIN * 6;
            }
          }

         return gap_by_seconds;
        }


    TimeBySecT CMachine::getPopEmailGap()
    {
    //  return 900;
      if (constants::TIME_GAIN == 1)
      {
        // live ambient
        if (isInSyncProcess())
          return 180; // every 3 minutes check email
        return 300; // every 5 minutes check email

      } else {
        // test ambient
        if (isInSyncProcess())
          return constants::TIME_GAIN / 1; // it is testing ambianet value
        return constants::TIME_GAIN / 1; // it is testing ambianet value
      }
    }

    TimeBySecT CMachine::getSendEmailGap()
    {
    //  return 900;
      if (constants::TIME_GAIN == 1)
      {
        // live ambient
        if (isInSyncProcess())
          return 180; // every 3 minutes check email
        return 300; // every 5 minutes check email

      } else {
        // test ambient
        if (isInSyncProcess())
          return constants::TIME_GAIN / 1; // it is testing ambianet value
        return constants::TIME_GAIN / 1; // it is testing ambianet value
      }
    }

    TimeBySecT CMachine::getHardDiskReadingGap()
    {
    //  return 900;
      if (isInSyncProcess())
      {
        if (constants::TIME_GAIN == 1)
          return 20; // every 20 seconds check read a file from inbox folder (if exists)
        return 30; // it is testing ambianet value

      } else {
        if (constants::TIME_GAIN == 1)
            return 120; // every 2 minutes check read a file from inbox folder (if exists)
        return constants::TIME_GAIN / 1; // it is testing ambianet value

      }
    }

    TimeBySecT CMachine::getConcludeTreatmentGap()
    {
    //  return 900;
      if (constants::TIME_GAIN == 1)
      {
        if (isInSyncProcess())
        {
            return 11; // every 11 seconds run concluding process
        }else{
          return 71 * 60;   // every 71 minutes check concluding contracts
        }

      } else {
        if (isInSyncProcess())
        {
          return 120; // every 2 minutes run concluding process
        }else{
          return constants::TIME_GAIN / 2; // it is testing ambianet value
        }

      }
    }

    TimeBySecT CMachine::getINamesSettlementGap()
    {
    //  return 900;
      if (constants::TIME_GAIN == 1)
      {
        if (isInSyncProcess())
        {
            return 11; // every 11 seconds run concluding process
        }else{
          return 71 * 60;   // every 71 minutes check concluding contracts
        }

      } else {
        if (isInSyncProcess())
        {
          return 120; // every 2 minutes run concluding process
        }else{
          return constants::TIME_GAIN / 2; // it is testing ambianet value
        }

      }
    }

    TimeBySecT CMachine::getSendingQGap()
    {
    //  return 900;
      TimeBySecT gap_by_seconds;
      if (constants::TIME_GAIN == 1)
      {
        // live
        if (isInSyncProcess())
        {
          gap_by_seconds = 50; // every 50 seconds send to sending q
        } else {
          gap_by_seconds = 100; // every 5 minutes send to sending q
        }

      } else {
        //develop
        if (isInSyncProcess())
        {
          gap_by_seconds = constants::TIME_GAIN / 2;
        } else {
          gap_by_seconds = constants::TIME_GAIN;
        }
      }
      CLog::log("sending Q fetch Gap = " + QString::number(gap_by_seconds), "app", "trace");
      return gap_by_seconds;
    }

}
     */
