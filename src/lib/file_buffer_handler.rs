
use crate::constants as CConsts;
use crate::lib::file_handler as file_handler;


// old name was maybeBootDAGFromBundle
pub fn maybe_boot_dag_from_bundle() -> bool {

    let clone_id: i8 = 1;
    let mut bundle = String::from("");
    let status = read_dag_bundle_if_exist(clone_id, &mut bundle);

    if !status || (bundle == "") { return false; };
    println!("ressssssss {}", bundle);

/*

    QJsonObject DAGBundle = CUtils::parseToJsonObj(bundle);
    QJsonArray blocks = DAGBundle.value("blocks").toArray();
    QJsonObject ballots = DAGBundle.value("ballots").toObject();

    CLog::log("Read & Dispatching (" + QString::number(blocks.size()) + ")blocks and (" + QString::number(ballots.keys().size()) + ")Ballots from DAGBundle");
    // normalizing/sanitize Ballots Receive Dates and upsert into kv
    try {
    QJsonObject sanBallots {};
    for (QString aBlt: ballots.keys())
    {
    QJsonObject a_ballot = ballots[aBlt].toObject();
    sanBallots[aBlt] = QJsonObject {
    {"baReceiveDate", CUtils::stripNonInDateString(a_ballot.value("baReceiveDate").toString())},
    {"baVoteRDiff", a_ballot.value("baVoteRDiff").toDouble()}};
    }
    KVHandler::upsertKValue("ballotsReceiveDates", CUtils::serializeJson(sanBallots));
    } catch (std::exception) {
    CLog::log("exception in reading DAGBundle", "sec", "error");

    }

    // dispatching blocks to sending q
    for(auto aBlock: blocks)
    {
    Dispatcher::dispatchMessage(
    "DAGBundle",
    aBlock.toObject(),
    CConsts::PRIVATE);
    }

    // Archive DAGBundle file in tmp folder
    FullDAGHandler::archiveDAGBundle();

    */
    return true;
}

// old name was readDAGBundleIfExist
pub fn read_dag_bundle_if_exist(
    clone_id: i8,
    mut contents: &mut String) -> bool
{
    let res = file_handler::read(
        &CConsts::HD_FILES.to_string(),
        &"DAGBundle.txt".to_string(),
        clone_id,
        &mut contents);
    return res;

}

