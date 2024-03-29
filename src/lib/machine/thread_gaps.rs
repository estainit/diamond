use crate::{application, CMachine};
use crate::lib::custom_types::TimeBySecT;

impl CMachine{
    /*

        //old_name_was getCoinbaseImportGap
        fn get_coinbase_import_gap(&self) -> TimeBySecT
        {
            let mut gap_by_seconds: TimeBySecT;
            if application().cycle() == 1 {
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
                    gap_by_seconds = (application().cycle() * 3) as TimeBySecT;
                } else {
                    gap_by_seconds = (application().cycle() * 6) as TimeBySecT;
                }
            }

            return gap_by_seconds;
        }


        //old_name_was getBlockInvokeGap
       fn get_block_invoke_gap(&self) -> TimeBySecT
        {
            //      return 500;
            let mut gap_by_seconds: TimeBySecT;
            if application().cycle() == 1
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

            if application().cycle() == 1
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
                //         gap_by_seconds = (application().cycle() * 3) as TimeBySecT;
                //     } else {
                //         gap_by_seconds = (application().cycle() * 6) as TimeBySecT;
                //     }
            }

            return gap_by_seconds;
        }






        // it means maximum how long we suppose some nodes creae a new block(except coinbase block)
        TimeBySecT CMachine::getAcceptableBlocksGap()
        {
          uint32_t gapByMinutes;
          if (application().cycle() == 1)
          {
            // live
            gapByMinutes = is_in_sync_process() ? 600 : 1200;
          } else {
            // devel
            gapByMinutes = is_in_sync_process() ? (uint32_t)(application().cycle() / 0.15) : (uint32_t)(application().cycle() / 0.5);
          }

          CLog::log("acceptable block gap By Minutes(" + String::number(gapByMinutes) + ") ", "app", "trace");
          return gapByMinutes;
        }

    */

    //old_name_was getInvokeLeavesGap
    pub fn get_invoke_leaves_gap(&mut self) -> TimeBySecT
    {
        //      return 500;
        let gap_by_seconds: TimeBySecT;
        if application().cycle_length() == 1 {
            if self.is_in_sync_process(false) {
                gap_by_seconds = 60 * 17;  // every 17 minutesd
            } else {
                gap_by_seconds = 60 * 71; // every 71 minutes
            }
        } else {
            if self.is_in_sync_process(false) {
                gap_by_seconds = ((application().cycle_length() * 60) / 9) as TimeBySecT;  // every 17 second
            } else {
                gap_by_seconds = ((application().cycle_length() * 60) / 3) as TimeBySecT;
            }
        }
        return gap_by_seconds;
    }

    /*


            TimeBySecT CMachine::getPrerequisitiesRemoverGap()
            {
            //      return 500;
              if (application().cycle() == 1)
              {
                if is_in_sync_process()                {
                  return 17;  // every 17 second
                }else{
                  return 120; // every 120 second
                }

              }else{
                if is_in_sync_process()                {
                  return 17;  // every 17 second
                }else{
                  return 120; // every 120 second
                }

              }
            }

            TimeBySecT CMachine::getParsingQGap()
            {
              TimeBySecT gap_by_seconds;
              if (application().cycle() == 1)
              {
                // live
                if is_in_sync_process()                {
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
                if is_in_sync_process()                {
                  gap_by_seconds = application().cycle() / 5;
                } else {
                  gap_by_seconds = application().cycle() / 1;
                }
              }
              CLog::log("parsing Q Gap every " + String::number(gap_by_seconds) + " second");
              return gap_by_seconds;
            }

            TimeBySecT CMachine::getCoinbaseImportGap()
            {
              TimeBySecT gap_by_seconds;
              if (application().cycle() == 1)
              {
                // live mode
                if is_in_sync_process()                {
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
                if is_in_sync_process()                {
                    gap_by_seconds = application().cycle() * 3;
                } else {
                    gap_by_seconds = application().cycle() * 6;
                }
              }

             return gap_by_seconds;
            }


        TimeBySecT CMachine::getPopEmailGap()
        {
        //  return 900;
          if (application().cycle() == 1)
          {
            // live ambient
            if is_in_sync_process()              return 180; // every 3 minutes check email
            return 300; // every 5 minutes check email

          } else {
            // test ambient
            if is_in_sync_process()              return application().cycle() / 1; // it is testing ambianet value
            return application().cycle() / 1; // it is testing ambianet value
          }
        }

        TimeBySecT CMachine::getSendEmailGap()
        {
        //  return 900;
          if (application().cycle() == 1)
          {
            // live ambient
            if is_in_sync_process()              return 180; // every 3 minutes check email
            return 300; // every 5 minutes check email

          } else {
            // test ambient
            if is_in_sync_process()              return application().cycle() / 1; // it is testing ambianet value
            return application().cycle() / 1; // it is testing ambianet value
          }
        }

        TimeBySecT CMachine::getHardDiskReadingGap()
        {
        //  return 900;
          if is_in_sync_process()          {
            if (application().cycle() == 1)
              return 20; // every 20 seconds check read a file from inbox folder (if exists)
            return 30; // it is testing ambianet value

          } else {
            if (application().cycle() == 1)
                return 120; // every 2 minutes check read a file from inbox folder (if exists)
            return application().cycle() / 1; // it is testing ambianet value

          }
        }

        TimeBySecT CMachine::getConcludeTreatmentGap()
        {
        //  return 900;
          if (application().cycle() == 1)
          {
            if is_in_sync_process()            {
                return 11; // every 11 seconds run concluding process
            }else{
              return 71 * 60;   // every 71 minutes check concluding contracts
            }

          } else {
            if is_in_sync_process()            {
              return 120; // every 2 minutes run concluding process
            }else{
              return application().cycle() / 2; // it is testing ambianet value
            }

          }
        }

        TimeBySecT CMachine::getINamesSettlementGap()
        {
        //  return 900;
          if (application().cycle() == 1)
          {
            if is_in_sync_process()            {
                return 11; // every 11 seconds run concluding process
            }else{
              return 71 * 60;   // every 71 minutes check concluding contracts
            }

          } else {
            if is_in_sync_process()            {
              return 120; // every 2 minutes run concluding process
            }else{
              return application().cycle() / 2; // it is testing ambianet value
            }

          }
        }

        TimeBySecT CMachine::getSendingQGap()
        {
        //  return 900;
          TimeBySecT gap_by_seconds;
          if (application().cycle() == 1)
          {
            // live
            if is_in_sync_process()            {
              gap_by_seconds = 50; // every 50 seconds send to sending q
            } else {
              gap_by_seconds = 100; // every 5 minutes send to sending q
            }

          } else {
            //develop
            if is_in_sync_process()            {
              gap_by_seconds = application().cycle() / 2;
            } else {
              gap_by_seconds = application().cycle();
            }
          }
          CLog::log("sending Q fetch Gap = " + String::number(gap_by_seconds), "app", "trace");
          return gap_by_seconds;
        }

         */
}
